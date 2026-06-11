/** Gap between two segments, in normalized path units (pathLength = 100). */
const SEGMENT_GAP = 1.5;
/** Floor so a tiny task stays visible (and clickable) in the ring. */
const MIN_ARC = 0.5;

export interface DonutSegmentInput {
  id: string;
  minutes: number;
}

/** One drawable arc: `start`/`length` in 0-100 path units along the ring. */
export interface DonutArc {
  id: string;
  start: number;
  length: number;
}

/**
 * Lay the day's entries out on a 100-unit ring, proportionally to duration,
 * with a small inter-segment gap (omitted for a single segment so it can close
 * the full circle). Pure, so the proportions are unit-testable.
 *
 * Entries run counter-clockwise from 12 o'clock — the first task fills the
 * top-LEFT and the order reads left to right across the top of the ring —
 * while `start` stays expressed in the SVG path direction (clockwise from
 * the rotated origin), ready for `stroke-dashoffset`.
 */
export function computeDonutArcs(segments: DonutSegmentInput[]): DonutArc[] {
  const total = segments.reduce((sum, s) => sum + s.minutes, 0);
  if (total <= 0) return [];

  const gap = segments.length > 1 ? SEGMENT_GAP : 0;
  let cursor = 0;
  return segments.map((s) => {
    const share = (s.minutes / total) * 100;
    const arc = {
      id: s.id,
      start: 100 - cursor - share + gap / 2,
      length: Math.max(share - gap, MIN_ARC),
    };
    cursor += share;
    return arc;
  });
}
