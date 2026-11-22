use iced::widget::{button, checkbox, container, row, scrollable, space, text, text_input};
use iced::{Alignment, Border, Element, Fill, Length};

use crate::app::{App, Message, TeacherEditor, TeacherFormMessage as FormMessage};
use crate::ui::{strings, theme};

pub fn view<'a>(app: &'a App, editor: &'a TeacherEditor) -> Element<'a, Message> {
    let title = if editor.id.is_some() {
        "Редактирование преподавателя"
    } else {
        "Новый преподаватель"
    };

    let header = row![
        text(title).size(20).font(theme::bold()).color(theme::TEXT),
        space::horizontal(),
        button(text("×").size(20))
            .padding([0, 8])
            .style(theme::ghost_button)
            .on_press(Message::CloseModal),
    ]
    .align_y(Alignment::Center);

    let name = text_input("Иванов Иван Иванович", &editor.name)
        .on_input(|value| Message::TeacherForm(FormMessage::Name(value)))
        .padding([8, 10])
        .size(13);

    let note = text_input("Комментарий", &editor.note)
        .on_input(|value| Message::TeacherForm(FormMessage::Note(value)))
        .padding([8, 10])
        .size(13);

    let mut palette = row![].spacing(8).align_y(Alignment::Center);

    for color in crate::app::TEACHER_COLORS {
        let selected = editor.color.eq_ignore_ascii_case(color);
        let parsed = theme::teacher_color(color);

        palette = palette.push(
            button(text("").size(1))
                .width(30)
                .height(30)
                .padding(0)
                .style(theme::color_swatch(parsed, selected))
                .on_press(Message::TeacherForm(FormMessage::Color(color.to_string()))),
        );
    }

    let mut subject_rows = iced::widget::column![].spacing(4).width(Fill);

    if app.subjects.is_empty() {
        subject_rows = subject_rows.push(
            text("Предметов пока нет. Добавьте их на экране «Предметы».")
                .size(12)
                .color(theme::MUTED),
        );
    } else {
        for subject in &app.subjects {
            let selected = editor.selected_subjects.contains(&subject.id);
            let subject_id = subject.id;

            subject_rows = subject_rows.push(
                checkbox(selected)
                    .label(subject.name.clone())
                    .size(15)
                    .text_size(13)
                    .on_toggle(move |_| {
                        Message::TeacherForm(FormMessage::ToggleSubject(subject_id))
                    }),
            );
        }
    }

    let subjects = container(scrollable(subject_rows).height(132).direction(
        scrollable::Direction::Vertical(scrollable::Scrollbar::default()),
    ))
    .padding(10)
    .width(Fill)
    .style(theme::muted_box);

    let mut content = iced::widget::column![
        section("Имя", name.into()),
        section("Предметы", subjects.into()),
        section("Заметка", note.into()),
        section("Цвет", palette.into()),
    ]
    .spacing(14)
    .width(Fill);

    if editor.id.is_some() {
        let assigned = app.slot_count_for_teacher(editor.id.unwrap_or_default());
        content = content.push(
            text(format!("Слотов с этим преподавателем: {assigned}"))
                .size(11)
                .color(theme::MUTED),
        );
    }

    if let Some(error) = &editor.error {
        content = content.push(
            container(text(error.as_str()).size(12).color(theme::DANGER))
                .padding(10)
                .width(Fill)
                .style(|_theme| container::Style {
                    background: Some(theme::DANGER_SOFT.into()),
                    border: Border {
                        color: theme::DANGER_SOFT,
                        width: 1.0,
                        radius: 8.0.into(),
                    },
                    ..container::Style::default()
                }),
        );
    }

    container(
        iced::widget::column![header, content, footer(editor)]
            .spacing(16)
            .width(Fill),
    )
    .width(Length::Fixed(500.0))
    .padding(22)
    .style(theme::modal)
    .into()
}

fn footer(editor: &TeacherEditor) -> Element<'_, Message> {
    if editor.confirm_delete {
        return container(
            row![
                text("Удалить преподавателя? Он будет снят со всех слотов.")
                    .size(12)
                    .color(theme::DANGER),
                space::horizontal(),
                button(text(strings::CANCEL).size(12))
                    .padding([7, 12])
                    .style(theme::secondary_button)
                    .on_press(Message::TeacherForm(FormMessage::CancelDelete)),
                button(text(strings::DELETE).size(12))
                    .padding([7, 12])
                    .style(theme::danger_button)
                    .on_press(Message::TeacherForm(FormMessage::ConfirmDelete)),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .padding(10)
        .width(Fill)
        .style(|_theme| container::Style {
            background: Some(theme::DANGER_SOFT.into()),
            border: Border {
                color: theme::DANGER_SOFT,
                width: 1.0,
                radius: 8.0.into(),
            },
            ..container::Style::default()
        })
        .into();
    }

    row![
        if editor.id.is_some() {
            Element::from(
                button(text(strings::DELETE).size(12))
                    .padding([9, 12])
                    .style(theme::ghost_button)
                    .on_press(Message::TeacherForm(FormMessage::RequestDelete)),
            )
        } else {
            Element::from(space().width(0))
        },
        space::horizontal(),
        button(text(strings::CANCEL).size(12))
            .padding([9, 14])
            .style(theme::secondary_button)
            .on_press(Message::CloseModal),
        button(text(strings::SAVE).size(12))
            .padding([9, 16])
            .style(theme::primary_button)
            .on_press(Message::TeacherForm(FormMessage::Save)),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .into()
}

fn section<'a>(label: &'static str, control: Element<'a, Message>) -> Element<'a, Message> {
    iced::widget::column![
        text(label)
            .size(11)
            .font(theme::semibold())
            .color(theme::MUTED),
        control,
    ]
    .spacing(5)
    .width(Fill)
    .into()
}
