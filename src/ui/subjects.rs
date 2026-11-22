use iced::widget::{button, container, row, scrollable, text};
use iced::{Alignment, Element, Fill, Length};

use crate::app::{App, Message};
use crate::domain::model::Subject;
use crate::ui::{strings, theme};

pub fn view(app: &App) -> Element<'_, Message> {
    let search = app.subject_search.trim().to_lowercase();

    let subjects: Vec<&Subject> = app
        .subjects
        .iter()
        .filter(|subject| search.is_empty() || subject.name.to_lowercase().contains(&search))
        .collect();

    let body: Element<'_, Message> = if app.subjects.is_empty() {
        empty_state(
            "Пока нет предметов",
            "Добавьте предметы, чтобы выбирать их в карточке преподавателя.",
            Some(("+ Предмет", Message::OpenNewSubject)),
        )
    } else if subjects.is_empty() {
        empty_state(
            strings::NO_SEARCH_RESULTS,
            "Измените поисковый запрос.",
            None,
        )
    } else {
        let mut list = iced::widget::column![].spacing(8).width(Fill);

        for subject in subjects {
            list = list.push(subject_row(app, subject));
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

fn subject_row<'a>(app: &'a App, subject: &'a Subject) -> Element<'a, Message> {
    let teacher_count = app.subject_teacher_count(subject.id);
    let teacher_names = app.subject_teacher_names(subject.id);
    let teachers_label = if teacher_names.is_empty() {
        "Не назначен преподавателям".to_string()
    } else {
        teacher_names.join(", ")
    };

    container(
        row![
            iced::widget::column![
                text(subject.name.clone())
                    .size(15)
                    .font(theme::semibold())
                    .color(theme::TEXT),
                text(teachers_label).size(11).color(theme::MUTED),
            ]
            .spacing(2)
            .width(Fill),
            container(
                text(format!("{teacher_count} {}", word_teachers(teacher_count)))
                    .size(11)
                    .color(theme::MUTED),
            )
            .padding([4, 8])
            .style(theme::muted_box),
            button(text(strings::EDIT).size(12))
                .padding([7, 11])
                .style(theme::secondary_button)
                .on_press(Message::EditSubject(subject.id)),
            button(text(strings::DELETE).size(12))
                .padding([7, 11])
                .style(theme::ghost_button)
                .on_press(Message::AskDeleteSubject(subject.id)),
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

fn word_teachers(count: usize) -> &'static str {
    let remainder_100 = count % 100;
    let remainder_10 = count % 10;

    if (11..=14).contains(&remainder_100) {
        "преподавателей"
    } else {
        match remainder_10 {
            1 => "преподаватель",
            2..=4 => "преподавателя",
            _ => "преподавателей",
        }
    }
}
