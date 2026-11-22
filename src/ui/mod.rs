//! Пользовательский интерфейс.

pub mod calendar;
mod modals;
mod strings;
mod teachers;
mod theme;

use iced::widget::{button, center, column, container, row, space, text};
use iced::{Alignment, Element, Fill, Length};

use crate::app::{App, Message, Screen};

pub fn view(app: &App) -> Element<'_, Message> {
    if let Some(error) = &app.startup_error {
        return error_screen(app, error);
    }

    let content: Element<'_, Message> = match app.screen {
        Screen::Calendar => calendar::view(app),
        Screen::Teachers => teachers::view(app),
    };

    let mut page = column![calendar::toolbar::view(app)]
        .spacing(0)
        .height(Fill);

    if let Some(banner) = calendar::toolbar::banner(app) {
        page = page.push(container(banner).padding([0, 16]).padding([8, 0]));
    }

    page = page.push(content);

    let page: Element<'_, Message> = container(page)
        .width(Fill)
        .height(Fill)
        .style(|_theme| container::Style {
            background: Some(theme::BACKGROUND.into()),
            ..container::Style::default()
        })
        .into();

    if let Some(modal) = &app.modal {
        modals::overlay(page, modals::view(app, modal))
    } else {
        page
    }
}

fn error_screen<'a>(app: &'a App, error: &'a str) -> Element<'a, Message> {
    center(
        container(
            column![
                text("Не удалось открыть базу данных")
                    .size(24)
                    .font(theme::bold())
                    .color(theme::TEXT),
                text(error).size(13).color(theme::DANGER),
                text(format!("Файл: {}", app.db_path.display()))
                    .size(11)
                    .color(theme::MUTED),
                row![
                    space::horizontal(),
                    button(text("Выход").size(12))
                        .padding([9, 14])
                        .style(theme::secondary_button)
                        .on_press(Message::Quit),
                    button(text("Повторить").size(12))
                        .padding([9, 16])
                        .style(theme::primary_button)
                        .on_press(Message::RetryStartup),
                ]
                .spacing(8)
                .align_y(Alignment::Center)
                .width(Length::Fill),
            ]
            .spacing(14)
            .width(Fill),
        )
        .width(Length::Fixed(620.0))
        .padding(26)
        .style(theme::modal),
    )
    .padding(30)
    .width(Fill)
    .height(Fill)
    .style(|_theme| container::Style {
        background: Some(theme::BACKGROUND.into()),
        ..container::Style::default()
    })
    .into()
}
