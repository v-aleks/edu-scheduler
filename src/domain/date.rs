use chrono::{Datelike, Duration, NaiveDate};

const MONTHS_NOMINATIVE: [&str; 12] = [
    "Январь",
    "Февраль",
    "Март",
    "Апрель",
    "Май",
    "Июнь",
    "Июль",
    "Август",
    "Сентябрь",
    "Октябрь",
    "Ноябрь",
    "Декабрь",
];

const MONTHS_GENITIVE: [&str; 12] = [
    "января",
    "февраля",
    "марта",
    "апреля",
    "мая",
    "июня",
    "июля",
    "августа",
    "сентября",
    "октября",
    "ноября",
    "декабря",
];

pub fn month_name_nominative(month: u32) -> &'static str {
    MONTHS_NOMINATIVE
        .get(month.saturating_sub(1) as usize)
        .copied()
        .unwrap_or("")
}

pub fn month_name_genitive(month: u32) -> &'static str {
    MONTHS_GENITIVE
        .get(month.saturating_sub(1) as usize)
        .copied()
        .unwrap_or("")
}

pub fn week_start(date: NaiveDate) -> NaiveDate {
    date - Duration::days(i64::from(date.weekday().num_days_from_monday()))
}

pub fn week_dates(date: NaiveDate) -> [NaiveDate; 7] {
    let start = week_start(date);

    std::array::from_fn(|index| start + Duration::days(index as i64))
}

pub fn month_grid(anchor: NaiveDate) -> [[NaiveDate; 7]; 6] {
    let first = NaiveDate::from_ymd_opt(anchor.year(), anchor.month(), 1).unwrap_or(anchor);
    let start = week_start(first);

    std::array::from_fn(|week| {
        std::array::from_fn(|day| start + Duration::days((week * 7 + day) as i64))
    })
}

pub fn shift_month(date: NaiveDate, delta: i32) -> NaiveDate {
    let zero_based = date.year() * 12 + date.month0() as i32 + delta;
    let year = zero_based.div_euclid(12);
    let month = zero_based.rem_euclid(12) as u32 + 1;
    let last_day = days_in_month(year, month);

    NaiveDate::from_ymd_opt(year, month, date.day().min(last_day)).unwrap_or(date)
}

fn days_in_month(year: i32, month: u32) -> u32 {
    let (next_year, next_month) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };

    let first_next =
        NaiveDate::from_ymd_opt(next_year, next_month, 1).expect("valid first day of month");

    (first_next - Duration::days(1)).day()
}

pub fn format_month(date: NaiveDate) -> String {
    format!("{} {}", month_name_nominative(date.month()), date.year())
}

pub fn format_week_range(start: NaiveDate) -> String {
    let end = start + Duration::days(6);
    let start_month = month_name_genitive(start.month());

    if start.year() == end.year() && start.month() == end.month() {
        format!(
            "{} – {} {} {}",
            start.day(),
            end.day(),
            start_month,
            start.year()
        )
    } else if start.year() == end.year() {
        format!(
            "{} {} – {} {} {}",
            start.day(),
            start_month,
            end.day(),
            month_name_genitive(end.month()),
            start.year()
        )
    } else {
        format!(
            "{} {} {} – {} {} {}",
            start.day(),
            start_month,
            start.year(),
            end.day(),
            month_name_genitive(end.month()),
            end.year()
        )
    }
}

pub fn format_day_short(date: NaiveDate) -> String {
    format!("{:02}.{:02}", date.day(), date.month())
}

pub fn format_day_long(date: NaiveDate) -> String {
    format!(
        "{} {} {}",
        date.day(),
        month_name_genitive(date.month()),
        date.year()
    )
}

pub fn format_duration(minutes: u16) -> String {
    let hours = minutes / 60;
    let rest = minutes % 60;

    match (hours, rest) {
        (0, minutes) => format!("{minutes} мин"),
        (hours, 0) => format!("{hours} ч"),
        (hours, minutes) => format!("{hours} ч {minutes} мин"),
    }
}

pub fn same_month(a: NaiveDate, b: NaiveDate) -> bool {
    a.year() == b.year() && a.month() == b.month()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    #[test]
    fn week_starts_on_monday() {
        let wednesday = date(2026, 11, 18);
        assert_eq!(week_start(wednesday), date(2026, 11, 16));

        let dates = week_dates(wednesday);
        assert_eq!(dates[0], date(2026, 11, 16));
        assert_eq!(dates[6], date(2026, 11, 22));
    }

    #[test]
    fn month_grid_has_six_full_weeks() {
        let grid = month_grid(date(2026, 11, 18));
        assert_eq!(grid[0][0], date(2026, 10, 26));
        assert_eq!(grid[5][6], date(2026, 12, 6));

        let leap_february = month_grid(date(2024, 2, 10));
        assert_eq!(leap_february[0][0], date(2024, 1, 29));
        assert!(
            leap_february
                .iter()
                .flatten()
                .any(|day| *day == date(2024, 2, 29))
        );
    }

    #[test]
    fn formats_week_ranges() {
        assert_eq!(format_week_range(date(2026, 11, 16)), "16 – 22 ноября 2026");

        assert_eq!(
            format_week_range(date(2026, 11, 30)),
            "30 ноября – 6 декабря 2026"
        );

        assert_eq!(
            format_week_range(date(2025, 12, 29)),
            "29 декабря 2025 – 4 января 2026"
        );
    }

    #[test]
    fn shifts_months_and_clamps_days() {
        assert_eq!(shift_month(date(2026, 1, 31), 1), date(2026, 2, 28));
        assert_eq!(shift_month(date(2024, 2, 29), 1), date(2024, 3, 29));
        assert_eq!(shift_month(date(2026, 1, 15), -1), date(2025, 12, 15));
        assert_eq!(shift_month(date(2026, 12, 15), 1), date(2027, 1, 15));
    }

    #[test]
    fn formats_month_and_duration() {
        assert_eq!(format_month(date(2026, 11, 18)), "Ноябрь 2026");
        assert_eq!(format_duration(10), "10 мин");
        assert_eq!(format_duration(60), "1 ч");
        assert_eq!(format_duration(90), "1 ч 30 мин");
    }
}
