//! Календарь: неделя и месяц.

pub mod month;
pub mod toolbar;
pub mod week;
pub mod week_layout;

use iced::Element;

use crate::app::{App, CalendarView, Message};

pub fn view(app: &App) -> Element<'_, Message> {
    match app.view {
        CalendarView::Week => week::view(app),
        CalendarView::Month => month::view(app),
    }
}
