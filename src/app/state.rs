use std::collections::BTreeSet;
use std::path::PathBuf;

use chrono::{Local, NaiveDate};

use crate::db::Database;
use crate::domain::model::{Slot, SlotId, SlotKind, Teacher, TeacherId, Weekday};
use crate::domain::validation::{SlotDraft, TeacherDraft};
use crate::domain::{week_dates, week_start};

pub const TEACHER_COLORS: [&str; 10] = [
    "#2563EB", "#0891B2", "#059669", "#D97706", "#EA580C", "#DC2626", "#DB2777", "#7C3AED",
    "#4F46E5", "#475569",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Calendar,
    Teachers,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalendarView {
    Week,
    Month,
}

#[derive(Debug, Clone)]
pub struct Banner {
    pub text: String,
}

pub struct App {
    pub(crate) db: Option<Database>,
    pub(crate) db_path: PathBuf,
    pub(crate) startup_error: Option<String>,
    pub(crate) teachers: Vec<Teacher>,
    pub(crate) slots: Vec<Slot>,
    pub(crate) anchor: NaiveDate,
    pub(crate) view: CalendarView,
    pub(crate) screen: Screen,
    pub(crate) filter: Option<TeacherId>,
    pub(crate) modal: Option<Modal>,
    pub(crate) banner: Option<Banner>,
    pub(crate) teacher_search: String,
    pub(crate) screenshot_path: Option<PathBuf>,
    pub(crate) screenshot_requested: bool,
}

pub enum Modal {
    Slot(SlotEditor),
    Teacher(TeacherEditor),
    ConfirmDeleteTeacher(ConfirmDeleteTeacher),
}

pub struct SlotEditor {
    pub id: Option<SlotId>,
    pub kind: SlotKind,
    pub weekday: Weekday,
    pub start: String,
    pub end: String,
    pub title: String,
    pub description: String,
    pub selected_teachers: BTreeSet<TeacherId>,
    pub error: Option<String>,
    pub confirm_delete: bool,
}

pub struct TeacherEditor {
    pub id: Option<TeacherId>,
    pub name: String,
    pub subject: String,
    pub note: String,
    pub color: String,
    pub error: Option<String>,
    pub confirm_delete: bool,
}

pub struct ConfirmDeleteTeacher {
    pub id: TeacherId,
    pub name: String,
    pub assigned_slots: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TeacherFilter {
    pub id: Option<TeacherId>,
    pub label: String,
}

impl TeacherFilter {
    pub fn all() -> Self {
        Self {
            id: None,
            label: "Все преподаватели".to_string(),
        }
    }

    pub fn teacher(teacher: &Teacher) -> Self {
        Self {
            id: Some(teacher.id),
            label: teacher.display_name().to_string(),
        }
    }
}

impl std::fmt::Display for TeacherFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.label)
    }
}

impl App {
    pub fn boot(db_path: PathBuf) -> (Self, iced::Task<crate::app::Message>) {
        Self::boot_with_options(db_path, None)
    }

    pub fn boot_with_options(
        db_path: PathBuf,
        screenshot_path: Option<PathBuf>,
    ) -> (Self, iced::Task<crate::app::Message>) {
        let mut app = match Database::open(&db_path) {
            Ok(database) => Self::with_database(database, db_path),
            Err(error) => Self::failed(db_path, error.to_string()),
        };
        app.screenshot_path = screenshot_path;
        app.apply_preview_from_env();

        (app, iced::Task::none())
    }

    pub(crate) fn with_database(database: Database, db_path: PathBuf) -> Self {
        let mut app = Self {
            db: Some(database),
            db_path,
            startup_error: None,
            teachers: Vec::new(),
            slots: Vec::new(),
            anchor: Local::now().date_naive(),
            view: CalendarView::Week,
            screen: Screen::Calendar,
            filter: None,
            modal: None,
            banner: None,
            teacher_search: String::new(),
            screenshot_path: None,
            screenshot_requested: false,
        };

        if let Err(error) = app.reload() {
            app.startup_error = Some(error);
            app.db = None;
        }

        app
    }

    fn apply_preview_from_env(&mut self) {
        match std::env::var("EDU_PREVIEW").as_deref() {
            Ok("month") => self.view = CalendarView::Month,
            Ok("teachers") => self.screen = Screen::Teachers,
            Ok("slot") => self.open_new_slot(None),
            Ok("teacher") => self.open_new_teacher(),
            Ok("edit-slot") => {
                if let Some(id) = self.slots.first().map(|slot| slot.id) {
                    self.open_slot(id);
                }
            }
            _ => {}
        }
    }

    fn failed(db_path: PathBuf, error: String) -> Self {
        Self {
            db: None,
            db_path,
            startup_error: Some(error),
            teachers: Vec::new(),
            slots: Vec::new(),
            anchor: Local::now().date_naive(),
            view: CalendarView::Week,
            screen: Screen::Calendar,
            filter: None,
            modal: None,
            banner: None,
            teacher_search: String::new(),
            screenshot_path: None,
            screenshot_requested: false,
        }
    }

    pub fn retry_boot(&mut self) {
        match Database::open(&self.db_path) {
            Ok(database) => {
                self.db = Some(database);
                self.startup_error = None;

                if let Err(error) = self.reload() {
                    self.startup_error = Some(error);
                    self.db = None;
                }
            }
            Err(error) => {
                self.startup_error = Some(error.to_string());
                self.db = None;
            }
        }
    }

    pub(crate) fn reload(&mut self) -> Result<(), String> {
        let database = self
            .db
            .as_ref()
            .ok_or_else(|| "База данных недоступна".to_string())?;

        let teachers = database.teachers().map_err(|error| error.to_string())?;
        let slots = database.slots().map_err(|error| error.to_string())?;

        self.teachers = teachers;
        self.slots = slots;

        if let Some(filter) = self.filter
            && !self.teachers.iter().any(|teacher| teacher.id == filter)
        {
            self.filter = None;
        }

        Ok(())
    }

    pub(crate) fn set_banner(&mut self, text: impl Into<String>) {
        self.banner = Some(Banner { text: text.into() });
    }

    pub(crate) fn week(&self) -> [NaiveDate; 7] {
        week_dates(self.anchor)
    }

    pub(crate) fn week_start(&self) -> NaiveDate {
        week_start(self.anchor)
    }

    pub(crate) fn teacher(&self, id: TeacherId) -> Option<&Teacher> {
        self.teachers.iter().find(|teacher| teacher.id == id)
    }

    pub(crate) fn teacher_filter_options(&self) -> Vec<TeacherFilter> {
        let mut options = Vec::with_capacity(self.teachers.len() + 1);
        options.push(TeacherFilter::all());
        options.extend(self.teachers.iter().map(TeacherFilter::teacher));
        options
    }

    pub(crate) fn current_filter(&self) -> TeacherFilter {
        self.filter
            .and_then(|id| self.teacher(id))
            .map(TeacherFilter::teacher)
            .unwrap_or_else(TeacherFilter::all)
    }

    pub(crate) fn slot_count_for_teacher(&self, id: TeacherId) -> usize {
        self.slots
            .iter()
            .filter(|slot| slot.has_teacher(id))
            .count()
    }

    pub(crate) fn open_new_slot(&mut self, prefill: Option<(Weekday, u16)>) {
        let (weekday, start_minute) =
            prefill.unwrap_or_else(|| (Weekday::from_date(self.anchor), 9 * 60));

        let start_minute = round_to_quarter(start_minute.min(22 * 60));
        let end_minute = (start_minute + 90).min(23 * 60 + 59);
        let mut selected_teachers = BTreeSet::new();

        if let Some(teacher_id) = self.filter {
            selected_teachers.insert(teacher_id);
        }

        self.modal = Some(Modal::Slot(SlotEditor {
            id: None,
            kind: SlotKind::Lesson,
            weekday,
            start: crate::domain::format_time(start_minute),
            end: crate::domain::format_time(end_minute),
            title: String::new(),
            description: String::new(),
            selected_teachers,
            error: None,
            confirm_delete: false,
        }));
    }

    pub(crate) fn open_slot(&mut self, id: SlotId) {
        let Some(slot) = self.slots.iter().find(|slot| slot.id == id) else {
            self.set_banner("Слот больше не существует");
            return;
        };

        self.modal = Some(Modal::Slot(SlotEditor {
            id: Some(slot.id),
            kind: slot.kind,
            weekday: slot.weekday,
            start: crate::domain::format_time(slot.start_minute),
            end: crate::domain::format_time(slot.end_minute),
            title: slot.title.clone(),
            description: slot.description.clone(),
            selected_teachers: slot.teacher_ids.iter().copied().collect(),
            error: None,
            confirm_delete: false,
        }));
    }

    pub(crate) fn open_new_teacher(&mut self) {
        let color = TEACHER_COLORS
            .iter()
            .find(|color| {
                !self
                    .teachers
                    .iter()
                    .any(|teacher| teacher.color.eq_ignore_ascii_case(color))
            })
            .copied()
            .unwrap_or(TEACHER_COLORS[0])
            .to_string();

        self.modal = Some(Modal::Teacher(TeacherEditor {
            id: None,
            name: String::new(),
            subject: String::new(),
            note: String::new(),
            color,
            error: None,
            confirm_delete: false,
        }));
    }

    pub(crate) fn open_teacher(&mut self, id: TeacherId) {
        let Some(teacher) = self.teacher(id) else {
            self.set_banner("Преподаватель больше не существует");
            return;
        };

        self.modal = Some(Modal::Teacher(TeacherEditor {
            id: Some(teacher.id),
            name: teacher.name.clone(),
            subject: teacher.subject.clone(),
            note: teacher.note.clone(),
            color: teacher.color.clone(),
            error: None,
            confirm_delete: false,
        }));
    }

    pub(crate) fn ask_delete_teacher(&mut self, id: TeacherId) {
        let Some(teacher) = self.teacher(id) else {
            self.set_banner("Преподаватель больше не существует");
            return;
        };

        self.modal = Some(Modal::ConfirmDeleteTeacher(ConfirmDeleteTeacher {
            id,
            name: teacher.display_name().to_string(),
            assigned_slots: self.slot_count_for_teacher(id),
        }));
    }

    pub(crate) fn slot_draft(editor: &SlotEditor) -> SlotDraft {
        SlotDraft {
            id: editor.id,
            weekday: editor.weekday,
            kind: editor.kind,
            start: editor.start.clone(),
            end: editor.end.clone(),
            title: editor.title.clone(),
            description: editor.description.clone(),
            teacher_ids: editor.selected_teachers.iter().copied().collect(),
        }
    }

    pub(crate) fn teacher_draft(editor: &TeacherEditor) -> TeacherDraft {
        TeacherDraft {
            id: editor.id,
            name: editor.name.clone(),
            subject: editor.subject.clone(),
            note: editor.note.clone(),
            color: editor.color.clone(),
        }
    }
}

fn round_to_quarter(minute: u16) -> u16 {
    minute - minute % 15
}
