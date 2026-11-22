use std::collections::{BTreeSet, HashSet};

use crate::domain::model::{
    Slot, SlotId, SlotKind, Subject, SubjectId, Teacher, TeacherId, Weekday,
};

pub const TEACHER_NAME_MAX: usize = 120;
pub const SUBJECT_NAME_MAX: usize = 120;
pub const TEACHER_NOTE_MAX: usize = 500;
pub const SLOT_TITLE_MAX: usize = 120;
pub const SLOT_DESCRIPTION_MAX: usize = 2000;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TimeError {
    #[error("Укажите время в формате ЧЧ:ММ, например 09:00")]
    Format,
    #[error("Часы должны быть от 00 до 23")]
    Hours,
    #[error("Минуты должны быть от 00 до 59")]
    Minutes,
}

pub fn parse_time(value: &str) -> Result<u16, TimeError> {
    let value = value.trim();
    let (hours, minutes) = value.split_once(':').ok_or(TimeError::Format)?;

    if hours.is_empty() || minutes.is_empty() {
        return Err(TimeError::Format);
    }

    let hours: u16 = hours.parse().map_err(|_| TimeError::Format)?;
    let minutes: u16 = minutes.parse().map_err(|_| TimeError::Format)?;

    if hours > 23 {
        return Err(TimeError::Hours);
    }

    if minutes > 59 {
        return Err(TimeError::Minutes);
    }

    Ok(hours * 60 + minutes)
}

pub fn format_time(minutes: u16) -> String {
    format!("{:02}:{:02}", minutes / 60, minutes % 60)
}

pub fn format_time_range(start_minute: u16, end_minute: u16) -> String {
    format!("{}–{}", format_time(start_minute), format_time(end_minute))
}

pub fn overlaps(a_start: u16, a_end: u16, b_start: u16, b_end: u16) -> bool {
    a_start < b_end && b_start < a_end
}

#[derive(Debug, Clone, Default)]
pub struct TeacherDraft {
    pub id: Option<TeacherId>,
    pub name: String,
    pub subject_ids: Vec<SubjectId>,
    pub note: String,
    pub color: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedTeacher {
    pub id: Option<TeacherId>,
    pub name: String,
    pub subject_ids: Vec<SubjectId>,
    pub note: String,
    pub color: String,
}

pub fn validate_teacher_draft(
    draft: &TeacherDraft,
    subjects: &[Subject],
) -> Result<ValidatedTeacher, Vec<String>> {
    let mut errors = Vec::new();
    let name = draft.name.trim().to_string();
    let note = draft.note.trim().to_string();

    if name.is_empty() {
        errors.push("Имя преподавателя не может быть пустым".to_string());
    } else if name.chars().count() > TEACHER_NAME_MAX {
        errors.push(format!(
            "Имя преподавателя не должно быть длиннее {TEACHER_NAME_MAX} символов"
        ));
    }

    if note.chars().count() > TEACHER_NOTE_MAX {
        errors.push(format!(
            "Заметка не должна быть длиннее {TEACHER_NOTE_MAX} символов"
        ));
    }

    if !is_valid_color(&draft.color) {
        errors.push("Выберите цвет преподавателя".to_string());
    }

    let known_subjects: HashSet<SubjectId> = subjects.iter().map(|subject| subject.id).collect();
    let mut subject_ids = Vec::new();
    let mut seen = HashSet::new();

    for subject_id in &draft.subject_ids {
        if !known_subjects.contains(subject_id) {
            errors.push("Выбран неизвестный предмет".to_string());
        } else if seen.insert(*subject_id) {
            subject_ids.push(*subject_id);
        }
    }

    subject_ids.sort_unstable();

    if errors.is_empty() {
        Ok(ValidatedTeacher {
            id: draft.id,
            name,
            subject_ids,
            note,
            color: draft.color.trim().to_string(),
        })
    } else {
        Err(errors)
    }
}

#[derive(Debug, Clone, Default)]
pub struct SubjectDraft {
    pub id: Option<SubjectId>,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedSubject {
    pub id: Option<SubjectId>,
    pub name: String,
}

pub fn validate_subject_draft(draft: &SubjectDraft) -> Result<ValidatedSubject, Vec<String>> {
    let mut errors = Vec::new();
    let name = draft.name.trim().to_string();

    if name.is_empty() {
        errors.push("Название предмета не может быть пустым".to_string());
    } else if name.chars().count() > SUBJECT_NAME_MAX {
        errors.push(format!(
            "Название предмета не должно быть длиннее {SUBJECT_NAME_MAX} символов"
        ));
    }

    if errors.is_empty() {
        Ok(ValidatedSubject { id: draft.id, name })
    } else {
        Err(errors)
    }
}

fn is_valid_color(color: &str) -> bool {
    let color = color.trim();
    color.len() == 7
        && color.starts_with('#')
        && color[1..]
            .chars()
            .all(|character| character.is_ascii_hexdigit())
}

#[derive(Debug, Clone)]
pub struct SlotDraft {
    pub id: Option<SlotId>,
    pub weekday: Weekday,
    pub kind: SlotKind,
    pub start: String,
    pub end: String,
    pub title: String,
    pub description: String,
    pub teacher_ids: Vec<TeacherId>,
}

impl Default for SlotDraft {
    fn default() -> Self {
        Self {
            id: None,
            weekday: Weekday::Monday,
            kind: SlotKind::Lesson,
            start: "09:00".to_string(),
            end: "10:30".to_string(),
            title: String::new(),
            description: String::new(),
            teacher_ids: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotCandidate {
    pub id: Option<SlotId>,
    pub weekday: Weekday,
    pub start_minute: u16,
    pub end_minute: u16,
    pub kind: SlotKind,
    pub title: String,
    pub description: String,
    pub teacher_ids: Vec<TeacherId>,
}

impl SlotDraft {
    pub fn validate(&self, teachers: &[Teacher]) -> Result<SlotCandidate, Vec<String>> {
        let mut errors = Vec::new();

        let start_minute = match parse_time(&self.start) {
            Ok(minutes) => Some(minutes),
            Err(error) => {
                errors.push(format!("Начало: {error}"));
                None
            }
        };

        let end_minute = match parse_time(&self.end) {
            Ok(minutes) => Some(minutes),
            Err(error) => {
                errors.push(format!("Конец: {error}"));
                None
            }
        };

        if let (Some(start), Some(end)) = (start_minute, end_minute)
            && end <= start
        {
            errors.push("Время окончания должно быть позже времени начала".to_string());
        }

        let title = self.title.trim().to_string();
        let description = self.description.trim().to_string();

        if title.chars().count() > SLOT_TITLE_MAX {
            errors.push(format!(
                "Название не должно быть длиннее {SLOT_TITLE_MAX} символов"
            ));
        }

        if description.chars().count() > SLOT_DESCRIPTION_MAX {
            errors.push(format!(
                "Описание не должно быть длиннее {SLOT_DESCRIPTION_MAX} символов"
            ));
        }

        let known_teachers: HashSet<TeacherId> =
            teachers.iter().map(|teacher| teacher.id).collect();
        let mut teacher_ids = Vec::new();
        let mut seen = HashSet::new();

        for teacher_id in &self.teacher_ids {
            if !known_teachers.contains(teacher_id) {
                errors.push("Выбран неизвестный преподаватель".to_string());
            } else if seen.insert(*teacher_id) {
                teacher_ids.push(*teacher_id);
            }
        }

        teacher_ids.sort_unstable();

        match (start_minute, end_minute) {
            (Some(start_minute), Some(end_minute)) if errors.is_empty() => Ok(SlotCandidate {
                id: self.id,
                weekday: self.weekday,
                start_minute,
                end_minute,
                kind: self.kind,
                title,
                description,
                teacher_ids,
            }),
            _ => Err(errors),
        }
    }
}

pub fn validate_slot_draft(
    draft: &SlotDraft,
    teachers: &[Teacher],
) -> Result<SlotCandidate, Vec<String>> {
    draft.validate(teachers)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotConflict {
    pub teacher_name: String,
    pub slot: Slot,
}

impl std::fmt::Display for SlotConflict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: пересечение с «{}» ({}, {}–{})",
            self.teacher_name,
            self.slot.display_title(),
            self.slot.weekday.short_label(),
            format_time(self.slot.start_minute),
            format_time(self.slot.end_minute)
        )
    }
}

pub fn find_conflicts(
    candidate: &SlotCandidate,
    existing: &[Slot],
    teachers: &[Teacher],
) -> Vec<SlotConflict> {
    let teacher_names: std::collections::HashMap<TeacherId, &str> = teachers
        .iter()
        .map(|teacher| (teacher.id, teacher.display_name()))
        .collect();

    let mut conflicts = Vec::new();
    let mut seen = BTreeSet::new();

    for teacher_id in &candidate.teacher_ids {
        let Some(teacher_name) = teacher_names.get(teacher_id) else {
            continue;
        };

        for slot in existing {
            if Some(slot.id) == candidate.id {
                continue;
            }

            if slot.weekday != candidate.weekday
                || !slot.has_teacher(*teacher_id)
                || !overlaps(
                    candidate.start_minute,
                    candidate.end_minute,
                    slot.start_minute,
                    slot.end_minute,
                )
            {
                continue;
            }

            if seen.insert((*teacher_id, slot.id)) {
                conflicts.push(SlotConflict {
                    teacher_name: (*teacher_name).to_string(),
                    slot: slot.clone(),
                });
            }
        }
    }

    conflicts
}

pub fn validation_message(errors: &[String]) -> String {
    errors.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn teacher(id: i64, name: &str) -> Teacher {
        Teacher {
            id,
            name: name.to_string(),
            subject_ids: Vec::new(),
            note: String::new(),
            color: "#2563EB".to_string(),
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    fn subject(id: i64, name: &str) -> Subject {
        Subject {
            id,
            name: name.to_string(),
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    fn slot(id: i64, weekday: Weekday, start: u16, end: u16, teachers: Vec<i64>) -> Slot {
        Slot {
            id,
            weekday,
            start_minute: start,
            end_minute: end,
            kind: SlotKind::Lesson,
            title: "Математика".to_string(),
            description: String::new(),
            teacher_ids: teachers,
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    fn draft(start: &str, end: &str, teachers: Vec<i64>) -> SlotDraft {
        SlotDraft {
            start: start.to_string(),
            end: end.to_string(),
            teacher_ids: teachers,
            ..SlotDraft::default()
        }
    }

    #[test]
    fn parses_and_formats_time() {
        assert_eq!(parse_time("09:00"), Ok(540));
        assert_eq!(parse_time("9:05"), Ok(545));
        assert_eq!(parse_time(" 23:59 "), Ok(1439));
        assert_eq!(format_time(540), "09:00");
        assert_eq!(format_time(1439), "23:59");
        assert_eq!(format_time_range(540, 630), "09:00–10:30");
    }

    #[test]
    fn rejects_invalid_time() {
        assert_eq!(parse_time("24:00"), Err(TimeError::Hours));
        assert_eq!(parse_time("09:60"), Err(TimeError::Minutes));
        assert_eq!(parse_time("900"), Err(TimeError::Format));
        assert_eq!(parse_time("09:"), Err(TimeError::Format));
    }

    #[test]
    fn overlap_touching_boundaries_do_not_conflict() {
        assert!(overlaps(540, 630, 600, 690));
        assert!(!overlaps(540, 630, 630, 720));
        assert!(!overlaps(630, 720, 540, 630));
    }

    #[test]
    fn validates_slot_time_order_and_lengths() {
        let errors = draft("10:00", "09:00", vec![]).validate(&[]).unwrap_err();
        assert!(errors.iter().any(|error| error.contains("позже")));

        let mut long_title = draft("09:00", "10:00", vec![]);
        long_title.title = "а".repeat(SLOT_TITLE_MAX + 1);
        assert!(long_title.validate(&[]).is_err());

        let candidate = draft("09:00", "10:00", vec![2, 2])
            .validate(&[teacher(2, "Иванов")])
            .unwrap();
        assert_eq!(candidate.teacher_ids, vec![2]);
    }

    #[test]
    fn resolves_teacher_conflicts() {
        let teachers = vec![teacher(1, "Иванов"), teacher(2, "Петров")];
        let existing = vec![
            slot(10, Weekday::Monday, 540, 630, vec![1]),
            slot(11, Weekday::Monday, 630, 720, vec![1]),
            slot(12, Weekday::Tuesday, 540, 630, vec![2]),
        ];

        let mut candidate_draft = draft("09:30", "10:15", vec![1, 2]);
        candidate_draft.weekday = Weekday::Monday;
        let candidate = candidate_draft.validate(&teachers).unwrap();
        let conflicts = find_conflicts(&candidate, &existing, &teachers);

        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].teacher_name, "Иванов");
        assert_eq!(conflicts[0].slot.id, 10);
    }

    #[test]
    fn editing_slot_ignores_itself() {
        let teachers = vec![teacher(1, "Иванов")];
        let existing = vec![slot(10, Weekday::Monday, 540, 630, vec![1])];

        let mut edit = draft("09:00", "10:30", vec![1]);
        edit.id = Some(10);

        let candidate = edit.validate(&teachers).unwrap();
        assert!(find_conflicts(&candidate, &existing, &teachers).is_empty());
    }

    #[test]
    fn validates_teacher() {
        let subjects = vec![subject(1, "Математика"), subject(2, "Физика")];
        let valid = TeacherDraft {
            name: "  Иванов Иван  ".to_string(),
            subject_ids: vec![2, 1, 2],
            color: "#2563EB".to_string(),
            ..TeacherDraft::default()
        };

        let teacher = validate_teacher_draft(&valid, &subjects).unwrap();
        assert_eq!(teacher.name, "Иванов Иван");
        assert_eq!(teacher.subject_ids, vec![1, 2]);

        let invalid = TeacherDraft {
            color: "red".to_string(),
            subject_ids: vec![99],
            ..TeacherDraft::default()
        };
        assert!(validate_teacher_draft(&invalid, &subjects).is_err());
    }

    #[test]
    fn validates_subject() {
        let valid = SubjectDraft {
            name: "  Математика  ".to_string(),
            ..SubjectDraft::default()
        };
        assert_eq!(validate_subject_draft(&valid).unwrap().name, "Математика");

        let invalid = SubjectDraft {
            name: "   ".to_string(),
            ..SubjectDraft::default()
        };
        assert!(validate_subject_draft(&invalid).is_err());
    }
}
