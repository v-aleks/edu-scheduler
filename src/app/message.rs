use chrono::NaiveDate;

use crate::app::state::{CalendarView, TimeField};
use crate::domain::model::{SlotId, SlotKind, SubjectId, TeacherId, Weekday};

use super::state::TeacherFilter;

#[derive(Debug, Clone)]
pub enum Message {
    Previous,
    Next,
    Today,
    SetView(CalendarView),
    ShowCalendar,
    ShowTeachers,
    ShowSubjects,
    FilterSelected(TeacherFilter),
    OpenNewSlot,
    OpenSlot(SlotId),
    SlotMoved {
        id: SlotId,
        weekday: Weekday,
        start_minute: u16,
    },
    OpenNewTeacher,
    EditTeacher(TeacherId),
    AskDeleteTeacher(TeacherId),
    DeleteTeacher(TeacherId),
    OpenNewSubject,
    EditSubject(SubjectId),
    AskDeleteSubject(SubjectId),
    DeleteSubject(SubjectId),
    CloseModal,
    DismissBanner,
    TeacherSearch(String),
    SubjectSearch(String),
    SlotForm(SlotFormMessage),
    TeacherForm(TeacherFormMessage),
    SubjectForm(SubjectFormMessage),
    WeekSlotClicked(SlotId),
    WeekEmptyClicked {
        weekday: Weekday,
        minute: u16,
    },
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
    ToggleTimePicker(TimeField),
    SetHour(TimeField, u8),
    SetMinute(TimeField, u8),
    TimePickerDone,
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
    Note(String),
    Color(String),
    ToggleSubject(SubjectId),
    Save,
    Cancel,
    RequestDelete,
    ConfirmDelete,
    CancelDelete,
}

#[derive(Debug, Clone)]
pub enum SubjectFormMessage {
    Name(String),
    Save,
    Cancel,
    RequestDelete,
    ConfirmDelete,
    CancelDelete,
}
