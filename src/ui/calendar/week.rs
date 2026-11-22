//! Недельный календарь на canvas.

use chrono::{Datelike, Local, NaiveDate};
use iced::alignment::{self, Alignment};
use iced::mouse;
use iced::widget::canvas::{self, Frame, Geometry, Path, Text};
use iced::widget::{canvas as canvas_widget, container, row, scrollable, space, text};
use iced::{Border, Color, Element, Fill, Point, Rectangle, Renderer, Size, Theme};

use crate::app::{App, Message};
use crate::domain::model::{Slot, Teacher, Weekday};
use crate::domain::validation::format_time;
use crate::ui::calendar::week_layout;
use crate::ui::theme;

pub fn view(app: &App) -> Element<'_, Message> {
    let start_minute = visible_range(app).0;
    let end_minute = visible_range(app).1;
    let week = app.week();
    let today = Local::now().date_naive();

    let header = week_header(&week, today);
    let height = (f32::from(end_minute - start_minute) / 60.0) * week_layout::HOUR_HEIGHT;

    let canvas = canvas_widget(WeekCanvas {
        week,
        today,
        slots: &app.slots,
        teachers: &app.teachers,
        filter: app.filter,
        start_minute,
        end_minute,
    })
    .width(Fill)
    .height(height);

    let body =
        scrollable(canvas)
            .height(Fill)
            .width(Fill)
            .direction(scrollable::Direction::Vertical(
                scrollable::Scrollbar::default(),
            ));

    container(iced::widget::column![header, body])
        .width(Fill)
        .height(Fill)
        .style(theme::card)
        .padding(0)
        .into()
}

fn visible_range(app: &App) -> (u16, u16) {
    let filtered = app.slots.iter().filter(|slot| match app.filter {
        Some(teacher_id) => slot.has_teacher(teacher_id),
        None => true,
    });

    week_layout::visible_range(filtered)
}

fn week_header(week: &[NaiveDate; 7], today: NaiveDate) -> Element<'static, Message> {
    let mut days = row![space().width(week_layout::TIME_GUTTER)]
        .width(Fill)
        .height(60);

    for date in week {
        let is_today = *date == today;
        let is_weekend = matches!(date.weekday(), chrono::Weekday::Sat | chrono::Weekday::Sun);

        let label = format!("{} {}", Weekday::from_date(*date).short_label(), date.day());

        let content = container(
            iced::widget::column![
                text(label)
                    .size(13)
                    .font(theme::semibold())
                    .color(if is_today { theme::ACCENT } else { theme::TEXT }),
                text(format!("{:02}.{:02}", date.day(), date.month()))
                    .size(11)
                    .color(theme::MUTED),
            ]
            .spacing(2)
            .align_x(Alignment::Center),
        )
        .width(Fill)
        .height(Fill)
        .padding([8, 4])
        .align_x(Alignment::Center)
        .style(move |_theme| container::Style {
            background: Some(
                if is_today {
                    theme::ACCENT_SOFT
                } else if is_weekend {
                    theme::WEEKEND_BG
                } else {
                    theme::SURFACE
                }
                .into(),
            ),
            border: Border {
                color: theme::BORDER,
                width: 0.0,
                radius: 0.0.into(),
            },
            ..container::Style::default()
        });

        days = days.push(content);
    }

    container(days)
        .width(Fill)
        .style(|_theme| container::Style {
            background: Some(theme::SURFACE.into()),
            border: Border {
                color: theme::BORDER,
                width: 0.0,
                radius: 0.0.into(),
            },
            ..container::Style::default()
        })
        .into()
}

struct WeekCanvas<'a> {
    week: [NaiveDate; 7],
    today: NaiveDate,
    slots: &'a [Slot],
    teachers: &'a [Teacher],
    filter: Option<i64>,
    start_minute: u16,
    end_minute: u16,
}

impl WeekCanvas<'_> {
    fn visible_slots(&self) -> Vec<&Slot> {
        self.slots
            .iter()
            .filter(|slot| match self.filter {
                Some(teacher_id) => slot.has_teacher(teacher_id),
                None => true,
            })
            .collect()
    }

    fn teacher_names(&self, slot: &Slot) -> String {
        let names: Vec<&str> = slot
            .teacher_ids
            .iter()
            .filter_map(|teacher_id| {
                self.teachers
                    .iter()
                    .find(|teacher| teacher.id == *teacher_id)
                    .map(Teacher::display_name)
            })
            .collect();

        if names.is_empty() {
            "Без преподавателя".to_string()
        } else {
            names.join(", ")
        }
    }

    fn slot_rects(&self, bounds: Rectangle) -> Vec<week_layout::LaidOutSlot> {
        let visible = self.visible_slots();

        week_layout::layout(
            Rectangle::new(Point::ORIGIN, bounds.size()),
            &visible,
            self.start_minute,
            self.end_minute,
        )
    }

    fn hit_test(&self, bounds: Rectangle, position: Point) -> Option<i64> {
        self.slot_rects(bounds)
            .iter()
            .rev()
            .find(|laid_out| laid_out.rect.contains(position))
            .map(|laid_out| laid_out.slot_id)
    }
}

#[derive(Default)]
struct WeekCanvasState {
    drag: Option<DragState>,
}

struct DragState {
    slot_id: i64,
    duration: u16,
    origin_start: u16,
    origin: Point,
    pointer: Point,
    moved: bool,
}

impl DragState {
    fn start_minute(&self) -> u16 {
        week_layout::start_minute_from_delta(
            self.origin_start,
            self.pointer.y - self.origin.y,
            self.duration,
        )
    }
}

impl canvas::Program<Message> for WeekCanvas<'_> {
    type State = WeekCanvasState;

    fn update(
        &self,
        state: &mut Self::State,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        if let canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) = event
            && let Some(drag) = state.drag.take()
        {
            if let Some(position) = cursor.position_in(bounds) {
                if drag.moved {
                    let weekday = week_layout::weekday_at(bounds, position.x)?;
                    let start_minute = week_layout::start_minute_from_delta(
                        drag.origin_start,
                        position.y - drag.origin.y,
                        drag.duration,
                    );

                    return Some(
                        canvas::Action::publish(Message::SlotMoved {
                            id: drag.slot_id,
                            weekday,
                            start_minute,
                        })
                        .and_capture(),
                    );
                }

                return Some(
                    canvas::Action::publish(Message::WeekSlotClicked(drag.slot_id)).and_capture(),
                );
            }

            return Some(canvas::Action::request_redraw().and_capture());
        }

        if let canvas::Event::Mouse(mouse::Event::CursorLeft) = event
            && state.drag.take().is_some()
        {
            return Some(canvas::Action::request_redraw().and_capture());
        }

        let position = cursor.position_in(bounds)?;

        let canvas::Event::Mouse(event) = event else {
            return None;
        };

        match event {
            mouse::Event::ButtonPressed(mouse::Button::Left) => {
                if let Some(slot_id) = self.hit_test(bounds, position) {
                    let slot = self.slots.iter().find(|slot| slot.id == slot_id)?;

                    state.drag = Some(DragState {
                        slot_id,
                        duration: slot.duration_minutes(),
                        origin_start: slot.start_minute,
                        origin: position,
                        pointer: position,
                        moved: false,
                    });

                    return Some(canvas::Action::request_redraw().and_capture());
                }

                let weekday = week_layout::weekday_at(bounds, position.x)?;
                let minute =
                    week_layout::minute_at(bounds, position.y, self.start_minute, self.end_minute);

                Some(
                    canvas::Action::publish(Message::WeekEmptyClicked { weekday, minute })
                        .and_capture(),
                )
            }
            mouse::Event::CursorMoved {
                position: window_position,
            } => {
                let drag = state.drag.as_mut()?;
                let local = Point::new(window_position.x - bounds.x, window_position.y - bounds.y);
                drag.pointer = local;

                if !drag.moved {
                    let distance = (local.x - drag.origin.x)
                        .abs()
                        .max((local.y - drag.origin.y).abs());

                    drag.moved = distance >= 4.0;
                }

                Some(canvas::Action::request_redraw().and_capture())
            }
            _ => None,
        }
    }

    fn mouse_interaction(
        &self,
        state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if state.drag.is_some() {
            return mouse::Interaction::Grabbing;
        }

        let Some(position) = cursor.position_in(bounds) else {
            return mouse::Interaction::default();
        };

        let over_slot = self
            .hit_test(bounds, position)
            .is_some_and(|slot_id| self.slots.iter().any(|slot| slot.id == slot_id));

        if over_slot {
            mouse::Interaction::Grab
        } else {
            mouse::Interaction::Crosshair
        }
    }

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());

        frame.fill_rectangle(Point::ORIGIN, bounds.size(), theme::SURFACE);

        let day_width = (bounds.width - week_layout::TIME_GUTTER) / 7.0;

        for (index, date) in self.week.iter().enumerate() {
            let x = week_layout::TIME_GUTTER + index as f32 * day_width;
            let background = if *date == self.today {
                theme::TODAY_BG
            } else if matches!(date.weekday(), chrono::Weekday::Sat | chrono::Weekday::Sun) {
                theme::WEEKEND_BG
            } else {
                theme::SURFACE
            };

            frame.fill_rectangle(
                Point::new(x, 0.0),
                Size::new(day_width, bounds.height),
                background,
            );

            frame.stroke(
                &Path::line(Point::new(x, 0.0), Point::new(x, bounds.height)),
                canvas::Stroke::default()
                    .with_color(theme::BORDER)
                    .with_width(1.0),
            );
        }

        let total_minutes = f32::from(self.end_minute - self.start_minute);
        let scale = bounds.height / total_minutes;

        let first_hour = self.start_minute.div_ceil(60);
        let last_hour = self.end_minute / 60;

        for hour in first_hour..=last_hour {
            let minute = hour * 60;

            if minute > self.end_minute {
                break;
            }

            let y = f32::from(minute - self.start_minute) * scale;

            frame.stroke(
                &Path::line(
                    Point::new(week_layout::TIME_GUTTER, y),
                    Point::new(bounds.width, y),
                ),
                canvas::Stroke::default()
                    .with_color(theme::BORDER)
                    .with_width(1.0),
            );

            frame.fill_text(Text {
                content: format_time(minute),
                position: Point::new(week_layout::TIME_GUTTER - 10.0, y + 2.0),
                color: theme::MUTED,
                size: 11.0.into(),
                font: theme::font(),
                align_x: alignment::Horizontal::Right.into(),
                align_y: alignment::Vertical::Top,
                ..Text::default()
            });
        }

        let visible = self.visible_slots();

        if visible.is_empty() {
            frame.fill_text(Text {
                content: "Пока нет слотов. Нажмите на сетку, чтобы создать.".to_string(),
                position: Point::new(
                    week_layout::TIME_GUTTER + (bounds.width - week_layout::TIME_GUTTER) / 2.0,
                    bounds.height / 2.0,
                ),
                color: theme::MUTED,
                size: 14.0.into(),
                font: theme::font(),
                align_x: alignment::Horizontal::Center.into(),
                align_y: alignment::Vertical::Center,
                ..Text::default()
            });

            return vec![frame.into_geometry()];
        }

        for laid_out in self.slot_rects(bounds) {
            let Some(slot) = self.slots.iter().find(|slot| slot.id == laid_out.slot_id) else {
                continue;
            };

            if state
                .drag
                .as_ref()
                .is_some_and(|drag| drag.moved && drag.slot_id == slot.id)
            {
                continue;
            }

            draw_slot(&mut frame, slot, laid_out.rect, self);
        }

        if let Some(drag) = &state.drag
            && drag.moved
            && let Some(weekday) = week_layout::weekday_at(bounds, drag.pointer.x)
            && let Some(slot) = self.slots.iter().find(|slot| slot.id == drag.slot_id)
        {
            let start_minute = drag.start_minute();
            draw_drag_preview(
                &mut frame,
                slot,
                weekday,
                start_minute,
                drag.duration,
                bounds,
                self,
            );
        }

        vec![frame.into_geometry()]
    }
}

fn draw_drag_preview(
    frame: &mut Frame,
    slot: &Slot,
    weekday: Weekday,
    start_minute: u16,
    duration: u16,
    bounds: Rectangle,
    canvas: &WeekCanvas<'_>,
) {
    let day_width = (bounds.width - week_layout::TIME_GUTTER) / 7.0;
    let x = week_layout::TIME_GUTTER + weekday.index() as f32 * day_width + 2.0;
    let y = week_layout::y_for_minute(
        Rectangle::new(Point::ORIGIN, bounds.size()),
        start_minute,
        canvas.start_minute,
        canvas.end_minute,
    );
    let height = (f32::from(duration) / f32::from(canvas.end_minute - canvas.start_minute)
        * bounds.height)
        .max(18.0);
    let rect = Rectangle::new(
        Point::new(x, y + 1.0),
        Size::new((day_width - 4.0).max(1.0), (height - 2.0).max(1.0)),
    );
    let accent = Color {
        a: 0.16,
        ..theme::ACCENT
    };

    frame.fill(
        &Path::rounded_rectangle(rect.position(), rect.size(), 7.0.into()),
        accent,
    );
    frame.stroke(
        &Path::rounded_rectangle(rect.position(), rect.size(), 7.0.into()),
        canvas::Stroke::default()
            .with_color(theme::ACCENT)
            .with_width(2.0),
    );

    frame.with_clip(rect, |frame| {
        frame.fill_text(Text {
            content: format!(
                "{}–{}",
                format_time(start_minute),
                format_time(start_minute.saturating_add(duration))
            ),
            position: Point::new(rect.x + 8.0, rect.y + 5.0),
            max_width: (rect.width - 16.0).max(1.0),
            color: theme::ACCENT,
            size: 10.0.into(),
            font: theme::semibold(),
            ..Text::default()
        });

        if rect.height >= 34.0 {
            frame.fill_text(Text {
                content: slot.display_title().to_string(),
                position: Point::new(rect.x + 8.0, rect.y + 20.0),
                max_width: (rect.width - 16.0).max(1.0),
                color: theme::TEXT,
                size: 12.0.into(),
                font: theme::semibold(),
                ..Text::default()
            });
        }
    });
}

fn draw_slot(frame: &mut Frame, slot: &Slot, rect: Rectangle, canvas: &WeekCanvas<'_>) {
    let is_break = slot.kind == crate::domain::model::SlotKind::Break;
    let radius = 7.0;

    let background = if is_break {
        theme::BREAK_BG
    } else {
        theme::SURFACE
    };
    let border = if is_break {
        theme::BREAK_BORDER
    } else {
        theme::BORDER_STRONG
    };

    frame.fill(
        &Path::rounded_rectangle(rect.position(), rect.size(), radius.into()),
        background,
    );

    frame.stroke(
        &Path::rounded_rectangle(rect.position(), rect.size(), radius.into()),
        canvas::Stroke::default().with_color(border).with_width(1.0),
    );

    let accent_colors: Vec<iced::Color> = slot
        .teacher_ids
        .iter()
        .filter_map(|teacher_id| {
            canvas
                .teachers
                .iter()
                .find(|teacher| teacher.id == *teacher_id)
                .map(|teacher| theme::teacher_color(&teacher.color))
        })
        .collect();

    let accent_width = 4.0_f32.min(rect.width);

    if accent_colors.is_empty() {
        frame.fill_rectangle(
            Point::new(rect.x, rect.y + radius),
            Size::new(accent_width, (rect.height - 2.0 * radius).max(1.0)),
            if is_break {
                theme::BREAK_BORDER
            } else {
                theme::ACCENT
            },
        );
    } else {
        let segment_height = (rect.height - 2.0 * radius) / accent_colors.len() as f32;

        for (index, color) in accent_colors.iter().enumerate() {
            frame.fill_rectangle(
                Point::new(rect.x, rect.y + radius + index as f32 * segment_height),
                Size::new(accent_width, segment_height.max(1.0)),
                *color,
            );
        }
    }

    let padding = 9.0;
    let text_x = rect.x + padding + accent_width;
    let text_width = (rect.width - padding * 2.0 - accent_width).max(1.0);
    let time = format!(
        "{}–{}",
        format_time(slot.start_minute),
        format_time(slot.end_minute)
    );
    let title = slot.display_title().to_string();
    let names = canvas.teacher_names(slot);
    let font = theme::font();
    let bold = theme::semibold();

    frame.with_clip(rect, |frame| {
        if rect.height < 30.0 {
            frame.fill_text(Text {
                content: format!("{time}  {title}"),
                position: Point::new(text_x, rect.y + 5.0),
                max_width: text_width,
                color: theme::TEXT,
                size: 10.0.into(),
                font: bold,
                ..Text::default()
            });
            return;
        }

        frame.fill_text(Text {
            content: time,
            position: Point::new(text_x, rect.y + 5.0),
            max_width: text_width,
            color: if is_break {
                theme::MUTED
            } else {
                theme::ACCENT
            },
            size: 10.0.into(),
            font: theme::semibold(),
            ..Text::default()
        });

        frame.fill_text(Text {
            content: title,
            position: Point::new(text_x, rect.y + 20.0),
            max_width: text_width,
            color: theme::TEXT,
            size: 12.0.into(),
            font: bold,
            ..Text::default()
        });

        if rect.height >= 56.0 {
            frame.fill_text(Text {
                content: names,
                position: Point::new(text_x, rect.y + 38.0),
                max_width: text_width,
                color: theme::MUTED,
                size: 10.0.into(),
                font,
                ..Text::default()
            });
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::SlotKind;
    use chrono::NaiveDate;
    use iced::Size;
    use iced::widget::canvas::Program;

    fn test_slot() -> Slot {
        Slot {
            id: 1,
            weekday: Weekday::Monday,
            start_minute: 9 * 60,
            end_minute: 10 * 60 + 30,
            kind: SlotKind::Lesson,
            title: "Математика".to_string(),
            description: String::new(),
            teacher_ids: Vec::new(),
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    fn test_canvas(slots: &[Slot]) -> WeekCanvas<'_> {
        let week = std::array::from_fn(|index| {
            NaiveDate::from_ymd_opt(2026, 11, 16).unwrap() + chrono::Duration::days(index as i64)
        });

        WeekCanvas {
            week,
            today: NaiveDate::from_ymd_opt(2026, 11, 16).unwrap(),
            slots,
            teachers: &[],
            filter: None,
            start_minute: 7 * 60,
            end_minute: 22 * 60,
        }
    }

    fn press() -> canvas::Event {
        canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
    }

    fn release() -> canvas::Event {
        canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
    }

    #[test]
    fn click_on_slot_is_published_on_release() {
        let slots = vec![test_slot()];
        let canvas = test_canvas(&slots);
        let bounds = Rectangle::new(Point::ORIGIN, Size::new(768.0, 900.0));
        let rect = canvas.slot_rects(bounds)[0].rect;
        let point = rect.center();
        let mut state = WeekCanvasState::default();

        let action = canvas.update(
            &mut state,
            &press(),
            bounds,
            mouse::Cursor::Available(point),
        );
        assert!(action.is_some());
        assert!(state.drag.is_some());

        let action = canvas
            .update(
                &mut state,
                &release(),
                bounds,
                mouse::Cursor::Available(point),
            )
            .unwrap();
        let (message, _, _) = action.into_inner();
        assert!(matches!(message, Some(Message::WeekSlotClicked(1))));
        assert!(state.drag.is_none());
    }

    #[test]
    fn dragging_slot_vertically_changes_time() {
        let slots = vec![test_slot()];
        let canvas = test_canvas(&slots);
        let bounds = Rectangle::new(Point::ORIGIN, Size::new(768.0, 900.0));
        let rect = canvas.slot_rects(bounds)[0].rect;
        let start = rect.center();
        let mut state = WeekCanvasState::default();

        canvas.update(
            &mut state,
            &press(),
            bounds,
            mouse::Cursor::Available(start),
        );

        let target = Point::new(start.x, start.y + 64.0);
        canvas.update(
            &mut state,
            &canvas::Event::Mouse(mouse::Event::CursorMoved { position: target }),
            bounds,
            mouse::Cursor::Available(target),
        );

        let action = canvas
            .update(
                &mut state,
                &release(),
                bounds,
                mouse::Cursor::Available(target),
            )
            .unwrap();
        let (message, _, _) = action.into_inner();

        match message {
            Some(Message::SlotMoved {
                id,
                weekday,
                start_minute,
            }) => {
                assert_eq!(id, 1);
                assert_eq!(weekday, Weekday::Monday);
                assert_eq!(start_minute, 10 * 60);
            }
            other => panic!("unexpected message: {other:?}"),
        }
    }

    #[test]
    fn dragging_slot_publishes_new_day_and_time() {
        let slots = vec![test_slot()];
        let canvas = test_canvas(&slots);
        let bounds = Rectangle::new(Point::ORIGIN, Size::new(768.0, 900.0));
        let rect = canvas.slot_rects(bounds)[0].rect;
        let start = rect.center();
        let mut state = WeekCanvasState::default();

        canvas.update(
            &mut state,
            &press(),
            bounds,
            mouse::Cursor::Available(start),
        );
        let day_width = (bounds.width - week_layout::TIME_GUTTER) / 7.0;
        let target = Point::new(week_layout::TIME_GUTTER + day_width * 1.5, start.y);
        canvas.update(
            &mut state,
            &canvas::Event::Mouse(mouse::Event::CursorMoved { position: target }),
            bounds,
            mouse::Cursor::Available(target),
        );
        assert!(state.drag.as_ref().is_some_and(|drag| drag.moved));

        let action = canvas
            .update(
                &mut state,
                &release(),
                bounds,
                mouse::Cursor::Available(target),
            )
            .unwrap();
        let (message, _, _) = action.into_inner();

        match message {
            Some(Message::SlotMoved {
                id,
                weekday,
                start_minute,
            }) => {
                assert_eq!(id, 1);
                assert_eq!(weekday, Weekday::Tuesday);
                assert_eq!(start_minute, 9 * 60);
            }
            other => panic!("unexpected message: {other:?}"),
        }
    }
}
