//! Модальные окна.

mod confirm;
mod slot_editor;
mod subject_editor;
mod teacher_editor;

use iced::widget::{center, container, mouse_area, opaque, stack};
use iced::{Color, Element};

use crate::app::{App, Message, Modal};

pub fn view<'a>(app: &'a App, modal: &'a Modal) -> Element<'a, Message> {
    match modal {
        Modal::Slot(editor) => slot_editor::view(app, editor),
        Modal::Teacher(editor) => teacher_editor::view(app, editor),
        Modal::Subject(editor) => subject_editor::view(editor),
        Modal::ConfirmDeleteTeacher(confirm) => confirm::view(confirm),
        Modal::ConfirmDeleteSubject(confirm) => confirm::subject_view(confirm),
    }
}

pub fn overlay<'a>(
    base: Element<'a, Message>,
    content: Element<'a, Message>,
) -> Element<'a, Message> {
    stack![
        base,
        opaque(
            mouse_area(center(opaque(content)).style(|_theme| {
                container::Style {
                    background: Some(
                        Color {
                            a: 0.45,
                            ..Color::BLACK
                        }
                        .into(),
                    ),
                    ..container::Style::default()
                }
            }))
            .on_press(Message::CloseModal)
        )
    ]
    .into()
}
