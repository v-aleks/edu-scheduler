use iced::widget::{
    button, checkbox, container, pick_list, row, scrollable, space, text, text_input,
};
use iced::{Alignment, Border, Element, Fill, Length};

use crate::app::{App, Message, SlotEditor, SlotFormMessage as FormMessage, TimeField};
use crate::domain::date::format_duration;
use crate::domain::model::{SlotKind, Weekday};
use crate::domain::validation::{format_time, parse_time};
use crate::ui::{strings, theme};

pub fn view<'a>(app: &'a App, editor: &'a SlotEditor) -> Element<'a, Message> {
    let title = if editor.id.is_some() {
        "Редактирование слота"
    } else {
        "Новый слот"
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

    let kind = row![
        toggle(
            "Занятие",
            editor.kind == SlotKind::Lesson,
            FormMessage::Kind(SlotKind::Lesson)
        ),
        toggle(
            "Перемена",
            editor.kind == SlotKind::Break,
            FormMessage::Kind(SlotKind::Break)
        ),
    ]
    .spacing(4);

    let weekday = pick_list(&Weekday::ALL[..], Some(editor.weekday), move |weekday| {
        Message::SlotForm(FormMessage::Weekday(weekday))
    })
    .width(Fill)
    .padding([8, 10])
    .text_size(13);

    let start_button = time_button(
        &editor.start,
        editor.active_time_picker == Some(TimeField::Start),
        TimeField::Start,
    );
    let end_button = time_button(
        &editor.end,
        editor.active_time_picker == Some(TimeField::End),
        TimeField::End,
    );

    let duration = match (parse_time(&editor.start), parse_time(&editor.end)) {
        (Ok(start), Ok(end)) if end > start => {
            text(format!("Длительность: {}", format_duration(end - start)))
                .size(11)
                .color(theme::MUTED)
        }
        (Ok(_), Ok(_)) => text("Конец должен быть позже начала")
            .size(11)
            .color(theme::DANGER),
        _ => text("Выберите время начала и окончания")
            .size(11)
            .color(theme::MUTED),
    };

    let time_row = row![
        field("Начало", start_button),
        field("Конец", end_button),
        container(duration)
            .padding([22, 0])
            .width(Fill)
            .align_x(Alignment::Start),
    ]
    .spacing(12)
    .align_y(Alignment::Start);

    let time_section: Element<'_, Message> = if let Some(time_field) = editor.active_time_picker {
        iced::widget::column![time_row, time_picker(editor, time_field)]
            .spacing(8)
            .into()
    } else {
        time_row.into()
    };

    let title_input = text_input("Например, математика", &editor.title)
        .on_input(|value| Message::SlotForm(FormMessage::Title(value)))
        .padding([8, 10])
        .size(13);

    let description = text_input(
        "Дополнительные детали, аудитория, тема занятия…",
        &editor.description,
    )
    .on_input(|value| Message::SlotForm(FormMessage::Description(value)))
    .padding([8, 10])
    .size(13);

    let mut teacher_rows = iced::widget::column![].spacing(4).width(Fill);

    if app.teachers.is_empty() {
        teacher_rows = teacher_rows.push(
            text("Преподавателей пока нет. Добавьте их на экране «Преподаватели».")
                .size(12)
                .color(theme::MUTED),
        );
    } else {
        for teacher in &app.teachers {
            let selected = editor.selected_teachers.contains(&teacher.id);
            let teacher_id = teacher.id;
            let color = theme::teacher_color(&teacher.color);
            let swatch = container(text(""))
                .width(10)
                .height(10)
                .style(move |_theme| container::Style {
                    background: Some(color.into()),
                    border: Border {
                        color: theme::BORDER,
                        width: 1.0,
                        radius: 999.0.into(),
                    },
                    ..container::Style::default()
                });

            let label = app.teacher_label(teacher);

            teacher_rows = teacher_rows.push(
                row![
                    swatch,
                    checkbox(selected)
                        .label(label)
                        .size(15)
                        .size(13)
                        .on_toggle(move |_| Message::SlotForm(FormMessage::ToggleTeacher(
                            teacher_id
                        ))),
                ]
                .spacing(8)
                .align_y(Alignment::Center)
                .width(Fill),
            );
        }
    }

    let teachers = container(scrollable(teacher_rows).height(138))
        .padding(10)
        .width(Fill)
        .style(theme::muted_box);

    let mut content = iced::widget::column![
        section("Тип", kind.into()),
        section("День недели", weekday.into()),
        section("Время", time_section),
        section("Название", title_input.into()),
        section("Описание", description.into()),
        section("Преподаватели", teachers.into()),
    ]
    .spacing(12)
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

    let footer = footer(editor);

    container(
        iced::widget::column![
            header,
            scrollable(content).height(Length::Fixed(550.0)),
            footer,
        ]
        .spacing(14)
        .width(Fill),
    )
    .width(Length::Fixed(580.0))
    .padding(22)
    .style(theme::modal)
    .into()
}

fn time_button<'a>(value: &'a str, active: bool, field: TimeField) -> Element<'a, Message> {
    button(
        row![
            text(value)
                .size(15)
                .font(theme::semibold())
                .color(if active { theme::ACCENT } else { theme::TEXT }),
            space::horizontal(),
            text("▾").size(11).color(theme::MUTED),
        ]
        .align_y(Alignment::Center),
    )
    .width(Length::Fixed(132.0))
    .padding([8, 10])
    .style(theme::secondary_button)
    .on_press(Message::SlotForm(FormMessage::ToggleTimePicker(field)))
    .into()
}

fn time_picker(editor: &SlotEditor, field: TimeField) -> Element<'_, Message> {
    let (value, default, title) = match field {
        TimeField::Start => (&editor.start, 9 * 60, "Время начала"),
        TimeField::End => (&editor.end, 10 * 60 + 30, "Время окончания"),
    };
    let current = parse_time(value).unwrap_or(default);
    let selected_hour = (current / 60) as u8;
    let selected_minute = (current % 60) as u8;

    let hours: Vec<u8> = (0..24).collect();
    let mut minutes: Vec<u8> = (0..60).step_by(5).collect();

    if !minutes.contains(&selected_minute) {
        minutes.push(selected_minute);
        minutes.sort_unstable();
    }

    let hour_message = move |hour: u8| Message::SlotForm(FormMessage::SetHour(field, hour));
    let minute_message = move |minute: u8| Message::SlotForm(FormMessage::SetMinute(field, minute));

    let content = iced::widget::column![
        row![
            text(title)
                .size(11)
                .font(theme::semibold())
                .color(theme::MUTED),
            space::horizontal(),
            text(format_time(current))
                .size(15)
                .font(theme::bold())
                .color(theme::ACCENT),
            button(text("Готово").size(11))
                .padding([4, 9])
                .style(theme::ghost_button)
                .on_press(Message::SlotForm(FormMessage::TimePickerDone)),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
        text("Часы").size(10).color(theme::MUTED),
        number_grid(&hours, selected_hour, 12, hour_message),
        text("Минуты").size(10).color(theme::MUTED),
        number_grid(&minutes, selected_minute, 12, minute_message),
    ]
    .spacing(6)
    .width(Fill);

    container(content)
        .padding(12)
        .width(Fill)
        .style(theme::muted_box)
        .into()
}

fn number_grid(
    values: &[u8],
    selected: u8,
    columns: usize,
    message: impl Fn(u8) -> Message + Copy + 'static,
) -> Element<'static, Message> {
    let mut grid = iced::widget::column![].spacing(4).width(Fill);

    for chunk in values.chunks(columns) {
        let mut grid_row = row![].spacing(4).width(Fill);

        for value in chunk {
            grid_row = grid_row.push(
                button(text(format!("{value:02}")).size(11))
                    .width(Fill)
                    .padding([5, 0])
                    .style(if *value == selected {
                        theme::active_toggle
                    } else {
                        theme::inactive_toggle
                    })
                    .on_press(message(*value)),
            );
        }

        for _ in chunk.len()..columns {
            grid_row = grid_row.push(space().width(Fill));
        }

        grid = grid.push(grid_row);
    }

    grid.into()
}

fn footer(editor: &SlotEditor) -> Element<'_, Message> {
    if editor.confirm_delete {
        return container(
            row![
                text("Удалить слот без возможности восстановления?")
                    .size(12)
                    .color(theme::DANGER),
                space::horizontal(),
                button(text(strings::CANCEL).size(12))
                    .padding([7, 12])
                    .style(theme::secondary_button)
                    .on_press(Message::SlotForm(FormMessage::CancelDelete)),
                button(text(strings::DELETE).size(12))
                    .padding([7, 12])
                    .style(theme::danger_button)
                    .on_press(Message::SlotForm(FormMessage::ConfirmDelete)),
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
                    .on_press(Message::SlotForm(FormMessage::RequestDelete)),
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
            .on_press(Message::SlotForm(FormMessage::Save)),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .into()
}

fn toggle(label: &'static str, selected: bool, message: FormMessage) -> Element<'static, Message> {
    button(text(label).size(12))
        .padding([7, 14])
        .style(if selected {
            theme::active_toggle
        } else {
            theme::inactive_toggle
        })
        .on_press(Message::SlotForm(message))
        .into()
}

fn field<'a>(label: &'static str, control: Element<'a, Message>) -> Element<'a, Message> {
    section(label, control)
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
