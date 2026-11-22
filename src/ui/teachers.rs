use iced::widget::{button, container, row, scrollable, space, text};
use iced::{Alignment, Border, Element, Fill, Length};

use crate::app::{App, Message};
use crate::domain::model::Teacher;
use crate::ui::{strings, theme};

pub fn view(app: &App) -> Element<'_, Message> {
    let search = app.teacher_search.trim().to_lowercase();

    let teachers: Vec<&Teacher> = app
        .teachers
        .iter()
        .filter(|teacher| {
            search.is_empty()
                || teacher.name.to_lowercase().contains(&search)
                || app
                    .teacher_subject_names(teacher)
                    .iter()
                    .any(|subject| subject.to_lowercase().contains(&search))
        })
        .collect();

    let body: Element<'_, Message> = if app.teachers.is_empty() {
        empty_state(
            strings::NO_TEACHERS,
            strings::NO_TEACHERS_HINT,
            Some((strings::ADD_TEACHER, Message::OpenNewTeacher)),
        )
    } else if teachers.is_empty() {
        empty_state(
            strings::NO_SEARCH_RESULTS,
            "Измените поисковый запрос.",
            None,
        )
    } else {
        let mut list = iced::widget::column![].spacing(8).width(Fill);

        for teacher in teachers {
            list = list.push(teacher_row(app, teacher));
        }

        scrollable(list)
            .height(Fill)
            .width(Fill)
            .direction(scrollable::Direction::Vertical(
                scrollable::Scrollbar::default(),
            ))
            .into()
    };

    container(body)
        .width(Fill)
        .height(Fill)
        .padding([16, 18])
        .style(|_theme| container::Style {
            background: Some(theme::BACKGROUND.into()),
            ..container::Style::default()
        })
        .into()
}

fn teacher_row<'a>(app: &'a App, teacher: &'a Teacher) -> Element<'a, Message> {
    let color = theme::teacher_color(&teacher.color);
    let slot_count = app.slot_count_for_teacher(teacher.id);

    let swatch = container(text(""))
        .width(12)
        .height(12)
        .style(move |_theme| container::Style {
            background: Some(color.into()),
            border: Border {
                color: theme::BORDER,
                width: 1.0,
                radius: 999.0.into(),
            },
            ..container::Style::default()
        });

    let subject_names = app.teacher_subject_names(teacher);
    let subject: Element<'_, Message> = if subject_names.is_empty() {
        text("Предметы не указаны")
            .size(11)
            .color(theme::MUTED)
            .into()
    } else {
        text(subject_names.join(", "))
            .size(11)
            .color(theme::MUTED)
            .into()
    };

    let note: Element<'_, Message> = if teacher.note.trim().is_empty() {
        space().width(0).into()
    } else {
        text(teacher.note.clone())
            .size(11)
            .color(theme::MUTED)
            .into()
    };

    container(
        row![
            swatch,
            iced::widget::column![
                text(teacher.display_name().to_string())
                    .size(15)
                    .font(theme::semibold())
                    .color(theme::TEXT),
                subject,
                note,
            ]
            .spacing(2)
            .width(Fill),
            container(
                text(format!("{slot_count} {}", word_slots(slot_count)))
                    .size(11)
                    .color(theme::MUTED),
            )
            .padding([4, 8])
            .style(theme::muted_box),
            button(text(strings::EDIT).size(12))
                .padding([7, 11])
                .style(theme::secondary_button)
                .on_press(Message::EditTeacher(teacher.id)),
            button(text(strings::DELETE).size(12))
                .padding([7, 11])
                .style(theme::ghost_button)
                .on_press(Message::AskDeleteTeacher(teacher.id)),
        ]
        .spacing(12)
        .align_y(Alignment::Center)
        .width(Fill),
    )
    .width(Fill)
    .padding(14)
    .style(theme::card)
    .into()
}

fn empty_state(
    title: &'static str,
    hint: &'static str,
    action: Option<(&'static str, Message)>,
) -> Element<'static, Message> {
    let mut content = iced::widget::column![
        text(title)
            .size(18)
            .font(theme::semibold())
            .color(theme::TEXT),
        text(hint).size(12).color(theme::MUTED),
    ]
    .spacing(8)
    .align_x(Alignment::Center);

    if let Some((label, message)) = action {
        content = content.push(
            button(text(label).size(12))
                .padding([9, 14])
                .style(theme::primary_button)
                .on_press(message),
        );
    }

    container(content)
        .width(Fill)
        .height(Length::Fill)
        .center_x(Fill)
        .center_y(Fill)
        .padding(40)
        .into()
}

fn word_slots(count: usize) -> &'static str {
    let remainder_100 = count % 100;
    let remainder_10 = count % 10;

    if (11..=14).contains(&remainder_100) {
        "слотов"
    } else {
        match remainder_10 {
            1 => "слот",
            2..=4 => "слота",
            _ => "слотов",
        }
    }
}
