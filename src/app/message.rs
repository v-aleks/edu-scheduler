use chrono::NaiveDate;

use crate::app::state::CalendarView;
use crate::domain::model::{SlotId, SlotKind, TeacherId, Weekday};

use super::state::TeacherFilter;

#[derive(Debug, Clone)]
pub enum Message {
    Previous,
    Next,
    Today,
    SetView(CalendarView),
    ShowCalendar,
    ShowTeachers,
    FilterSelected(TeacherFilter),
    OpenNewSlot,
    OpenSlot(SlotId),
    OpenNewTeacher,
    EditTeacher(TeacherId),
    AskDeleteTeacher(TeacherId),
    DeleteTeacher(TeacherId),
    CloseModal,
    DismissBanner,
    TeacherSearch(String),
    SlotForm(SlotFormMessage),
    TeacherForm(TeacherFormMessage),
    WeekSlotClicked(SlotId),
    WeekEmptyClicked { weekday: Weekday, minute: u16 },
    MonthDaySelected(NaiveDate),
    RetryStartup,
    Quit,
    ScreenshotTick,
    ScreenshotSaved(Result<(), String>),
    Event(iced::Event),
}

#[derive(Debug, Clone)]
pub enum SlotFormMessage {
    Kind(SlotKind),
    Weekday(Weekday),
    Start(String),
    End(String),
    Title(String),
    Description(String),
    ToggleTeacher(TeacherId),
    Save,
    Cancel,
    RequestDelete,
    ConfirmDelete,
    CancelDelete,
}

#[derive(Debug, Clone)]
pub enum TeacherFormMessage {
    Name(String),
    Subject(String),
    Note(String),
    Color(String),
    Save,
    Cancel,
    RequestDelete,
    ConfirmDelete,
    CancelDelete,
}
