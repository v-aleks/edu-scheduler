//! Состояние приложения и логика обновления.

mod message;
mod state;
mod update;

pub use message::{Message, SlotFormMessage, SubjectFormMessage, TeacherFormMessage};
pub use state::{
    App, Banner, CalendarView, ConfirmDeleteSubject, ConfirmDeleteTeacher, Modal, Screen,
    SlotEditor, SubjectEditor, TEACHER_COLORS, TeacherEditor, TeacherFilter, TimeField,
};
