//! Доменная модель, валидация и работа с датами.

pub mod date;
pub mod model;
pub mod validation;

pub use date::{format_month, format_week_range, month_grid, week_dates, week_start};
pub use model::{Slot, SlotId, SlotKind, Teacher, TeacherId, Weekday};
pub use validation::{
    SlotConflict, SlotDraft, TeacherDraft, find_conflicts, format_time, overlaps, parse_time,
    validate_slot_draft, validate_teacher_draft,
};
