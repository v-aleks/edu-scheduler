use iced::{Point, Rectangle, Size};

use crate::domain::model::{Slot, Weekday};

pub const TIME_GUTTER: f32 = 68.0;
pub const HOUR_HEIGHT: f32 = 64.0;

#[derive(Debug, Clone, Copy)]
pub struct LaidOutSlot {
    pub slot_id: i64,
    pub rect: Rectangle,
}

pub fn visible_range<'a>(slots: impl IntoIterator<Item = &'a Slot>) -> (u16, u16) {
    let mut start = 7 * 60;
    let mut end = 22 * 60;
    let mut any = false;

    for slot in slots {
        any = true;
        start = start.min(slot.start_minute);
        end = end.max(slot.end_minute);
    }

    if !any {
        return (7 * 60, 22 * 60);
    }

    let start = (start / 60) * 60;
    let end = end.div_ceil(60) * 60;
    let end = end.min(24 * 60);

    (start.min(end.saturating_sub(60)), end)
}

pub fn layout(
    bounds: Rectangle,
    slots: &[&Slot],
    start_minute: u16,
    end_minute: u16,
) -> Vec<LaidOutSlot> {
    if slots.is_empty() || bounds.width <= TIME_GUTTER || end_minute <= start_minute {
        return Vec::new();
    }

    let day_width = (bounds.width - TIME_GUTTER) / 7.0;
    let minutes = f32::from(end_minute - start_minute);
    let scale = bounds.height / minutes;

    let mut by_day: [Vec<&Slot>; 7] = std::array::from_fn(|_| Vec::new());

    for slot in slots {
        by_day[slot.weekday.index()].push(slot);
    }

    let mut result = Vec::with_capacity(slots.len());

    for (day_index, mut day_slots) in by_day.into_iter().enumerate() {
        if day_slots.is_empty() {
            continue;
        }

        day_slots.sort_by_key(|slot| (slot.start_minute, slot.end_minute, slot.id));

        let mut lane_ends: Vec<u16> = Vec::new();
        let mut assigned = Vec::with_capacity(day_slots.len());

        for slot in &day_slots {
            let lane = lane_ends.iter().position(|end| *end <= slot.start_minute);

            let lane = match lane {
                Some(lane) => {
                    lane_ends[lane] = slot.end_minute;
                    lane
                }
                None => {
                    lane_ends.push(slot.end_minute);
                    lane_ends.len() - 1
                }
            };

            assigned.push((*slot, lane));
        }

        let lanes = lane_ends.len().max(1);
        let lane_width = day_width / lanes as f32;
        let x0 = TIME_GUTTER + day_index as f32 * day_width;

        for (slot, lane) in assigned {
            let y = bounds.y + (f32::from(slot.start_minute - start_minute) * scale);
            let height = (f32::from(slot.end_minute - slot.start_minute) * scale).max(1.0);

            let rect = Rectangle::new(
                Point::new(x0 + lane as f32 * lane_width + 2.0, y + 1.0),
                Size::new((lane_width - 4.0).max(1.0), (height - 2.0).max(1.0)),
            );

            result.push(LaidOutSlot {
                slot_id: slot.id,
                rect,
            });
        }
    }

    result
}

pub fn weekday_at(bounds: Rectangle, x: f32) -> Option<Weekday> {
    if x < TIME_GUTTER || x > bounds.width {
        return None;
    }

    let day_width = (bounds.width - TIME_GUTTER) / 7.0;

    if day_width <= 0.0 {
        return None;
    }

    let index = ((x - TIME_GUTTER) / day_width).floor() as usize;
    Weekday::ALL.get(index.min(6)).copied()
}

pub fn minute_at(bounds: Rectangle, y: f32, start_minute: u16, end_minute: u16) -> u16 {
    if bounds.height <= 0.0 || end_minute <= start_minute {
        return start_minute;
    }

    let ratio = ((y - bounds.y) / bounds.height).clamp(0.0, 1.0);
    let minute = f32::from(end_minute - start_minute) * ratio;
    let minute = start_minute.saturating_add(minute as u16);
    let snapped = minute - minute % 15;

    snapped.min(end_minute.saturating_sub(15))
}

pub fn y_for_minute(bounds: Rectangle, minute: u16, start_minute: u16, end_minute: u16) -> f32 {
    if end_minute <= start_minute {
        return bounds.y;
    }

    let ratio =
        f32::from(minute.saturating_sub(start_minute)) / f32::from(end_minute - start_minute);

    bounds.y + ratio.clamp(0.0, 1.0) * bounds.height
}

pub fn target_start_minute(
    bounds: Rectangle,
    pointer: Point,
    grab_offset_y: f32,
    duration: u16,
    start_minute: u16,
    end_minute: u16,
) -> u16 {
    let latest = 1439_u16
        .saturating_sub(duration)
        .min(end_minute.saturating_sub(duration))
        .max(start_minute);
    let earliest = start_minute.min(latest);
    let desired_y = pointer.y - grab_offset_y;

    if bounds.height <= 0.0 || end_minute <= start_minute {
        return earliest;
    }

    let ratio = ((desired_y - bounds.y) / bounds.height).clamp(0.0, 1.0);
    let minute = f32::from(end_minute - start_minute) * ratio;
    let minute = start_minute.saturating_add(minute as u16);
    let snapped = minute - minute % 15;

    snapped.clamp(earliest, latest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::SlotKind;

    fn slot(id: i64, weekday: Weekday, start: u16, end: u16) -> Slot {
        Slot {
            id,
            weekday,
            start_minute: start,
            end_minute: end,
            kind: SlotKind::Lesson,
            title: String::new(),
            description: String::new(),
            teacher_ids: Vec::new(),
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    #[test]
    fn expands_visible_range_to_slots() {
        let slots = [
            slot(1, Weekday::Monday, 6 * 60 + 30, 8 * 60),
            slot(2, Weekday::Tuesday, 20 * 60, 23 * 60 + 30),
        ];
        let refs: Vec<&Slot> = slots.iter().collect();

        assert_eq!(visible_range(refs), (6 * 60, 24 * 60));
        assert_eq!(visible_range(std::iter::empty()), (7 * 60, 22 * 60));
    }

    #[test]
    fn overlapping_slots_use_different_lanes() {
        let slots = [
            slot(1, Weekday::Monday, 540, 630),
            slot(2, Weekday::Monday, 600, 690),
            slot(3, Weekday::Monday, 700, 750),
        ];
        let refs: Vec<&Slot> = slots.iter().collect();
        let bounds = Rectangle::new(Point::ORIGIN, Size::new(768.0, 600.0));
        let layout = layout(bounds, &refs, 7 * 60, 22 * 60);

        assert_eq!(layout.len(), 3);
        assert_ne!(layout[0].rect.x, layout[1].rect.x);
        assert_eq!(layout[2].rect.x, layout[0].rect.x);
        assert!(layout[0].rect.x >= TIME_GUTTER);
    }

    #[test]
    fn maps_coordinates_to_weekday_and_minute() {
        let bounds = Rectangle::new(Point::ORIGIN, Size::new(768.0, 900.0));
        let day_width = (bounds.width - TIME_GUTTER) / 7.0;

        assert_eq!(weekday_at(bounds, 10.0), None);
        assert_eq!(
            weekday_at(bounds, TIME_GUTTER + day_width * 2.5),
            Some(Weekday::Wednesday)
        );
        assert_eq!(minute_at(bounds, 450.0, 7 * 60, 22 * 60), 7 * 60 + 450);
    }

    #[test]
    fn maps_drag_position_to_snapped_start() {
        let bounds = Rectangle::new(Point::ORIGIN, Size::new(768.0, 900.0));

        assert_eq!(
            target_start_minute(bounds, Point::new(300.0, 430.0), 10.0, 90, 7 * 60, 22 * 60),
            7 * 60 + 420
        );

        assert_eq!(
            target_start_minute(bounds, Point::new(300.0, -100.0), 10.0, 90, 7 * 60, 22 * 60),
            7 * 60
        );
    }

    #[test]
    fn y_for_minute_reverses_position() {
        let bounds = Rectangle::new(Point::ORIGIN, Size::new(768.0, 900.0));
        let y = y_for_minute(bounds, 7 * 60 + 450, 7 * 60, 22 * 60);
        assert!((y - 450.0).abs() < 0.01);
    }
}
