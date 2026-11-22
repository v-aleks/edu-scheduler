use rusqlite::Connection;

pub const SCHEMA_VERSION: i64 = 2;

const SCHEMA_V1: &str = r#"
CREATE TABLE teachers (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL CHECK (length(trim(name)) > 0),
  subject TEXT NOT NULL DEFAULT '',
  note TEXT NOT NULL DEFAULT '',
  color TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE slots (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  weekday INTEGER NOT NULL CHECK (weekday BETWEEN 1 AND 7),
  start_minute INTEGER NOT NULL CHECK (start_minute BETWEEN 0 AND 1439),
  end_minute INTEGER NOT NULL CHECK (end_minute BETWEEN 1 AND 1440),
  kind TEXT NOT NULL CHECK (kind IN ('lesson', 'break')),
  title TEXT NOT NULL DEFAULT '',
  description TEXT NOT NULL DEFAULT '',
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  CHECK (end_minute > start_minute)
);

CREATE TABLE slot_teachers (
  slot_id INTEGER NOT NULL REFERENCES slots(id) ON DELETE CASCADE,
  teacher_id INTEGER NOT NULL REFERENCES teachers(id) ON DELETE CASCADE,
  PRIMARY KEY (slot_id, teacher_id)
);

CREATE INDEX idx_slots_weekday_time ON slots(weekday, start_minute);
CREATE INDEX idx_slot_teachers_teacher ON slot_teachers(teacher_id);
"#;

const SCHEMA_V2: &str = r#"
CREATE TABLE subjects (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL CHECK (length(trim(name)) > 0),
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX idx_subjects_name ON subjects(name);

CREATE TABLE teacher_subjects (
  teacher_id INTEGER NOT NULL REFERENCES teachers(id) ON DELETE CASCADE,
  subject_id INTEGER NOT NULL REFERENCES subjects(id) ON DELETE CASCADE,
  PRIMARY KEY (teacher_id, subject_id)
);

CREATE INDEX idx_teacher_subjects_subject ON teacher_subjects(subject_id);

INSERT INTO subjects (name)
SELECT DISTINCT trim(subject)
FROM teachers
WHERE trim(subject) <> '';

INSERT INTO teacher_subjects (teacher_id, subject_id)
SELECT teachers.id, subjects.id
FROM teachers
JOIN subjects ON subjects.name = trim(teachers.subject)
WHERE trim(teachers.subject) <> '';

ALTER TABLE teachers DROP COLUMN subject;
"#;

pub fn migrate(connection: &mut Connection) -> Result<(), rusqlite::Error> {
    let version: i64 = connection.pragma_query_value(None, "user_version", |row| row.get(0))?;

    if version > SCHEMA_VERSION {
        return Err(rusqlite::Error::InvalidParameterName(format!(
            "Неподдерживаемая версия схемы БД: {version}"
        )));
    }

    if version == 0 {
        let transaction = connection.transaction()?;
        transaction.execute_batch(SCHEMA_V1)?;
        transaction.execute_batch(SCHEMA_V2)?;
        transaction.pragma_update(None, "user_version", SCHEMA_VERSION)?;
        transaction.commit()?;
    } else if version == 1 {
        let transaction = connection.transaction()?;
        transaction.execute_batch(SCHEMA_V2)?;
        transaction.pragma_update(None, "user_version", SCHEMA_VERSION)?;
        transaction.commit()?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrates_fresh_database() {
        let mut connection = Connection::open_in_memory().unwrap();
        connection
            .execute_batch("PRAGMA foreign_keys = ON;")
            .unwrap();

        migrate(&mut connection).unwrap();
        migrate(&mut connection).unwrap();

        let version: i64 = connection
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(version, SCHEMA_VERSION);

        let tables: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name IN ('teachers', 'slots', 'slot_teachers', 'subjects', 'teacher_subjects')",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(tables, 5);
    }

    #[test]
    fn migrates_v1_subjects_without_data_loss() {
        let mut connection = Connection::open_in_memory().unwrap();
        connection
            .execute_batch("PRAGMA foreign_keys = ON;")
            .unwrap();
        connection.execute_batch(SCHEMA_V1).unwrap();
        connection
            .execute(
                "INSERT INTO teachers (name, subject, color) VALUES ('Иванов', 'Математика', '#2563EB')",
                [],
            )
            .unwrap();
        connection.pragma_update(None, "user_version", 1).unwrap();

        migrate(&mut connection).unwrap();

        let subject: String = connection
            .query_row("SELECT name FROM subjects", [], |row| row.get(0))
            .unwrap();
        assert_eq!(subject, "Математика");

        let link_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM teacher_subjects", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(link_count, 1);

        let has_subject_column = connection
            .prepare("PRAGMA table_info(teachers)")
            .unwrap()
            .query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
            .iter()
            .any(|column| column == "subject");
        assert!(!has_subject_column);
    }
}
