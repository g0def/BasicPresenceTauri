//! Period statistics for the charts and the month summary (ported from
//! `features/stats/domain/aggregate.ts` and the header totals of
//! `routes/_authenticated.tsx`).
//!
//! A presence's `day` is epoch-ms at UTC midnight: its calendar date is the
//! UTC date, whatever the machine's time zone.

use chrono::{Datelike, Days, Months, NaiveDate};

use crate::application::dto::presence_dto::PresenceDto;
use crate::domain::entities::presence::PresenceType;

const MS_PER_DAY: i64 = 86_400_000;

/// Time resolution of the period charts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Granularity {
    Month,
    Week,
}

/// One time bucket (a calendar month or an ISO week) with its summed metrics.
#[derive(Debug, Clone, PartialEq)]
pub struct PeriodBucket {
    /// Chronologically sortable identity: `year*100 + month` for months,
    /// `iso_year*100 + iso_week` for weeks.
    pub key: i32,
    /// First day of the bucket (the UI derives its label from it).
    pub start: NaiveDate,
    /// Sum of the non-null `co2_kg` (vacation/holiday days carry none).
    pub co2_kg: f64,
    /// Sum of `work_minutes` over every day in the bucket.
    pub work_minutes: i64,
    /// Days that landed in this bucket (0 for gap-filled buckets).
    pub day_count: u32,
}

/// Calendar date of a presence day (epoch-ms at UTC midnight).
pub fn civil_date(day_ms: i64) -> NaiveDate {
    let epoch = NaiveDate::from_ymd_opt(1970, 1, 1).expect("valid epoch");
    let days = day_ms.div_euclid(MS_PER_DAY);
    if days >= 0 {
        epoch.checked_add_days(Days::new(days as u64))
    } else {
        epoch.checked_sub_days(Days::new(days.unsigned_abs()))
    }
    .unwrap_or(epoch)
}

/// Epoch-ms at UTC midnight of a calendar date (inverse of [`civil_date`]).
pub fn day_ms(date: NaiveDate) -> i64 {
    let epoch = NaiveDate::from_ymd_opt(1970, 1, 1).expect("valid epoch");
    (date - epoch).num_days() * MS_PER_DAY
}

/// The bucket key of a date. Keys sort chronologically, so a
/// `start <= key <= end` test selects a contiguous range of periods.
///
/// Weeks use the ISO week-year (not the calendar year): the week holding
/// January 1st can belong to the previous year, and vice versa.
pub fn period_key(date: NaiveDate, granularity: Granularity) -> i32 {
    match granularity {
        Granularity::Month => date.year() * 100 + date.month() as i32,
        Granularity::Week => {
            let w = date.iso_week();
            w.year() * 100 + w.week() as i32
        }
    }
}

/// First day of the period holding `date`.
pub fn start_of_period(date: NaiveDate, granularity: Granularity) -> NaiveDate {
    match granularity {
        Granularity::Month => date.with_day(1).unwrap_or(date),
        Granularity::Week => date
            .checked_sub_days(Days::new(date.weekday().num_days_from_monday().into()))
            .unwrap_or(date),
    }
}

fn next_period(start: NaiveDate, granularity: Granularity) -> Option<NaiveDate> {
    match granularity {
        Granularity::Month => start.checked_add_months(Months::new(1)),
        Granularity::Week => start.checked_add_days(Days::new(7)),
    }
}

fn empty_bucket(key: i32, start: NaiveDate) -> PeriodBucket {
    PeriodBucket {
        key,
        start,
        co2_kg: 0.0,
        work_minutes: 0,
        day_count: 0,
    }
}

/// Group presences into month or ISO-week buckets, sorted ascending and
/// **gap-filled**: empty periods between the first and the last one are
/// inserted as zero buckets, so the bars read as a continuous timeline.
///
/// `until` (typically today) extends the timeline forward so the current
/// period stays on the chart even when its days are not encoded yet.
pub fn bucket_presences(
    presences: &[PresenceDto],
    granularity: Granularity,
    until: Option<NaiveDate>,
) -> Vec<PeriodBucket> {
    let mut buckets: std::collections::BTreeMap<i32, PeriodBucket> = Default::default();
    let mut range: Option<(NaiveDate, NaiveDate)> = None;

    for p in presences {
        let date = civil_date(p.day);
        let key = period_key(date, granularity);
        let bucket = buckets
            .entry(key)
            .or_insert_with(|| empty_bucket(key, start_of_period(date, granularity)));
        if let Some(co2) = p.co2_kg {
            bucket.co2_kg += co2;
        }
        bucket.work_minutes += p.work_minutes;
        bucket.day_count += 1;
        range = Some(match range {
            None => (date, date),
            Some((min, max)) => (min.min(date), max.max(date)),
        });
    }

    let Some((min, max)) = range else {
        return Vec::new();
    };
    let mut end = start_of_period(max, granularity);
    if let Some(until) = until {
        end = end.max(start_of_period(until, granularity));
    }

    let mut filled = Vec::new();
    let mut cursor = Some(start_of_period(min, granularity));
    // `guard`: safety net against a malformed range looping forever.
    let mut guard = 0;
    while let Some(start) = cursor {
        if start > end || guard >= 10_000 {
            break;
        }
        let key = period_key(start, granularity);
        filled.push(
            buckets
                .remove(&key)
                .unwrap_or_else(|| empty_bucket(key, start)),
        );
        cursor = next_period(start, granularity);
        guard += 1;
    }
    filled
}

/// Day count per presence type, in canonical order, honoring the `enabled`
/// filter. Disabled types and types without days are omitted.
pub fn breakdown_by_type(
    presences: &[PresenceDto],
    enabled: &[PresenceType],
) -> Vec<(PresenceType, u32)> {
    PresenceType::ALL
        .into_iter()
        .filter(|t| enabled.contains(t))
        .map(|t| {
            let days = presences
                .iter()
                .filter(|p| PresenceType::parse(&p.kind).ok() == Some(t))
                .count() as u32;
            (t, days)
        })
        .filter(|(_, days)| *days > 0)
        .collect()
}

/// Totals of one calendar month, for the header summary.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct MonthTotals {
    pub work_minutes: i64,
    pub co2_kg: f64,
}

/// Sum the work minutes and the (non-null) CO₂ of the days in `year`/`month`.
pub fn month_totals(presences: &[PresenceDto], year: i32, month: u32) -> MonthTotals {
    presences
        .iter()
        .filter(|p| {
            let d = civil_date(p.day);
            d.year() == year && d.month() == month
        })
        .fold(MonthTotals::default(), |acc, p| MonthTotals {
            work_minutes: acc.work_minutes + p.work_minutes,
            co2_kg: acc.co2_kg + p.co2_kg.unwrap_or(0.0),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    /// Minimal presence: only the fields the aggregation reads matter.
    fn presence(y: i32, m: u32, d: u32, kind: &str, co2: Option<f64>, minutes: i64) -> PresenceDto {
        PresenceDto {
            id: format!("p-{y}-{m}-{d}-{kind}"),
            profile_id: "profile".into(),
            day: day_ms(date(y, m, d)),
            kind: kind.into(),
            co2_kg: co2,
            is_estimated: false,
            created_at: 0,
            updated_at: 0,
            work_minutes: minutes,
        }
    }

    fn office(y: i32, m: u32, d: u32) -> PresenceDto {
        presence(y, m, d, "office", Some(0.0), 0)
    }

    #[test]
    fn civil_date_round_trips() {
        assert_eq!(civil_date(0), date(1970, 1, 1));
        assert_eq!(civil_date(1_717_200_000_000), date(2024, 6, 1));
        assert_eq!(day_ms(date(2024, 6, 1)), 1_717_200_000_000);
        assert_eq!(civil_date(day_ms(date(1969, 12, 31))), date(1969, 12, 31));
    }

    #[test]
    fn empty_dataset_has_no_bucket() {
        assert!(bucket_presences(&[], Granularity::Month, None).is_empty());
        assert!(bucket_presences(&[], Granularity::Week, Some(date(2026, 1, 1))).is_empty());
    }

    #[test]
    fn same_month_days_collapse_and_sum() {
        let b = bucket_presences(
            &[
                presence(2026, 3, 3, "office", Some(4.0), 480),
                presence(2026, 3, 18, "office", Some(6.0), 60),
            ],
            Granularity::Month,
            None,
        );
        assert_eq!(b.len(), 1);
        assert_eq!(
            (b[0].co2_kg, b[0].work_minutes, b[0].day_count),
            (10.0, 540, 2)
        );
        assert_eq!(b[0].start, date(2026, 3, 1));
        assert_eq!(b[0].key, 202603);
    }

    #[test]
    fn empty_months_are_gap_filled() {
        let b = bucket_presences(
            &[
                presence(2026, 1, 10, "office", Some(0.0), 60),
                presence(2026, 3, 10, "office", Some(0.0), 60),
            ],
            Granularity::Month,
            None,
        );
        assert_eq!(b.iter().map(|x| x.day_count).collect::<Vec<_>>(), [1, 0, 1]);
        assert_eq!(b[1], empty_bucket(202602, date(2026, 2, 1)));
    }

    #[test]
    fn timeline_extends_until_today() {
        let b = bucket_presences(
            &[office(2026, 1, 10)],
            Granularity::Month,
            Some(date(2026, 3, 5)),
        );
        assert_eq!(b.len(), 3);
        assert_eq!(b[2].day_count, 0);
        // An `until` before the data does not shrink the timeline.
        let b = bucket_presences(
            &[office(2026, 1, 10)],
            Granularity::Month,
            Some(date(2025, 1, 1)),
        );
        assert_eq!(b.len(), 1);
    }

    #[test]
    fn null_co2_is_skipped_but_the_day_counts() {
        let b = bucket_presences(
            &[
                presence(2026, 3, 3, "office", Some(5.0), 480),
                presence(2026, 3, 4, "vacation", None, 0),
            ],
            Granularity::Month,
            None,
        );
        assert_eq!((b[0].co2_kg, b[0].day_count), (5.0, 2));
    }

    #[test]
    fn iso_weeks_use_the_iso_year_across_new_year() {
        // 2025-12-29 (Mon) and 2026-01-01 (Thu) are both ISO week 1 of 2026.
        let b = bucket_presences(
            &[office(2025, 12, 29), office(2026, 1, 1)],
            Granularity::Week,
            None,
        );
        assert_eq!(b.len(), 1);
        assert_eq!((b[0].key, b[0].day_count), (202601, 2));
        assert_eq!(b[0].start, date(2025, 12, 29));
    }

    #[test]
    fn distinct_iso_weeks_are_separated() {
        let b = bucket_presences(
            &[office(2025, 12, 29), office(2026, 1, 5)],
            Granularity::Week,
            None,
        );
        assert_eq!(
            b.iter().map(|x| x.key).collect::<Vec<_>>(),
            [202601, 202602]
        );
        assert_eq!(b.iter().map(|x| x.day_count).collect::<Vec<_>>(), [1, 1]);
    }

    #[test]
    fn period_key_matches_bucket_key() {
        let d = date(2026, 3, 18);
        let b = bucket_presences(&[office(2026, 3, 18)], Granularity::Month, None);
        assert_eq!(period_key(d, Granularity::Month), b[0].key);
        let d = date(2025, 12, 29);
        let b = bucket_presences(&[office(2025, 12, 29)], Granularity::Week, None);
        assert_eq!(period_key(d, Granularity::Week), b[0].key);
        assert_eq!(period_key(d, Granularity::Week), 202601);
    }

    #[test]
    fn breakdown_counts_in_canonical_order_and_honors_filter() {
        let data = [
            presence(2026, 1, 1, "remote", None, 0),
            presence(2026, 1, 2, "office", None, 0),
            presence(2026, 1, 3, "remote", None, 0),
            presence(2026, 1, 4, "vacation", None, 0),
        ];
        assert_eq!(
            breakdown_by_type(&data, &PresenceType::ALL),
            [
                (PresenceType::Office, 1),
                (PresenceType::Remote, 2),
                (PresenceType::Vacation, 1)
            ]
        );
        assert_eq!(
            breakdown_by_type(&data, &[PresenceType::Office]),
            [(PresenceType::Office, 1)]
        );
    }

    #[test]
    fn month_totals_only_sum_the_month() {
        let data = [
            presence(2026, 3, 1, "office", Some(2.5), 480),
            presence(2026, 3, 31, "remote", Some(0.0), 420),
            presence(2026, 3, 15, "vacation", None, 0),
            presence(2026, 4, 1, "office", Some(9.0), 60),
        ];
        assert_eq!(
            month_totals(&data, 2026, 3),
            MonthTotals {
                work_minutes: 900,
                co2_kg: 2.5
            }
        );
        assert_eq!(month_totals(&data, 2025, 3), MonthTotals::default());
    }
}
