use iced::widget::{button, container, pick_list, row, space, text, text_input};
use iced::{Alignment, Element, Fill, Length};

use crate::app::{App, CalendarView, Message, Screen};
use crate::domain::date::{format_month, format_week_range};
use crate::ui::{strings, theme};

pub fn view(app: &App) -> Element<'_, Message> {
    match app.screen {
        Screen::Calendar => calendar_toolbar(app),
        Screen::Teachers => teachers_toolbar(app),
    }
}

pub fn banner(app: &App) -> Option<Element<'_, Message>> {
    let banner = app.banner.as_ref()?;

    Some(
        container(
            row![
                text(banner.text.clone()).size(13).color(theme::TEXT),
                space::horizontal(),
                button(text("×").size(18))
                    .padding([0, 8])
                    .style(theme::ghost_button)
                    .on_press(Message::DismissBanner),
            ]
            .align_y(Alignment::Center)
            .spacing(12),
        )
        .padding([10, 14])
        .width(Fill)
        .style(theme::banner)
        .into(),
    )
}

fn calendar_toolbar(app: &App) -> Element<'_, Message> {
    let period = match app.view {
        CalendarView::Week => format_week_range(app.week_start()),
        CalendarView::Month => format_month(app.anchor),
    };

    let title_row = row![
        iced::widget::column![
            text(strings::CALENDAR)
                .size(22)
                .font(theme::bold())
                .color(theme::TEXT),
            text("Управление недельным расписанием преподавателей")
                .size(12)
                .color(theme::MUTED),
        ]
        .spacing(2),
        space::horizontal(),
        button(text(strings::ADD_SLOT).size(13))
            .padding([9, 14])
            .style(theme::primary_button)
            .on_press(Message::OpenNewSlot),
        button(text(strings::TEACHERS).size(13))
            .padding([9, 14])
            .style(theme::secondary_button)
            .on_press(Message::ShowTeachers),
    ]
    .align_y(Alignment::Center)
    .spacing(10);

    let navigation = row![
        button(text("←").size(14))
            .padding([7, 11])
            .style(theme::secondary_button)
            .on_press(Message::Previous),
        button(text(strings::TODAY).size(13))
            .padding([7, 12])
            .style(theme::secondary_button)
            .on_press(Message::Today),
        button(text("→").size(14))
            .padding([7, 11])
            .style(theme::secondary_button)
            .on_press(Message::Next),
        text(period)
            .size(16)
            .font(theme::semibold())
            .color(theme::TEXT),
        space::horizontal(),
        view_switch(app.view),
        teacher_filter(app),
    ]
    .align_y(Alignment::Center)
    .spacing(8);

    container(
        iced::widget::column![title_row, navigation]
            .spacing(12)
            .width(Fill),
    )
    .padding([14, 18])
    .width(Fill)
    .style(|_theme| container::Style {
        background: Some(theme::SURFACE.into()),
        border: iced::Border {
            color: theme::BORDER,
            width: 0.0,
            radius: 0.0.into(),
        },
        ..container::Style::default()
    })
    .into()
}

fn view_switch(current: CalendarView) -> Element<'static, Message> {
    let week = toggle_button(
        "Неделя",
        current == CalendarView::Week,
        Message::SetView(CalendarView::Week),
    );
    let month = toggle_button(
        "Месяц",
        current == CalendarView::Month,
        Message::SetView(CalendarView::Month),
    );

    container(row![week, month].spacing(2))
        .padding(3)
        .style(theme::muted_box)
        .into()
}

fn toggle_button(
    label: &'static str,
    selected: bool,
    message: Message,
) -> Element<'static, Message> {
    button(text(label).size(12))
        .padding([6, 12])
        .style(if selected {
            theme::active_toggle
        } else {
            theme::inactive_toggle
        })
        .on_press(message)
        .into()
}

fn teacher_filter(app: &App) -> Element<'_, Message> {
    let options = app.teacher_filter_options();

    pick_list(options, Some(app.current_filter()), Message::FilterSelected)
        .width(Length::Fixed(210.0))
        .padding([7, 10])
        .text_size(12)
        .into()
}

fn teachers_toolbar(app: &App) -> Element<'_, Message> {
    let title_row = row![
        button(text(strings::BACK_TO_CALENDAR).size(13))
            .padding([8, 11])
            .style(theme::ghost_button)
            .on_press(Message::ShowCalendar),
        iced::widget::column![
            text(strings::TEACHERS)
                .size(22)
                .font(theme::bold())
                .color(theme::TEXT),
            text(format!("Всего: {}", app.teachers.len()))
                .size(12)
                .color(theme::MUTED),
        ]
        .spacing(2),
        space::horizontal(),
        text_input(strings::SEARCH_TEACHERS, &app.teacher_search)
            .on_input(Message::TeacherSearch)
            .padding([8, 10])
            .size(13)
            .width(Length::Fixed(280.0)),
        button(text(strings::ADD_TEACHER).size(13))
            .padding([9, 14])
            .style(theme::primary_button)
            .on_press(Message::OpenNewTeacher),
    ]
    .align_y(Alignment::Center)
    .spacing(10);

    container(title_row)
        .padding([14, 18])
        .width(Fill)
        .style(|_theme| container::Style {
            background: Some(theme::SURFACE.into()),
            border: iced::Border {
                color: theme::BORDER,
                width: 0.0,
                radius: 0.0.into(),
            },
            ..container::Style::default()
        })
        .into()
}
