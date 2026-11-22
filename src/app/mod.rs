//! Состояние приложения и логика обновления.

mod message;
mod state;
mod update;

pub use message::{Message, SlotFormMessage, TeacherFormMessage};
pub use state::{
    App, Banner, CalendarView, ConfirmDeleteTeacher, Modal, Screen, SlotEditor, TEACHER_COLORS,
    TeacherEditor, TeacherFilter,
};
