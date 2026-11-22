use chrono::Datelike;

pub type TeacherId = i64;
pub type SlotId = i64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Teacher {
    pub id: TeacherId,
    pub name: String,
    pub subject: String,
    pub note: String,
    pub color: String,
    pub created_at: String,
    pub updated_at: String,
}

impl Teacher {
    pub fn display_name(&self) -> &str {
        let trimmed = self.name.trim();
        if trimmed.is_empty() {
            "Без имени"
        } else {
            trimmed
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Weekday {
    Monday = 1,
    Tuesday = 2,
    Wednesday = 3,
    Thursday = 4,
    Friday = 5,
    Saturday = 6,
    Sunday = 7,
}

impl Weekday {
    pub const ALL: [Weekday; 7] = [
        Weekday::Monday,
        Weekday::Tuesday,
        Weekday::Wednesday,
        Weekday::Thursday,
        Weekday::Friday,
        Weekday::Saturday,
        Weekday::Sunday,
    ];

    pub const fn number(self) -> i64 {
        self as i64
    }

    pub fn from_number(number: i64) -> Option<Self> {
        match number {
            1 => Some(Weekday::Monday),
            2 => Some(Weekday::Tuesday),
            3 => Some(Weekday::Wednesday),
            4 => Some(Weekday::Thursday),
            5 => Some(Weekday::Friday),
            6 => Some(Weekday::Saturday),
            7 => Some(Weekday::Sunday),
            _ => None,
        }
    }

    pub const fn index(self) -> usize {
        (self as usize) - 1
    }

    pub const fn short_label(self) -> &'static str {
        match self {
            Weekday::Monday => "Пн",
            Weekday::Tuesday => "Вт",
            Weekday::Wednesday => "Ср",
            Weekday::Thursday => "Чт",
            Weekday::Friday => "Пт",
            Weekday::Saturday => "Сб",
            Weekday::Sunday => "Вс",
        }
    }

    pub const fn long_label(self) -> &'static str {
        match self {
            Weekday::Monday => "Понедельник",
            Weekday::Tuesday => "Вторник",
            Weekday::Wednesday => "Среда",
            Weekday::Thursday => "Четверг",
            Weekday::Friday => "Пятница",
            Weekday::Saturday => "Суббота",
            Weekday::Sunday => "Воскресенье",
        }
    }

    pub fn from_chrono(weekday: chrono::Weekday) -> Self {
        match weekday {
            chrono::Weekday::Mon => Weekday::Monday,
            chrono::Weekday::Tue => Weekday::Tuesday,
            chrono::Weekday::Wed => Weekday::Wednesday,
            chrono::Weekday::Thu => Weekday::Thursday,
            chrono::Weekday::Fri => Weekday::Friday,
            chrono::Weekday::Sat => Weekday::Saturday,
            chrono::Weekday::Sun => Weekday::Sunday,
        }
    }

    pub fn from_date(date: chrono::NaiveDate) -> Self {
        Self::from_chrono(date.weekday())
    }
}

impl std::fmt::Display for Weekday {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.long_label())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotKind {
    Lesson,
    Break,
}

impl SlotKind {
    pub const ALL: [SlotKind; 2] = [SlotKind::Lesson, SlotKind::Break];

    pub const fn label(self) -> &'static str {
        match self {
            SlotKind::Lesson => "Занятие",
            SlotKind::Break => "Перемена",
        }
    }

    pub const fn default_title(self) -> &'static str {
        match self {
            SlotKind::Lesson => "",
            SlotKind::Break => "Перемена",
        }
    }

    pub const fn as_db(self) -> &'static str {
        match self {
            SlotKind::Lesson => "lesson",
            SlotKind::Break => "break",
        }
    }

    pub fn from_db(value: &str) -> Option<Self> {
        match value {
            "lesson" => Some(SlotKind::Lesson),
            "break" => Some(SlotKind::Break),
            _ => None,
        }
    }
}

impl std::fmt::Display for SlotKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Slot {
    pub id: SlotId,
    pub weekday: Weekday,
    pub start_minute: u16,
    pub end_minute: u16,
    pub kind: SlotKind,
    pub title: String,
    pub description: String,
    pub teacher_ids: Vec<TeacherId>,
    pub created_at: String,
    pub updated_at: String,
}

impl Slot {
    pub fn duration_minutes(&self) -> u16 {
        self.end_minute.saturating_sub(self.start_minute)
    }

    pub fn display_title(&self) -> &str {
        let title = self.title.trim();
        if title.is_empty() {
            self.kind.default_title()
        } else {
            title
        }
    }

    pub fn has_teacher(&self, teacher_id: TeacherId) -> bool {
        self.teacher_ids.contains(&teacher_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn weekday_numbers_roundtrip() {
        for weekday in Weekday::ALL {
            assert_eq!(Weekday::from_number(weekday.number()), Some(weekday));
        }

        assert_eq!(Weekday::from_number(0), None);
        assert_eq!(Weekday::from_number(8), None);
    }

    #[test]
    fn weekday_from_date() {
        let monday = NaiveDate::from_ymd_opt(2026, 11, 16).unwrap();
        assert_eq!(Weekday::from_date(monday), Weekday::Monday);

        let sunday = NaiveDate::from_ymd_opt(2026, 11, 22).unwrap();
        assert_eq!(Weekday::from_date(sunday), Weekday::Sunday);
    }

    #[test]
    fn slot_kind_roundtrip() {
        for kind in SlotKind::ALL {
            assert_eq!(SlotKind::from_db(kind.as_db()), Some(kind));
        }

        assert_eq!(SlotKind::from_db("unknown"), None);
    }

    #[test]
    fn display_title_falls_back_to_kind() {
        let slot = Slot {
            id: 1,
            weekday: Weekday::Monday,
            start_minute: 540,
            end_minute: 630,
            kind: SlotKind::Break,
            title: "   ".to_string(),
            description: String::new(),
            teacher_ids: vec![],
            created_at: String::new(),
            updated_at: String::new(),
        };

        assert_eq!(slot.display_title(), "Перемена");
        assert_eq!(slot.duration_minutes(), 90);
    }
}
