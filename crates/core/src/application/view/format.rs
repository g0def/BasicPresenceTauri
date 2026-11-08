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
    fn lang_codes_and_separators() {
        assert_eq!(Lang::from_code("en"), Lang::En);
        assert_eq!(Lang::from_code("en-GB"), Lang::En);
        assert_eq!(Lang::from_code("fr"), Lang::Fr);
        assert_eq!(Lang::from_code("de"), Lang::Fr);
        assert_eq!(Lang::Fr.decimal_separator(), ",");
        assert_eq!(Lang::En.decimal_separator(), ".");
    }
}
