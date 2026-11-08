//! Geometry of the work-hours donut (ported from `donut-geometry.ts`).

/// Gap between two segments, in normalized ring units (the ring = 100).
const SEGMENT_GAP: f64 = 1.5;
/// Floor so a tiny task stays visible (and clickable) in the ring.
const MIN_ARC: f64 = 0.5;

/// One drawable arc: `start` / `length` in 0–100 units along the ring.
#[derive(Debug, Clone, PartialEq)]
pub struct DonutArc {
    pub id: String,
    pub start: f64,
    pub length: f64,
}

/// Lay the day's entries (`(id, minutes)`) out on a 100-unit ring,
/// proportionally to duration, with a small gap between segments (omitted for
/// a single segment so it closes the full circle).
///
/// Entries run counter-clockwise from 12 o'clock — the first task fills the
/// top-left and the order reads left to right across the top of the ring —
/// while `start` is expressed clockwise from 12 o'clock.
pub fn compute_donut_arcs(segments: &[(String, i64)]) -> Vec<DonutArc> {
    let total: i64 = segments.iter().map(|(_, m)| *m).sum();
    if total <= 0 {
        return Vec::new();
    }
    let gap = if segments.len() > 1 { SEGMENT_GAP } else { 0.0 };
    let mut cursor = 0.0;
    segments
        .iter()
        .map(|(id, minutes)| {
            let share = *minutes as f64 / total as f64 * 100.0;
            let arc = DonutArc {
                id: id.clone(),
                start: 100.0 - cursor - share + gap / 2.0,
                length: (share - gap).max(MIN_ARC),
            };
            cursor += share;
            arc
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seg(id: &str, minutes: i64) -> (String, i64) {
        (id.to_string(), minutes)
    }

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn empty_day_has_no_arc() {
        assert!(compute_donut_arcs(&[]).is_empty());
        assert!(compute_donut_arcs(&[seg("a", 0)]).is_empty());
    }

    #[test]
    fn single_task_closes_the_ring() {
        let arcs = compute_donut_arcs(&[seg("a", 90)]);
        assert!(close(arcs[0].start, 0.0));
        assert!(close(arcs[0].length, 100.0));
    }

    #[test]
    fn ring_is_split_proportionally_counter_clockwise() {
        let arcs = compute_donut_arcs(&[seg("a", 120), seg("b", 60), seg("c", 60)]);
        assert!(close(arcs[0].start, 50.75) && close(arcs[0].length, 48.5));
        assert!(close(arcs[1].start, 25.75) && close(arcs[1].length, 23.5));
        assert!(close(arcs[2].start, 0.75) && close(arcs[2].length, 23.5));
        // Consecutive tasks stay adjacent.
        assert!(close(
            arcs[1].start + arcs[1].length + SEGMENT_GAP,
            arcs[0].start
        ));
        assert!(close(
            arcs[2].start + arcs[2].length + SEGMENT_GAP,
            arcs[1].start
        ));
    }

    #[test]
    fn tiny_task_keeps_a_minimum_arc() {
        let arcs = compute_donut_arcs(&[seg("tiny", 5), seg("big", 480)]);
        assert!(arcs[0].length >= MIN_ARC);
    }
}
