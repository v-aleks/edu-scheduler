use iced::widget::{button, container, row, space, text, text_input};
use iced::{Alignment, Border, Element, Fill, Length};

use crate::app::{Message, SubjectEditor, SubjectFormMessage as FormMessage};
use crate::ui::{strings, theme};

pub fn view(editor: &SubjectEditor) -> Element<'_, Message> {
    let title = if editor.id.is_some() {
        "Редактирование предмета"
    } else {
        "Новый предмет"
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

    let name = text_input("Например, математика", &editor.name)
        .on_input(|value| Message::SubjectForm(FormMessage::Name(value)))
        .padding([8, 10])
        .size(13);

    let mut content = iced::widget::column![section("Название", name.into())]
        .spacing(14)
        .width(Fill);

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
    .width(Length::Fixed(460.0))
    .padding(22)
    .style(theme::modal)
    .into()
}

fn footer(editor: &SubjectEditor) -> Element<'_, Message> {
    if editor.confirm_delete {
        return container(
            row![
                text("Удалить предмет? Он будет убран у всех преподавателей.")
                    .size(12)
                    .color(theme::DANGER),
                space::horizontal(),
                button(text(strings::CANCEL).size(12))
                    .padding([7, 12])
                    .style(theme::secondary_button)
                    .on_press(Message::SubjectForm(FormMessage::CancelDelete)),
                button(text(strings::DELETE).size(12))
                    .padding([7, 12])
                    .style(theme::danger_button)
                    .on_press(Message::SubjectForm(FormMessage::ConfirmDelete)),
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
                    .on_press(Message::SubjectForm(FormMessage::RequestDelete)),
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
            .on_press(Message::SubjectForm(FormMessage::Save)),
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
