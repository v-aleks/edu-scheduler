use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use rusqlite::{Connection, params};

use crate::db::migrations::{SCHEMA_VERSION, migrate};
use crate::domain::model::{Slot, SlotKind, Teacher, Weekday};
use crate::domain::validation::{SlotCandidate, ValidatedTeacher};

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("Ошибка базы данных: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("Не удалось создать каталог данных {path}: {source}")]
    CreateDir {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("Преподаватель с id {0} не найден")]
    TeacherNotFound(i64),

    #[error("Слот с id {0} не найден")]
    SlotNotFound(i64),

    #[error("Неподдерживаемая версия схемы БД: {0}")]
    UnsupportedSchema(i64),

    #[error("Повреждённые данные в БД: {0}")]
    CorruptData(String),
}

pub struct Database {
    connection: Connection,
    path: PathBuf,
}

impl Database {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, DbError> {
        let path = path.as_ref().to_path_buf();

        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            fs::create_dir_all(parent).map_err(|source| DbError::CreateDir {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        let mut connection = Connection::open(&path)?;
        configure(&connection)?;

        let version: i64 = connection.pragma_query_value(None, "user_version", |row| row.get(0))?;

        if version > SCHEMA_VERSION {
            return Err(DbError::UnsupportedSchema(version));
        }

        migrate(&mut connection)?;

        Ok(Self { connection, path })
    }

    pub fn open_in_memory() -> Result<Self, DbError> {
        let mut connection = Connection::open_in_memory()?;
        configure(&connection)?;
        migrate(&mut connection)?;

        Ok(Self {
            connection,
            path: PathBuf::from(":memory:"),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn teachers(&self) -> Result<Vec<Teacher>, DbError> {
        let mut statement = self.connection.prepare(
            "SELECT id, name, subject, note, color, created_at, updated_at
             FROM teachers",
        )?;

        let mut teachers = statement
            .query_map([], |row| {
                Ok(Teacher {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    subject: row.get(2)?,
                    note: row.get(3)?,
                    color: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        teachers.sort_by(|left, right| {
            left.name
                .to_lowercase()
                .cmp(&right.name.to_lowercase())
                .then_with(|| left.id.cmp(&right.id))
        });

        Ok(teachers)
    }

    pub fn slots(&self) -> Result<Vec<Slot>, DbError> {
        let mut assignments: HashMap<i64, Vec<i64>> = HashMap::new();

        {
            let mut statement = self.connection.prepare(
                "SELECT slot_id, teacher_id FROM slot_teachers
                 ORDER BY slot_id, teacher_id",
            )?;
            let rows = statement
                .query_map([], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)))?;

            for row in rows {
                let (slot_id, teacher_id) = row?;
                assignments.entry(slot_id).or_default().push(teacher_id);
            }
        }

        let mut statement = self.connection.prepare(
            "SELECT id, weekday, start_minute, end_minute, kind, title,
                    description, created_at, updated_at
             FROM slots
             ORDER BY weekday, start_minute, id",
        )?;

        let rows = statement.query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, String>(7)?,
                row.get::<_, String>(8)?,
            ))
        })?;

        let mut slots = Vec::new();

        for row in rows {
            let (
                id,
                weekday,
                start_minute,
                end_minute,
                kind,
                title,
                description,
                created_at,
                updated_at,
            ) = row?;

            let weekday = Weekday::from_number(weekday).ok_or_else(|| {
                DbError::CorruptData(format!("неизвестный день недели {weekday}"))
            })?;
            let kind = SlotKind::from_db(&kind)
                .ok_or_else(|| DbError::CorruptData(format!("неизвестный тип слота {kind}")))?;

            slots.push(Slot {
                id,
                weekday,
                start_minute: u16::try_from(start_minute)
                    .map_err(|_| DbError::CorruptData("некорректное время начала".to_string()))?,
                end_minute: u16::try_from(end_minute).map_err(|_| {
                    DbError::CorruptData("некорректное время окончания".to_string())
                })?,
                kind,
                title,
                description,
                teacher_ids: assignments.remove(&id).unwrap_or_default(),
                created_at,
                updated_at,
            });
        }

        Ok(slots)
    }

    pub fn save_teacher(&mut self, teacher: &ValidatedTeacher) -> Result<i64, DbError> {
        match teacher.id {
            Some(id) => {
                let changed = self.connection.execute(
                    "UPDATE teachers
                     SET name = ?1, subject = ?2, note = ?3, color = ?4,
                         updated_at = CURRENT_TIMESTAMP
                     WHERE id = ?5",
                    params![
                        teacher.name,
                        teacher.subject,
                        teacher.note,
                        teacher.color,
                        id
                    ],
                )?;

                if changed == 0 {
                    return Err(DbError::TeacherNotFound(id));
                }

                Ok(id)
            }
            None => {
                self.connection.execute(
                    "INSERT INTO teachers (name, subject, note, color)
                     VALUES (?1, ?2, ?3, ?4)",
                    params![teacher.name, teacher.subject, teacher.note, teacher.color],
                )?;

                Ok(self.connection.last_insert_rowid())
            }
        }
    }

    pub fn delete_teacher(&mut self, id: i64) -> Result<(), DbError> {
        let deleted = self
            .connection
            .execute("DELETE FROM teachers WHERE id = ?1", params![id])?;

        if deleted == 0 {
            return Err(DbError::TeacherNotFound(id));
        }

        Ok(())
    }

    pub fn save_slot(&mut self, slot: &SlotCandidate) -> Result<i64, DbError> {
        let transaction = self.connection.transaction()?;

        let id = match slot.id {
            Some(id) => {
                let changed = transaction.execute(
                    "UPDATE slots
                     SET weekday = ?1, start_minute = ?2, end_minute = ?3,
                         kind = ?4, title = ?5, description = ?6,
                         updated_at = CURRENT_TIMESTAMP
                     WHERE id = ?7",
                    params![
                        slot.weekday.number(),
                        slot.start_minute,
                        slot.end_minute,
                        slot.kind.as_db(),
                        slot.title,
                        slot.description,
                        id
                    ],
                )?;

                if changed == 0 {
                    return Err(DbError::SlotNotFound(id));
                }

                id
            }
            None => {
                transaction.execute(
                    "INSERT INTO slots
                        (weekday, start_minute, end_minute, kind, title, description)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![
                        slot.weekday.number(),
                        slot.start_minute,
                        slot.end_minute,
                        slot.kind.as_db(),
                        slot.title,
                        slot.description
                    ],
                )?;

                transaction.last_insert_rowid()
            }
        };

        transaction.execute("DELETE FROM slot_teachers WHERE slot_id = ?1", params![id])?;

        {
            let mut statement = transaction
                .prepare("INSERT INTO slot_teachers (slot_id, teacher_id) VALUES (?1, ?2)")?;

            for teacher_id in &slot.teacher_ids {
                statement.execute(params![id, teacher_id])?;
            }
        }

        transaction.commit()?;

        Ok(id)
    }

    pub fn delete_slot(&mut self, id: i64) -> Result<(), DbError> {
        let deleted = self
            .connection
            .execute("DELETE FROM slots WHERE id = ?1", params![id])?;

        if deleted == 0 {
            return Err(DbError::SlotNotFound(id));
        }

        Ok(())
    }
}

fn configure(connection: &Connection) -> Result<(), DbError> {
    connection.execute_batch(
        "PRAGMA foreign_keys = ON;
         PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;",
    )?;
    connection.busy_timeout(Duration::from_secs(5))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::Weekday;
    use crate::domain::validation::ValidatedTeacher;

    fn teacher(name: &str) -> ValidatedTeacher {
        ValidatedTeacher {
            id: None,
            name: name.to_string(),
            subject: String::new(),
            note: String::new(),
            color: "#2563EB".to_string(),
        }
    }

    fn slot(weekday: Weekday, start: u16, end: u16, teacher_ids: Vec<i64>) -> SlotCandidate {
        SlotCandidate {
            id: None,
            weekday,
            start_minute: start,
            end_minute: end,
            kind: SlotKind::Lesson,
            title: "Математика".to_string(),
            description: "Алгебра".to_string(),
            teacher_ids,
        }
    }

    #[test]
    fn saves_and_loads_teachers() {
        let mut database = Database::open_in_memory().unwrap();
        let first = database.save_teacher(&teacher("Пётр")).unwrap();
        let second = database.save_teacher(&teacher("Анна")).unwrap();

        let teachers = database.teachers().unwrap();
        assert_eq!(teachers.len(), 2);
        assert_eq!(teachers[0].name, "Анна");

        let mut updated = teacher("Анна Петрова");
        updated.id = Some(second);
        database.save_teacher(&updated).unwrap();

        let teachers = database.teachers().unwrap();
        assert_eq!(teachers[0].name, "Анна Петрова");
        assert_eq!(teachers[1].id, first);
    }

    #[test]
    fn saves_slot_with_teachers_and_updates_assignments() {
        let mut database = Database::open_in_memory().unwrap();
        let first = database.save_teacher(&teacher("Иванов")).unwrap();
        let second = database.save_teacher(&teacher("Петров")).unwrap();

        let mut candidate = slot(Weekday::Monday, 540, 630, vec![first, second]);
        let id = database.save_slot(&candidate).unwrap();

        let slots = database.slots().unwrap();
        assert_eq!(slots.len(), 1);
        assert_eq!(slots[0].teacher_ids, vec![first, second]);
        assert_eq!(slots[0].duration_minutes(), 90);

        candidate.id = Some(id);
        candidate.end_minute = 660;
        candidate.teacher_ids = vec![second];
        database.save_slot(&candidate).unwrap();

        let slots = database.slots().unwrap();
        assert_eq!(slots[0].end_minute, 660);
        assert_eq!(slots[0].teacher_ids, vec![second]);
    }

    #[test]
    fn deleting_teacher_keeps_slots_but_removes_assignments() {
        let mut database = Database::open_in_memory().unwrap();
        let first = database.save_teacher(&teacher("Иванов")).unwrap();
        let second = database.save_teacher(&teacher("Петров")).unwrap();
        database
            .save_slot(&slot(Weekday::Monday, 540, 630, vec![first, second]))
            .unwrap();

        database.delete_teacher(first).unwrap();

        let slots = database.slots().unwrap();
        assert_eq!(slots.len(), 1);
        assert_eq!(slots[0].teacher_ids, vec![second]);
    }

    #[test]
    fn deleting_slot_removes_assignments() {
        let mut database = Database::open_in_memory().unwrap();
        let first = database.save_teacher(&teacher("Иванов")).unwrap();
        let id = database
            .save_slot(&slot(Weekday::Monday, 540, 630, vec![first]))
            .unwrap();

        database.delete_slot(id).unwrap();

        assert!(database.slots().unwrap().is_empty());
        assert_eq!(database.teachers().unwrap().len(), 1);
    }

    #[test]
    fn rejects_over_100_percent_teacher_ids_via_foreign_keys() {
        let mut database = Database::open_in_memory().unwrap();
        let result = database.save_slot(&slot(Weekday::Monday, 540, 630, vec![999]));
        assert!(result.is_err());
    }

    #[test]
    fn persists_to_file() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("schedule.db");

        {
            let mut database = Database::open(&path).unwrap();
            database.save_teacher(&teacher("Иванов")).unwrap();
        }

        let database = Database::open(&path).unwrap();
        assert_eq!(database.teachers().unwrap().len(), 1);
    }
}
