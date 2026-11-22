use iced::widget::{button, container, row, space, text};
use iced::{Alignment, Element, Length};

use crate::app::{ConfirmDeleteTeacher, Message};
use crate::ui::{strings, theme};

pub fn view(confirm: &ConfirmDeleteTeacher) -> Element<'_, Message> {
    let usage = if confirm.assigned_slots == 0 {
        "Преподаватель пока не назначен ни на один слот.".to_string()
    } else {
        format!(
            "Преподаватель будет снят со слотов: {}. Сами слоты сохранятся.",
            confirm.assigned_slots
        )
    };

    container(
        iced::widget::column![
            text("Удалить преподавателя?")
                .size(18)
                .font(theme::bold())
                .color(theme::TEXT),
            text(format!("«{}»", confirm.name))
                .size(14)
                .color(theme::TEXT),
            text(usage).size(12).color(theme::MUTED),
            row![
                space::horizontal(),
                button(text(strings::CANCEL).size(12))
                    .padding([9, 14])
                    .style(theme::secondary_button)
                    .on_press(Message::CloseModal),
                button(text(strings::DELETE).size(12))
                    .padding([9, 14])
                    .style(theme::danger_button)
                    .on_press(Message::DeleteTeacher(confirm.id)),
            ]
            .spacing(8)
            .align_y(Alignment::Center)
            .width(Length::Fill),
        ]
        .spacing(12)
        .width(Length::Fill),
    )
    .width(Length::Fixed(440.0))
    .padding(22)
    .style(theme::modal)
    .into()
}
