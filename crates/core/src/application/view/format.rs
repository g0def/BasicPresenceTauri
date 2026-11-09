//! Durations, clock times and footprints formatted for display.
//!
//! Ported from `shared/format/duration-format.ts` and
//! `entities/commute/lib/commute-format.ts`.

/// UI language. French is the fallback for anything that is not English.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Lang {
    #[default]
    Fr,
    En,
}

impl Lang {
    /// `"en"` (or `"en-…"`) → English, anything else → French.
    pub fn from_code(code: &str) -> Self {
        let code = code.trim().to_ascii_lowercase();
        if code == "en" || code.starts_with("en-") || code.starts_with("en_") {
            Lang::En
        } else {
            Lang::Fr
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Lang::Fr => "fr",
            Lang::En => "en",
        }
    }

    /// Decimal mark of the language (`,` in French, `.` in English).
    pub fn decimal_separator(self) -> &'static str {
        match self {
            Lang::Fr => ",",
            Lang::En => ".",
        }
    }

    /// Digit-group separator (narrow no-break space in French, as Intl does).
    fn group_separator(self) -> &'static str {
        match self {
            Lang::Fr => "\u{202f}",
            Lang::En => ",",
        }
    }
}

/// Compact duration on the h/min convention shared by FR and EN:
/// `45 → "45min"`, `60 → "1h"`, `65 → "1h05"`, `450 → "7h30"`.
/// Negative values clamp to zero.
pub fn format_minutes(minutes: i64) -> String {
    let safe = minutes.max(0);
    let (hours, mins) = (safe / 60, safe % 60);
    match (hours, mins) {
        (0, m) => format!("{m}min"),
        (h, 0) => format!("{h}h"),
        (h, m) => format!("{h}h{m:02}"),
    }
}

/// Lenient clock-time parsing for free typing: `"8"` → 480, and `"8:30"`,
/// `"8h30"`, `"8.30"`, `"8,30"`, `"830"`, `"0830"` all → 510. Returns `None`
/// when the text is not a valid time of day.
pub fn parse_clock_time(raw: &str) -> Option<i64> {
    let s = raw.trim().to_lowercase();
    let digits_end = s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len());
    let (digits, rest) = s.split_at(digits_end);
    let number = |d: &str| d.parse::<i64>().ok();

    let (hours, minutes) = if rest.is_empty() {
        // Digits only: "8", "08", or compact "830" / "0830".
        match digits.len() {
            1 | 2 => (number(digits)?, 0),
            3 | 4 => {
                let (h, m) = digits.split_at(digits.len() - 2);
                (number(h)?, number(m)?)
            }
            _ => return None,
        }
    } else {
        // "<h> <sep> <mm>" with an optional minute part.
        if !(1..=2).contains(&digits.len()) {
            return None;
        }
        let rest = rest.trim_start();
        let mut chars = rest.chars();
        if !matches!(chars.next(), Some(':' | 'h' | '.' | ',')) {
            return None;
        }
        let mins = chars.as_str().trim_start();
        if mins.len() > 2 || !mins.chars().all(|c| c.is_ascii_digit()) {
            return None;
        }
        let m = if mins.is_empty() { 0 } else { number(mins)? };
        (number(digits)?, m)
    };

    (hours <= 23 && minutes <= 59).then_some(hours * 60 + minutes)
}

/// Minutes since midnight → `"HH:MM"`. Values past 1440 wrap to the next day
/// (stacked tasks can cross midnight).
pub fn format_clock_time(minutes: i64) -> String {
    let safe = minutes.rem_euclid(1440);
    format!("{:02}:{:02}", safe / 60, safe % 60)
}

/// A number with at most one decimal, grouped and localized
/// (`1234.56` → `"1 234,6"` in French, `"1,234.6"` in English).
pub fn format_decimal_1(value: f64, lang: Lang) -> String {
    let tenths = (value * 10.0).round();
    let negative = tenths < 0.0;
    let tenths = tenths.abs() as u64;
    let (int_part, frac) = (tenths / 10, tenths % 10);

    let digits = int_part.to_string();
    let mut grouped = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            grouped.push_str(lang.group_separator());
        }
        grouped.push(c);
    }
    let sign = if negative && (int_part > 0 || frac > 0) {
        "-"
    } else {
        ""
    };
    if frac == 0 {
        format!("{sign}{grouped}")
    } else {
        format!("{sign}{grouped}{}{frac}", lang.decimal_separator())
    }
}

/// A footprint in kgCO₂e, e.g. `"4,8 kg"` (fr) / `"4.8 kg"` (en).
pub fn format_co2(kg: f64, lang: Lang) -> String {
    format!("{} kg", format_decimal_1(kg, lang))
}

const MONTHS_FR: [&str; 12] = [
    "janv.", "févr.", "mars", "avr.", "mai", "juin", "juil.", "août", "sept.", "oct.", "nov.",
    "déc.",
];
const MONTHS_EN: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// A timestamp (epoch-ms) as a medium date + short time, in the user's local
/// time given by `utc_offset_minutes`: `"9 nov. 2026, 09:23"` (fr),
/// `"Nov 9, 2026, 9:23 AM"` (en) — the `Intl.DateTimeFormat` medium/short
/// rendering of the former frontend.
pub fn format_datetime(epoch_ms: i64, utc_offset_minutes: i32, lang: Lang) -> String {
    use chrono::{Datelike, Timelike};

    let local_ms = epoch_ms + i64::from(utc_offset_minutes) * 60_000;
    let Some(dt) = chrono::DateTime::from_timestamp_millis(local_ms) else {
        return String::new();
    };
    let dt = dt.naive_utc();
    let month = dt.month0() as usize;
    match lang {
        Lang::Fr => format!(
            "{} {} {}, {:02}:{:02}",
            dt.day(),
            MONTHS_FR[month],
            dt.year(),
            dt.hour(),
            dt.minute()
        ),
        Lang::En => {
            let (pm, hour12) = dt.hour12();
            format!(
                "{} {}, {}, {}:{:02} {}",
                MONTHS_EN[month],
                dt.day(),
                dt.year(),
                hour12,
                dt.minute(),
                if pm { "PM" } else { "AM" }
            )
        }
    }
}

const MONTHS_LONG_FR: [&str; 12] = [
    "janvier",
    "février",
    "mars",
    "avril",
    "mai",
    "juin",
    "juillet",
    "août",
    "septembre",
    "octobre",
    "novembre",
    "décembre",
];
const MONTHS_LONG_EN: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];
const WEEKDAYS_FR: [&str; 7] = [
    "lundi", "mardi", "mercredi", "jeudi", "vendredi", "samedi", "dimanche",
];
const WEEKDAYS_EN: [&str; 7] = [
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
    "Sunday",
];

/// First letter in upper case (`"novembre 2026"` → `"Novembre 2026"`).
pub fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

/// Month heading of the calendar: `"novembre 2026"` / `"November 2026"`.
pub fn format_month_year(date: chrono::NaiveDate, lang: Lang) -> String {
    use chrono::Datelike;
    let names = match lang {
        Lang::Fr => MONTHS_LONG_FR,
        Lang::En => MONTHS_LONG_EN,
    };
    format!("{} {}", names[date.month0() as usize], date.year())
}

/// Short weekday headers, Monday first: `lu ma me je ve sa di` / `Mo Tu …`.
pub fn weekday_initials(lang: Lang) -> [String; 7] {
    let names = match lang {
        Lang::Fr => WEEKDAYS_FR,
        Lang::En => WEEKDAYS_EN,
    };
    names.map(|n| n.chars().take(2).collect())
}

fn ordinal_suffix(day: u32) -> &'static str {
    match (day % 10, day % 100) {
        (_, 11..=13) => "th",
        (1, _) => "st",
        (2, _) => "nd",
        (3, _) => "rd",
        _ => "th",
    }
}

/// Medium date: `"9 novembre 2026"` / `"November 9th, 2026"`.
pub fn format_date(date: chrono::NaiveDate, lang: Lang) -> String {
    use chrono::Datelike;
    match lang {
        Lang::Fr => format!(
            "{} {} {}",
            date.day(),
            MONTHS_LONG_FR[date.month0() as usize],
            date.year()
        ),
        Lang::En => format!(
            "{} {}{}, {}",
            MONTHS_LONG_EN[date.month0() as usize],
            date.day(),
            ordinal_suffix(date.day()),
            date.year()
        ),
    }
}

/// Full date with the weekday: `"lundi 9 novembre 2026"` /
/// `"Monday, November 9th, 2026"`.
pub fn format_long_date(date: chrono::NaiveDate, lang: Lang) -> String {
    use chrono::Datelike;
    let i = date.weekday().num_days_from_monday() as usize;
    match lang {
        Lang::Fr => format!("{} {}", WEEKDAYS_FR[i], format_date(date, lang)),
        Lang::En => format!("{}, {}", WEEKDAYS_EN[i], format_date(date, lang)),
    }
}

/// Remaining time as `m:ss` (the session countdown); negative clamps to `0:00`.
pub fn format_countdown(remaining_ms: i64) -> String {
    let total = remaining_ms.max(0) / 1000;
    format!("{}:{:02}", total / 60, total % 60)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_minutes_follows_the_h_min_convention() {
        assert_eq!(format_minutes(5), "5min");
        assert_eq!(format_minutes(45), "45min");
        assert_eq!(format_minutes(60), "1h");
        assert_eq!(format_minutes(480), "8h");
        assert_eq!(format_minutes(65), "1h05");
        assert_eq!(format_minutes(450), "7h30");
        assert_eq!(format_minutes(-10), "0min");
        assert_eq!(format_minutes(0), "0min");
    }

    #[test]
    fn format_clock_time_pads_and_wraps() {
        assert_eq!(format_clock_time(0), "00:00");
        assert_eq!(format_clock_time(510), "08:30");
        assert_eq!(format_clock_time(1020), "17:00");
        // 23:30 + 1h of tasks ends at 00:30 the next day.
        assert_eq!(format_clock_time(1470), "00:30");
        assert_eq!(format_clock_time(-30), "23:30");
    }

    #[test]
    fn parse_clock_time_accepts_bare_hours() {
        assert_eq!(parse_clock_time("8"), Some(480));
        assert_eq!(parse_clock_time("08"), Some(480));
        assert_eq!(parse_clock_time("0"), Some(0));
        assert_eq!(parse_clock_time("23"), Some(1380));
    }

    #[test]
    fn parse_clock_time_accepts_usual_separators() {
        for s in ["8:30", "8h30", "8H30", "8.30", "8,30", " 8 : 30 "] {
            assert_eq!(parse_clock_time(s), Some(510), "{s:?}");
        }
    }

    #[test]
    fn parse_clock_time_accepts_compact_digits() {
        assert_eq!(parse_clock_time("830"), Some(510));
        assert_eq!(parse_clock_time("0830"), Some(510));
        assert_eq!(parse_clock_time("1745"), Some(1065));
    }

    #[test]
    fn parse_clock_time_trailing_separator_is_a_round_hour() {
        assert_eq!(parse_clock_time("8h"), Some(480));
        assert_eq!(parse_clock_time("8:"), Some(480));
    }

    #[test]
    fn parse_clock_time_rejects_malformed_input() {
        for s in ["", "24", "8:60", "875", "abc", "8:30:00", "123:00", "8:300"] {
            assert_eq!(parse_clock_time(s), None, "{s:?}");
        }
    }

    #[test]
    fn co2_is_localized_with_one_decimal() {
        assert_eq!(format_co2(4.83, Lang::Fr), "4,8 kg");
        assert_eq!(format_co2(4.83, Lang::En), "4.8 kg");
        assert_eq!(format_co2(5.0, Lang::Fr), "5 kg");
        assert_eq!(format_co2(0.04, Lang::Fr), "0 kg");
        assert_eq!(format_co2(1234.56, Lang::Fr), "1\u{202f}234,6 kg");
        assert_eq!(format_co2(1234.56, Lang::En), "1,234.6 kg");
        assert_eq!(format_co2(1_000_000.0, Lang::En), "1,000,000 kg");
    }

    #[test]
    fn datetime_is_localized_and_offset() {
        // 2026-11-09 08:23:00 UTC.
        let ms = 1_794_212_580_000;
        assert_eq!(format_datetime(ms, 0, Lang::Fr), "9 nov. 2026, 08:23");
        assert_eq!(format_datetime(ms, 60, Lang::Fr), "9 nov. 2026, 09:23");
        assert_eq!(format_datetime(ms, 60, Lang::En), "Nov 9, 2026, 9:23 AM");
        assert_eq!(format_datetime(ms, 600, Lang::En), "Nov 9, 2026, 6:23 PM");
        // Offsets can cross the day boundary.
        assert_eq!(format_datetime(ms, -600, Lang::Fr), "8 nov. 2026, 22:23");
    }

    #[test]
    fn countdown_is_minutes_and_padded_seconds() {
        assert_eq!(format_countdown(900_000), "15:00");
        assert_eq!(format_countdown(61_500), "1:01");
        assert_eq!(format_countdown(999), "0:00");
        assert_eq!(format_countdown(-5), "0:00");
    }

    #[test]
    fn calendar_names() {
        let d = chrono::NaiveDate::from_ymd_opt(2026, 11, 9).unwrap();
        assert_eq!(format_month_year(d, Lang::Fr), "novembre 2026");
        assert_eq!(capitalize(&format_month_year(d, Lang::Fr)), "Novembre 2026");
        assert_eq!(format_month_year(d, Lang::En), "November 2026");
        assert_eq!(weekday_initials(Lang::Fr)[0], "lu");
        assert_eq!(weekday_initials(Lang::En)[6], "Su");
        assert_eq!(format_date(d, Lang::Fr), "9 novembre 2026");
        assert_eq!(format_date(d, Lang::En), "November 9th, 2026");
        assert_eq!(format_long_date(d, Lang::Fr), "lundi 9 novembre 2026");
        assert_eq!(format_long_date(d, Lang::En), "Monday, November 9th, 2026");
        let ordinal = |day| {
            format_date(
                chrono::NaiveDate::from_ymd_opt(2026, 1, day).unwrap(),
                Lang::En,
            )
        };
        assert_eq!(ordinal(1), "January 1st, 2026");
        assert_eq!(ordinal(2), "January 2nd, 2026");
        assert_eq!(ordinal(3), "January 3rd, 2026");
        assert_eq!(ordinal(11), "January 11th, 2026");
        assert_eq!(ordinal(22), "January 22nd, 2026");
        assert_eq!(capitalize(""), "");
        assert_eq!(capitalize("été"), "Été");
    }

    #[test]
    fn lang_codes_and_separators() {
        assert_eq!(Lang::from_code("en"), Lang::En);
        assert_eq!(Lang::from_code("en-GB"), Lang::En);
        assert_eq!(Lang::from_code("fr"), Lang::Fr);
        assert_eq!(Lang::from_code("de"), Lang::Fr);
        assert_eq!(Lang::Fr.decimal_separator(), ",");
        assert_eq!(Lang::En.decimal_separator(), ".");
    }
}
