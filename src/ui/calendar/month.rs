//! Месячный вид — проекция недельного шаблона на календарь.

use chrono::{Datelike, Local, NaiveDate};
use iced::widget::{button, container, row, scrollable, space, text};
use iced::{Alignment, Border, Element, Fill, Length};

use crate::app::{App, Message};
use crate::domain::date::{month_grid, same_month};
use crate::domain::model::{Slot, Weekday};
use crate::domain::validation::format_time;
use crate::ui::theme;

const CELL_HEIGHT: f32 = 122.0;

pub fn view(app: &App) -> Element<'_, Message> {
    let today = Local::now().date_naive();
    let grid = month_grid(app.anchor);

    let header = row(Weekday::ALL.map(|weekday| {
        container(
            text(weekday.short_label())
                .size(11)
                .font(theme::semibold())
                .color(theme::MUTED),
        )
        .width(Fill)
        .padding([7, 6])
        .align_x(Alignment::Center)
        .into()
    }))
    .width(Fill);

    let mut weeks = iced::widget::column![].spacing(0).width(Fill);

    for week in grid {
        let mut week_row = row![].width(Fill);

        for date in week {
            week_row = week_row.push(day_cell(app, date, today));
        }

        weeks = weeks.push(week_row);
    }

    let content = iced::widget::column![header, weeks].width(Fill);

    container(scrollable(content).height(Fill).width(Fill).direction(
        scrollable::Direction::Vertical(scrollable::Scrollbar::default()),
    ))
    .width(Fill)
    .height(Fill)
    .padding(12)
    .style(theme::card)
    .into()
}

fn day_cell(app: &App, date: NaiveDate, today: NaiveDate) -> Element<'_, Message> {
    let is_current_month = same_month(date, app.anchor);
    let is_today = date == today;

    let mut slots: Vec<&Slot> = app
        .slots
        .iter()
        .filter(|slot| slot.weekday == Weekday::from_date(date))
        .filter(|slot| match app.filter {
            Some(teacher_id) => slot.has_teacher(teacher_id),
            None => true,
        })
        .collect();

    slots.sort_by_key(|slot| (slot.start_minute, slot.id));

    let mut content = iced::widget::column![].spacing(3).width(Fill);

    let day_label = button(text(date.day().to_string()).size(12).color(if is_today {
        theme::ACCENT
    } else if is_current_month {
        theme::TEXT
    } else {
        theme::MUTED
    }))
    .padding([1, 3])
    .style(theme::ghost_button)
    .on_press(Message::MonthDaySelected(date));

    content = content.push(
        row![day_label, space::horizontal()]
            .align_y(Alignment::Center)
            .width(Fill),
    );

    for slot in slots.iter().take(2) {
        content = content.push(chip(app, slot));
    }

    if slots.len() > 2 {
        content = content.push(
            button(
                text(format!("+{} ещё", slots.len() - 2))
                    .size(9)
                    .color(theme::MUTED),
            )
            .padding([1, 3])
            .style(theme::ghost_button)
            .on_press(Message::MonthDaySelected(date)),
        );
    }

    if slots.is_empty() {
        content = content.push(space().height(Length::Fill));
    }

    container(content)
        .width(Fill)
        .height(CELL_HEIGHT)
        .padding(6)
        .style(move |_theme| container::Style {
            background: Some(
                if is_today {
                    theme::TODAY_BG
                } else if is_current_month {
                    theme::SURFACE
                } else {
                    theme::WEEKEND_BG
                }
                .into(),
            ),
            border: Border {
                color: theme::BORDER,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..container::Style::default()
        })
        .into()
}

fn chip<'a>(app: &App, slot: &'a Slot) -> Element<'a, Message> {
    let teacher_names: Vec<&str> = slot
        .teacher_ids
        .iter()
        .filter_map(|teacher_id| {
            app.teachers
                .iter()
                .find(|teacher| teacher.id == *teacher_id)
                .map(|teacher| teacher.display_name())
        })
        .collect();

    let suffix = teacher_names.join(", ");

    let label = if suffix.is_empty() {
        format!(
            "{} {}",
            format_time(slot.start_minute),
            slot.display_title()
        )
    } else {
        format!(
            "{} {} · {}",
            format_time(slot.start_minute),
            slot.display_title(),
            suffix
        )
    };

    button(text(label).size(9).color(theme::TEXT))
        .width(Fill)
        .padding([3, 4])
        .clip(true)
        .style(if slot.kind == crate::domain::model::SlotKind::Break {
            theme::chip_break
        } else {
            theme::chip_lesson
        })
        .on_press(Message::OpenSlot(slot.id))
        .into()
}
