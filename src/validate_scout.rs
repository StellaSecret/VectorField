//! Automated evidence for *scout, then call* levels (`Rules::Scout`, docs/07-combinations.md). Two decisions, from
//! Survey (where to scout and what the push says) and Predict (which exit to call), must both be necessary, and
//! every number comes from the real simulation.
//!
//! For each hidden source in the pool this measures:
//! - the *true exit* of the main probe, and whether the visible arrows alone (no hidden source) would have named it;
//! - whether a **reader** who launches three scouts, triangulates the hidden source from the push they felt and
//!   simulates the main probe with that estimate calls the exit right;
//! - how **tolerant** that call is: the share of position errors of `TOLERANCE_ERR` (in eight directions) that still
//!   give the right exit, because a person cannot locate the source exactly;
//! - whether the three scouts felt the source at all.
//!
//! And across the pool: the true exits must be reached and pairwise different, so that **no fixed call** (including
//! the visible-only call) is right for more than one hidden source, and a streak of correct calls cannot be had
//! without reading the evidence.
use crate::field::{SourceKind, V2};
use crate::level::{Level, Rules};
use crate::modes::scout::ScoutSpec;
use crate::validate_shape::shifted;

/// The reader's three scouts start here (clamped into the scouting strip): across the lane, level with the probe.
pub const READER_STARTS: [(f64, f64); 3] = [(-5.0, -1.5), (-5.0, 0.0), (-5.0, 1.5)];
/// The position error (world units) the tolerance check applies to the reader's estimate, in eight directions.
pub const TOLERANCE_ERR: f64 = 0.3;
/// The share of those errors that must still give the right exit.
pub const TOLERANCE_MIN: f64 = 0.8;
/// How far the target gates are moved in the robustness check.
pub const MOVE: f64 = 0.3;

#[derive(Clone, Debug, Default)]
pub struct ScoutReport {
    pub entries: usize,
    /// The true exit of each hidden source (`None`: the main probe leaves through no exit).
    pub exits: Vec<Option<usize>>,
    pub visible_exit: Option<usize>,
    /// Every true exit is reached and they are pairwise different.
    pub distinct: bool,
    /// Entries where the visible arrows alone name the true exit.
    pub visible_right: usize,
    /// Entries where the three-scout reader calls the true exit.
    pub reader_right: usize,
    /// Entries where the reader's scouts felt the source (at least two push samples).
    pub seen: usize,
    /// The share of position errors that still give the right exit, over every entry.
    pub tolerance: f64,
    /// The most entries any single fixed call is right for.
    pub best_fixed: usize,
    /// Correct calls in a row needed (the level's rule).
    pub streak: usize,
    /// For each component of the final choice that has a neutral option (default slot, phase or dial; no source; no
    /// gust): the number of hidden sources that some winning option wins with that neutral choice. A level needs
    /// every one of its controls when all of these are 0 (`controls_needed`).
    pub neutral_wins: Vec<Option<usize>>,
}

impl ScoutReport {
    /// Every control of the final choice is needed: no hidden source is beaten with a neutral choice in any component.
    pub fn controls_needed(&self) -> bool {
        self.neutral_wins.iter().flatten().all(|&n| n == 0)
    }
    /// The level needs both decisions: no fixed or visible-only call can pass, the evidence is enough to pass, and
    /// that holds under a person's imprecision.
    pub fn ablation_holds(&self) -> bool {
        self.distinct
            && self.best_fixed < self.streak
            && self.visible_right < self.streak
            && self.reader_right == self.entries
            && self.seen == self.entries
            && self.tolerance >= TOLERANCE_MIN
    }
}

/// Triangulate a hidden source from push samples (position, push). Each push lies along the line through its position
/// and the source, so the source is the point closest to all of those lines (weighted by push strength); the kind
/// follows from whether the pushes point away from it (repeller) or toward it (attractor).
pub fn estimate(samples: &[(V2, V2)]) -> Option<(SourceKind, V2)> {
    if samples.len() < 2 {
        return None;
    }
    let (mut a11, mut a12, mut a22, mut b1, mut b2) = (0.0, 0.0, 0.0, 0.0, 0.0);
    for (p, f) in samples {
        let w = f.len();
        let d = V2 {
            x: f.x / w,
            y: f.y / w,
        };
        let (m11, m12, m22) = (1.0 - d.x * d.x, -d.x * d.y, 1.0 - d.y * d.y);
        a11 += w * m11;
        a12 += w * m12;
        a22 += w * m22;
        b1 += w * (m11 * p.x + m12 * p.y);
        b2 += w * (m12 * p.x + m22 * p.y);
    }
    let det = a11 * a22 - a12 * a12;
    if det.abs() < 1e-6 {
        return None;
    }
    let h = V2 {
        x: (a22 * b1 - a12 * b2) / det,
        y: (a11 * b2 - a12 * b1) / det,
    };
    let s: f64 = samples
        .iter()
        .map(|(p, f)| (f.x * (p.x - h.x) + f.y * (p.y - h.y)) / f.len())
        .sum();
    Some((
        if s > 0.0 {
            SourceKind::Repeller
        } else {
            SourceKind::Attractor
        },
        h,
    ))
}

pub fn report(level: &Level) -> ScoutReport {
    let spec = ScoutSpec::from_level(level);
    let n = spec.rules.pool.len();
    let opts = spec.options();
    let visible_pick = spec.pick_in(&spec.level.field);
    let mut r = ScoutReport {
        entries: n,
        streak: spec.rules.streak,
        visible_exit: visible_pick,
        ..Default::default()
    };
    let (mut tol_ok, mut tol_n) = (0usize, 0usize);
    let mut winner_sets: Vec<Vec<usize>> = vec![];
    for a in 0..n {
        let truth_field = spec.truth_field(a);
        let wins = spec.winners(a);
        r.exits.push(wins.first().copied());
        if visible_pick.is_some_and(|o| spec.try_option(&truth_field, o).0) {
            r.visible_right += 1;
        }
        // the reader: three scouts, triangulate, choose the option that works in the estimated field
        let mut samples = vec![];
        for (x, y) in READER_STARTS {
            let p = V2 {
                x: x.clamp(spec.rules.zone.min.x, spec.rules.zone.max.x),
                y: y.clamp(spec.rules.zone.min.y, spec.rules.zone.max.y),
            };
            samples.extend(spec.scout(a, p).pushes);
        }
        if samples.len() >= 2 {
            r.seen += 1;
        }
        // levels with several choices use the careful reader (the choice that survives the most nearby estimates)
        let careful = matches!(spec.rules.act, crate::level::ScoutAct::Seq(_));
        // an ordered control (the launch phase) is read by aiming at the middle of a window of working options
        let centered = matches!(spec.rules.act, crate::level::ScoutAct::Phase { .. });
        let right = |kind: SourceKind, h: V2| {
            let pick = if careful {
                spec.pick_robust(kind, h, TOLERANCE_ERR)
            } else if centered {
                spec.pick_center(&spec.field_with(kind, h))
            } else {
                spec.pick_in(&spec.field_with(kind, h))
            };
            pick.is_some_and(|o| spec.try_option(&truth_field, o).0)
        };
        if let Some((kind, h)) = estimate(&samples) {
            if right(kind, h) {
                r.reader_right += 1;
            }
            for i in 0..8 {
                let ang = i as f64 * std::f64::consts::TAU / 8.0;
                tol_n += 1;
                if right(
                    kind,
                    V2 {
                        x: h.x + TOLERANCE_ERR * ang.cos(),
                        y: h.y + TOLERANCE_ERR * ang.sin(),
                    },
                ) {
                    tol_ok += 1;
                }
            }
        } else {
            tol_n += 8;
        }
        winner_sets.push(wins);
    }
    r.tolerance = if tol_n == 0 {
        0.0
    } else {
        tol_ok as f64 / tol_n as f64
    };
    // every hidden source has a winning option and no option wins for two of them
    r.distinct = winner_sets.iter().all(|w| !w.is_empty())
        && (0..opts).all(|o| winner_sets.iter().filter(|w| w.contains(&o)).count() <= 1);
    let spec_n = spec.components().len();
    r.neutral_wins = (0..spec_n)
        .map(|i| {
            spec.neutral(i).map(|nz| {
                winner_sets
                    .iter()
                    .filter(|w| w.iter().any(|&o| spec.decode(o)[i] == nz))
                    .count()
            })
        })
        .collect();
    r.best_fixed = (0..opts)
        .map(|o| winner_sets.iter().filter(|w| w.contains(&o)).count())
        .max()
        .unwrap_or(0);
    r
}

/// For each of the four moves of the exit gates by `MOVE`: does the ablation still hold?
pub fn robustness(level: &Level) -> Vec<((f64, f64), bool)> {
    [(MOVE, 0.0), (-MOVE, 0.0), (0.0, MOVE), (0.0, -MOVE)]
        .iter()
        .map(|&(dx, dy)| ((dx, dy), report(&shifted(level, dx, dy)).ablation_holds()))
        .collect()
}

pub fn solvable(level: &Level) -> bool {
    let r = report(level);
    r.entries > 0 && r.reader_right == r.entries
}

pub fn summarize(level: &Level) -> String {
    let r = report(level);
    format!(
        "{} hidden sources; true exits {:?} (visible-only {:?}), distinct {}; reader right {}/{}, scouts felt the source {}/{}, tolerance {:.0}%, best fixed call right for {}",
        r.entries, r.exits, r.visible_exit, r.distinct, r.reader_right, r.entries, r.seen, r.entries, 100.0 * r.tolerance, r.best_fixed
    )
}

/// Markdown table for `cargo run --release --bin report -- --combo`.
pub fn markdown(levels: &[Level]) -> String {
    let mut s = String::from("\n### Scout, then call\n\nA hidden source from a pool (drawn at random each attempt, never the one just used) bends every flight. The reader launches three scouts at fixed starts, triangulates the source from the push they felt, and simulates the main probe with that estimate. *Tolerance* is the share of position errors of 0.3 in that estimate that still give the right exit.\n\n| level | hidden sources | winning option | visible-only right | best fixed call right | reader right | scouts felt it | tolerance | streak needed | robust to moved gates |\n|---|---|---|---|---|---|---|---|---|---|\n");
    for l in levels {
        let r = report(l);
        let rob = robustness(l);
        let Rules::Scout(ru) = &l.rules else { continue };
        let spec = ScoutSpec::from_level(l);
        let _ = ru;
        let name = |e: &Option<usize>| e.map_or("none".to_string(), |i| spec.option_label(i));
        let exits = r.exits.iter().map(name).collect::<Vec<_>>().join(", ");
        s.push_str(&format!(
            "| {} {} | {} | {} | {}/{} | {}/{} | {}/{} | {}/{} | {:.0}% | {} | {}/4 |\n",
            l.id,
            l.title,
            r.entries,
            exits,
            r.visible_right,
            r.entries,
            r.best_fixed,
            r.entries,
            r.reader_right,
            r.entries,
            r.seen,
            r.entries,
            100.0 * r.tolerance,
            r.streak,
            rob.iter().filter(|(_, ok)| *ok).count()
        ));
    }
    s
}

/// Is this a Scout level?
pub fn is_scout(level: &Level) -> bool {
    matches!(level.rules, Rules::Scout(_))
}
