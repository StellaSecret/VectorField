//! All content, as data. Field types, constraints and mechanics are combined here and nowhere else.
use crate::field::{v2, Field, SourceKind, SwirlKind, Term, V2};
use crate::level::*;
use crate::sim::{Rect, Region, RegionKind};
use std::f64::consts::TAU;

const B: Rect = Rect::new(-6.0, -4.0, 6.0, 4.0);

fn radial(x: f64, y: f64, k: f64, soft: f64) -> Term {
    Term::Radial {
        c: v2(x, y),
        k,
        soft,
    }
}
fn vortex(x: f64, y: f64, k: f64, soft: f64) -> Term {
    Term::Vortex {
        c: v2(x, y),
        k,
        soft,
    }
}
fn uni(x: f64, y: f64) -> Term {
    Term::Uniform(v2(x, y))
}
fn right_exits() -> Vec<Region> {
    vec![
        Region::exit(5.2, 3.0, 0.85, 'A'),
        Region::exit(5.2, 1.0, 0.85, 'B'),
        Region::exit(5.2, -1.0, 0.85, 'C'),
        Region::exit(5.2, -3.0, 0.85, 'D'),
    ]
}
fn ring_exits() -> Vec<Region> {
    // Corner exits: together they catch most of the boundary a spiral can reach.
    vec![
        Region::exit(5.6, 3.6, 2.2, 'A'),
        Region::exit(5.6, -3.6, 2.2, 'B'),
        Region::exit(-5.6, -3.6, 2.2, 'C'),
        Region::exit(-5.6, 3.6, 2.2, 'D'),
    ]
}
fn zr(x0: f64, y0: f64, x1: f64, y1: f64) -> Zone {
    Zone::Rect(Rect::new(x0, y0, x1, y1))
}
fn base(
    id: &'static str,
    chapter: Chapter,
    title: &'static str,
    objective: &'static str,
    field: Field,
    regions: Vec<Region>,
    rules: Rules,
) -> Level {
    Level {
        id,
        chapter,
        title,
        objective,
        hints: vec![],
        field,
        bounds: B,
        regions,
        rules,
        max_time: 40.0,
        seed: id.bytes().fold(7u64, |a, b| a * 31 + b as u64),
    }
}
fn hints(mut l: Level, h: &[&'static str]) -> Level {
    l.hints = h.to_vec();
    l
}
fn pos(slots: Vec<Slot>) -> Rules {
    Rules::Position {
        slots,
        period: None,
        intensity: None,
    }
}
fn slot(zone: Zone, goal: usize) -> Slot {
    Slot { zone, goal }
}
fn pred(zone: Zone, reveal: Reveal) -> Rules {
    Rules::Predict {
        zone,
        rounds: 3,
        pass: 2,
        reveal,
    }
}

// Periodic helpers for Tempo (period 12 s)
const T12: f64 = 12.0;
fn square_up(_p: V2, t: f64) -> V2 {
    v2(0.0, (4.0 * (TAU * t / T12).sin()).tanh())
}

// Leverage field for Steer level 2: a rightward stream whose vertical divergence is strong around
// x = 0 (a small sideways offset entering that zone is amplified) and mildly convergent beyond it.
fn lever_field(p: V2, _t: f64) -> V2 {
    let dz = (-(p.x / 1.5).powi(2)).exp();
    let k = 1.3 * dz - 0.35 * (1.0 - dz) * (p.x > 0.0) as i32 as f64;
    v2(1.4, k * p.y)
}

// Survey tuning (see docs/09-survey.md and docs/SURVEY_VALIDATION.md for how these were chosen).
const SURVEY_K: f64 = 1.6;
const SURVEY_SIGMA: f64 = 1.0;
/// How close the committed marker must be to count as found. 0.5 was the first guess and was never calibrated;
/// in play, six consecutive misses landed at 0.52, 0.55, 0.61, 0.66, 0.90 and 1.05 (people located the region but
/// not to 0.5). 1.0 (= sigma) was tried and rejected: the free scout alone then settles (-2.75, 2.25), which the level
/// must never allow. 0.75 keeps every authored position valid (see docs/SURVEY_VALIDATION.md) and is the middle ground.
const SURVEY_TOL: f64 = 0.75;
/// Level 5 of Survey ("The Gyre"): the same source as level 3 (sigma 0.8, k 1.65, tolerance 0.6) in a background that
/// circulates. The charted flow is a counter-clockwise vortex about the centre of the arena (`GYRE_K`, core 1.5) with a
/// gentle inward drift (`GYRE_DRIFT`), so a scout does not fly straight: it spirals in, at about 0.9-1.1 (peak push 0.8
/// stays under it, nothing stalls). Where it will have been after five seconds is no longer where it started, so the
/// player has to choose a release point for where the scout will *pass*, and the charted-path preview shows exactly that.
/// The free scout is released at (-4, 0), flies down the left side and ends at (-0.9, -3.2).
/// The positions are in play order: a graze on the left (two weak arrows from the free scout), one far along the free
/// scout's own streamline (it ends 5 s short of it: a scout released *where the free one stopped* gets there), then quiet
/// ones that none of the free scout's ground reaches. Every one shows the free scout at most two arrows, is read by no one
/// scout alone, is solved by well-placed scouts, and no two are within twice the tolerance (tested).
const GYRE_K: f64 = 4.0;
const GYRE_SOFT: f64 = 1.5;
const GYRE_DRIFT: f64 = 0.6;
const SURVEY5_POOL: [(f64, f64); 6] = [
    (-3.0, 1.0),
    (2.5, -0.5),
    (0.5, 2.0),
    (1.0, -1.5),
    (-1.5, 2.5),
    (3.5, -1.5),
];
/// Level 4 of Survey ("Two of Them"): the same source as level 3 (sigma 0.8, k 1.65, tolerance 0.6), two of it at a time,
/// in order of separation 4.5 (one above the other), 4.0 (diagonal), then side by side along the stream at 3.2, 2.8, 2.4
/// and 2.2. Every pair lies about 1.5-1.7 above or below the free scout's line, within its first reach, so the free
/// scout **always has a lead**: it comes within 1.6-2.1 of both sources and shows 2-5 arrows. (The first layout put
/// pairs anywhere in the arena: the free scout felt at most one source, and in three attempts neither, so the player
/// found the first blind and had nothing to follow to the second; "what should I do?".) Side by side, the arrows look
/// like one source in the middle and the stacked ghost fits the free trail (deviation about 0.2): the lead says
/// "something is there", and telling one from two is the level. Each pair is also at its own place, no two within twice
/// the tolerance of each other. (first, second).
const SURVEY4_K: f64 = 1.65;
const SURVEY4_SIGMA: f64 = 0.8;
const SURVEY4_TOL: f64 = 0.6;
const SURVEY4_POOL: [((f64, f64), (f64, f64)); 6] = [
    ((-3.7, 1.7), (-0.75, -1.7)),
    ((-0.3, 1.7), (-2.4, -1.7)),
    ((-0.3, 1.6), (-3.5, 1.6)),
    ((0.2, -1.6), (-2.6, -1.6)),
    ((0.3, 1.5), (-2.1, 1.5)),
    ((-1.9, -1.5), (-4.1, -1.5)),
];
/// Level 3 of Survey ("One Look Is a Line"): peak push 0.8 (k * sigma * e^-0.5), felt out to 1.87 at `PUSH_MIN`.
const SURVEY3_K: f64 = 1.65;
const SURVEY3_SIGMA: f64 = 0.8;
/// 0.75 sigma, as for level 1.
const SURVEY3_TOL: f64 = 0.6;
/// Hidden positions and kinds in play order: graze, head-on, quiet, graze, graze, quiet, so that the three attempts
/// it takes to finish the level (`SurveyGoal::Finds(3)`) are one of each situation and, as it happens, one of each kind.
/// There is one head-on only: it must sit 1.4-1.6 beyond the free trail's end, so a second one would be within the
/// tolerance of the first and a player could pass it by re-marking the earlier answer. No two positions are within
/// twice the tolerance of each other (tested).
/// The free scout flies y = 0 from x = -5 and ends at x = 0 (sensing radius 1.87). Graze sources sit about 1.5 from
/// that line or its end, head-on ones on it 1.4-1.6 beyond its end (one weak arrow, and the slowdown is too small to
/// read from the dashes), quiet ones more than 2.4 off it. Checked by `tests/survey.rs` (`level_three_*`).
const SURVEY3_POOL: [(f64, f64, SourceKind); 6] = [
    (-1.5, 1.6, SourceKind::Repeller),
    (1.55, 0.05, SourceKind::Attractor),
    (3.0, 3.0, SourceKind::SwirlCcw),
    (0.5, -1.6, SourceKind::SwirlCw),
    (0.4, 1.5, SourceKind::Repeller),
    (2.5, -3.3, SourceKind::Attractor),
];
/// Level 2 of Survey ("Which Way It Pushes"): peak push 0.79, tolerance 0.75 (sigma 1). The level is complete when a
/// repeller, an attractor and a swirl have each been found, and the next attempt is always of the first kind still
/// missing (`pick_index` in modes/survey.rs), so a miss gives another of the same kind. Each kind has two or three
/// positions, in the order they are used: repeller, repeller; attractor, attractor; swirl CCW, swirl CW, then a hard
/// swirl close to the free trail.
const SURVEY2_K: f64 = 1.3;
const SURVEY2_SIGMA: f64 = 1.0;
const SURVEY2_TOL: f64 = 0.75;
const SURVEY2_POOL: [(f64, f64, SourceKind); 7] = [
    (-2.5, 1.2, SourceKind::Repeller),
    (-1.5, -1.3, SourceKind::Attractor),
    (-3.0, 1.1, SourceKind::SwirlCcw),
    (-2.6, -0.9, SourceKind::Repeller),
    (-2.2, 1.4, SourceKind::Attractor),
    (-1.0, -1.2, SourceKind::SwirlCw),
    (-1.8, 0.4, SourceKind::SwirlCcw),
];
/// The six positions level 1 first shipped with, chosen by the trail-similarity validator at a time when the free scout
/// showed no arrows on any of them (see `survey1_legacy`, `validate_survey::report` and `tests/survey.rs`).
pub const SURVEY_LEGACY_POOL: [(f64, f64); 6] = [
    (-3.25, -2.25),
    (-2.75, 2.25),
    (-0.75, -2.25),
    (-0.25, 2.25),
    (3.25, -1.75),
    (2.25, 2.25),
];
/// Seconds of trail a scout leaves.
const SURVEY_TRAIL: f64 = 5.0;
/// Level 1's hidden positions, in play order. The first two are **grazes** (about 2 from the free scout's path): it
/// feels the source and shows 5 arrows, so the arrows and rays are on screen from the first attempt, yet reading them
/// with error rarely settles it from the free scout alone (2-5% careful, 0% by eye) and a scout released well solves it
/// (92%). The other four are out of the free scout's reach (no arrows: a search). The level is complete after one find,
/// so a player who reads the first attempt well never meets the searches. No two positions are within twice the
/// tolerance (tested), so an earlier answer cannot be reused. The six positions the level first shipped with, chosen by
/// the older trail-similarity validator when the free scout showed no arrows on any of them, are `SURVEY_LEGACY_POOL`.
const SURVEY_POOL: [(f64, f64); 6] = [
    (-2.4, 2.0),
    (-1.5, -1.9),
    (-3.25, -2.25),
    (-0.25, 2.25),
    (3.25, -1.75),
    (2.25, 2.25),
];

pub fn position_levels() -> Vec<Level> {
    vec![
        hints(base("pos-1", Chapter::Position, "Open Water", "Drop the probe anywhere in the zone so it reaches the target.",
            Field::new().fixed(uni(0.6, 0.0)).fixed(radial(4.0, 0.5, -2.4, 0.6)),
            vec![Region::target(4.0, 0.5, 0.6)],
            pos(vec![slot(zr(-5.5, -3.0, -3.8, 3.0), 0)])),
            &["The target pulls everything toward it. Arrows near it all point inward.", "Almost any start works here. Watch where the arrows lead."]),
        hints(base("pos-2", Chapter::Position, "Two Roads", "The vortex splits the flow. Find a start that ends at the target.",
            Field::new().fixed(uni(0.7, 0.0)).fixed(vortex(0.0, 0.0, -2.5, 0.8)),
            vec![Region::target(4.0, -2.0, 0.6)],
            pos(vec![slot(zr(-5.5, -3.0, -4.0, 3.0), 0)])),
            &["Clockwise vortex: above it the flow runs right, below it the flow runs left.", "Starts above and below the centre line take very different roads."]),
        hints(base("pos-3", Chapter::Position, "Saddle Point", "Only part of the zone can reach the target. Which part?",
            Field::new().fixed(Term::Saddle { c: v2(4.0, 0.0), k: 0.5 }),
            vec![Region::target(3.0, 2.5, 0.8)],
            pos(vec![slot(Zone::Circle(v2(-4.2, 0.0), 1.8), 0)])),
            &["Flow squeezes in along the horizontal axis and is flung out vertically.", "Probes above the axis are thrown up, those below are thrown down."]),
        hints(base("pos-4", Chapter::Position, "Mirror Mirror", "The field is symmetric. The goal is not. Avoid the hazard.",
            Field::new().fixed(uni(0.5, 0.0)).fixed(vortex(0.0, 1.6, 1.5, 0.7)).fixed(vortex(0.0, -1.6, -1.5, 0.7)),
            vec![Region::target(4.5, 1.2, 0.6), Region::hazard(4.5, -1.2, 0.6)],
            pos(vec![slot(zr(-5.5, -3.0, -4.0, 3.0), 0)])),
            &["The two vortices are mirror images, so a path and its mirror path look equally good.", "The mirror path of a winning start ends in the hazard."]),
        hints(base("pos-5", Chapter::Position, "The Needle", "A tiny target and a sensitive field. Aim carefully.",
            Field::new().fixed(Term::Saddle { c: v2(5.0, 0.0), k: 0.7 }),
            vec![Region::target(3.8, 1.6, 0.3)],
            pos(vec![slot(Zone::Circle(v2(-4.5, 0.0), 1.2), 0)])),
            &["Near a saddle, tiny differences in the start grow quickly.", "Use the arrow keys for fine adjustments of the release point."]),
    ]
}

pub fn predict_levels() -> Vec<Level> {
    vec![
        hints(base("pre-1", Chapter::Predict, "Straight Shot", "Which exit will the probe leave through?",
            Field::new().fixed(uni(1.0, 0.0)), right_exits(),
            pred(zr(-5.5, -3.0, -4.0, 3.0), Reveal::Hidden)),
            &["A uniform flow moves everything in one direction at one speed."]),
        hints(base("pre-2", Chapter::Predict, "Gentle Curve", "The flow bends. Follow it.",
            Field::new().fixed(uni(1.0, 0.0)).fixed(vortex(-1.0, -5.0, -4.0, 1.0)), right_exits(),
            pred(zr(-5.5, -3.0, -4.0, 3.0), Reveal::Hidden)),
            &["A distant vortex adds a steady sideways push that bends every path."]),
        hints(base("pre-3", Chapter::Predict, "Spiral Out", "Probes spiral around the centre. Where do they escape?",
            Field::new().fixed(vortex(0.0, 0.0, 2.0, 0.6)).fixed(radial(0.0, 0.0, 0.8, 0.6)), ring_exits(),
            pred(Zone::Circle(v2(0.0, 0.0), 2.2), Reveal::Hidden)),
            &["The vortex turns the probe while the outward push makes it spiral outward.", "The turning direction tells you which exit comes first."]),
        hints(base("pre-4", Chapter::Predict, "False Lead", "The arrows near the start point at one exit. Think further ahead.",
            Field::new().fixed(uni(0.9, 0.0)).fixed(vortex(1.0, 0.0, 3.0, 0.5)), right_exits(),
            pred(zr(-5.5, -3.0, -4.0, 3.0), Reveal::Hidden)),
            &["Do not trust the first arrows. A strong vortex waits halfway across.", "Where the probe meets the vortex decides which way it is carried."]),
        hints(base("pre-5", Chapter::Predict, "Crosswinds", "Several features act in turn. Extrapolate step by step.",
            Field::new().fixed(uni(0.06, 0.0)).fixed(vortex(-1.17, -0.58, 2.52, 0.6)).fixed(vortex(-0.86, -0.99, -1.34, 0.6)).fixed(radial(1.03, 0.55, 1.04, 0.6)),
            ring_exits(),
            pred(Zone::Circle(v2(0.0, 0.0), 2.8), Reveal::Hidden)),
            &["Break the path into stages: which feature does the probe meet first, and where does it leave that feature?", "A pair of opposite vortices near the centre acts like a jet. The repeller on the right sends what reaches it outward."]),
    ]
}

pub fn tempo_levels() -> Vec<Level> {
    let w = TAU / T12;
    let tempo = |start: V2, goal: usize| Rules::Tempo {
        start,
        period: T12,
        goal,
        dial: None,
    };
    vec![
        hints(base("tem-1", Chapter::Tempo, "Two Moods", "The field has two phases. Launch in the right one.",
            Field::new().fixed(uni(3.0, 0.0)).fixed(Term::Custom(square_up)),
            vec![Region::target(4.5, 3.0, 0.9)], tempo(v2(-4.5, 0.0), 0)),
            &["For half the cycle the field lifts the probe, for the other half it pushes it down.", "Watch the timeline while the field plays."]),
        hints(base("tem-2", Chapter::Tempo, "Slow Tide", "A broad window of good launch times.",
            Field::new().fixed(uni(2.5, 0.0)).fixed(Term::Modulated { inner: Box::new(uni(0.0, 1.0)), offset: 0.0, amp: 1.0, omega: w, phase: 0.0 }),
            vec![Region::target(4.5, 2.6, 0.9)], tempo(v2(-4.5, 0.0), 0)),
            &["The vertical push swings smoothly up and down. The probe feels the sum over its whole trip."]),
        hints(base("tem-3", Chapter::Tempo, "Knife Edge", "Only a narrow window works.",
            Field::new().fixed(uni(2.5, 0.0)).fixed(Term::Modulated { inner: Box::new(uni(0.0, 1.0)), offset: 0.0, amp: 1.0, omega: w, phase: 0.0 }),
            vec![Region::target(4.5, 0.0, 0.4)], tempo(v2(-4.5, 0.0), 0)),
            &["The probe must feel equal up and down pushes. That balance happens at very particular phases."]),
        hints(base("tem-4", Chapter::Tempo, "Double Beat", "The field repeats twice per cycle: several windows exist.",
            Field::new().fixed(uni(2.5, 0.0)).fixed(Term::Modulated { inner: Box::new(uni(0.0, 1.6)), offset: 0.0, amp: 1.0, omega: 2.0 * w, phase: 0.0 }),
            vec![Region::target(4.5, 2.0, 0.7)], tempo(v2(-4.5, 0.0), 0)),
            &["Two full swings fit in one cycle, so the same outcome can be reached at more than one phase."]),
        hints(base("tem-5", Chapter::Tempo, "Moving Vortex", "A vortex circles the centre. Time your pass.",
            Field::new().fixed(uni(2.5, 0.0)).fixed(Term::Orbiting { kind: SwirlKind::Vortex, c: v2(0.0, 0.0), orbit: 2.0, omega: w, phase: 0.0, k: 3.0, soft: 0.6 }),
            vec![Region::target(4.5, 2.0, 0.7)], tempo(v2(-4.5, 0.0), 0)),
            &["Where the vortex is when the probe passes decides whether it is lifted or dropped.", "The vortex travels one lap per cycle."]),
    ]
}

fn spiral() -> Field {
    Field::new()
        .fixed(radial(0.0, 0.0, 1.2, 0.6))
        .tunable(vortex(0.0, 0.0, -1.0, 0.6))
}

pub fn experiment_levels() -> Vec<Level> {
    let ex = |start: V2, range: (f64, f64), challenge: Challenge| Rules::Experiment {
        start,
        range,
        challenge,
    };
    vec![
        hints(base("exp-1", Chapter::Experiment, "Which Way?", "Learn how intensity moves the exit, then report the rule.",
            spiral(), ring_exits(),
            ex(v2(1.4, 0.0), (0.0, 2.0), Challenge::ExitShift { center: v2(0.0, 0.0) })),
            &["Run several intensities and keep the runs visible: compare where each one leaves.", "Look at the order of the exits as the number grows."]),
        hints(base("exp-2", Chapter::Experiment, "The Switch", "Find the intensity where the exit changes.",
            Field::new().fixed(uni(1.5, 0.0)).tunable(vortex(0.0, 0.0, 2.0, 0.8)),
            vec![Region::exit(4.0, 2.0, 2.0, 'A'), Region::exit(4.0, -2.0, 2.0, 'B')],
            ex(v2(-4.5, 0.5), (0.0, 3.0), Challenge::Threshold { from: 0, to: 1, tol: 0.15 })),
            &["Find one value for each exit, then narrow the gap between them like a search.", "Preserved runs show which side of the switch you are on."]),
        hints(base("exp-3", Chapter::Experiment, "Round Trip", "Does the probe ever come back to an earlier exit?",
            spiral(), ring_exits(),
            ex(v2(1.4, 0.0), (0.0, 8.0), Challenge::Return)),
            &["Sweep the whole range and note the exits in order.", "The exit keeps moving around the ring as the vortex strengthens."]),
        hints(base("exp-4", Chapter::Experiment, "Beyond the Rig", "The test rig stops at intensity 4. Predict what happens at 6.2.",
            spiral(), ring_exits(),
            ex(v2(1.4, 0.0), (0.0, 8.0), Challenge::Extrapolate { limit: 4.0, target: 6.2 })),
            &["You cannot test 6.2, so look for a rule. Measure how far each exit sits from the last one as intensity grows.", "The winding angle grows steadily with intensity. A full extra turn brings the probe back to the same exit."]),
        hints(base("exp-5", Chapter::Experiment, "The Window", "Find the interval that sends the probe through exit D.",
            spiral(), ring_exits(),
            ex(v2(1.4, 0.0), (0.0, 5.0), Challenge::Interval { exit: 3, tol: 0.2 })),
            &["Exit D is reached for a band of intensities, not a single value.", "Find both edges of the band."]),
    ]
}

fn steer(start: V2, beats: usize, budget: usize, strength: f64, goal: usize) -> Rules {
    Rules::Steer {
        start,
        beats,
        gust_budget: budget,
        gust_strength: strength,
        dirs: GustDirs::Eight,
        beat_time: 1.0,
        goal,
        period: None,
        start_zone: None,
        sources: None,
        dial: None,
        companion: None,
    }
}

pub fn steer_levels() -> Vec<Level> {
    vec![
        hints(base("ste-1", Chapter::Steer, "First Gust", "Nudge the probe so it reaches the target. A gust changes its velocity for one beat.",
            Field::new().fixed(uni(1.6, 0.05)),
            vec![Region::target(2.9, 0.1, 0.5)],
            steer(v2(-5.0, -0.9), 5, 1, 1.0, 0)),
            &["Left alone, the probe drifts past just below the target.", "A gust adds to the velocity for one beat only. The field carries the probe on afterwards."]),
        hints(base("ste-2", Chapter::Steer, "Leverage", "The same gust matters more in some places than others. Find where.",
            Field::new().fixed(Term::Custom(lever_field)),
            vec![Region::target(3.2, -1.6, 0.5)],
            steer(v2(-5.0, 0.15), 6, 2, 0.35, 0)),
            &["Watch how the arrows spread apart around the middle of the field: offsets grow there.", "A small push before the spreading zone is magnified. The same push afterwards is not."]),
        hints(base("ste-3", Chapter::Steer, "Tipping Point", "A saddle splits the flow. Cross to the other side in time.",
            Field::new().fixed(uni(1.6, 0.0)).fixed(Term::Saddle { c: v2(-0.7, 0.0), k: 0.27 }),
            vec![Region::target(2.62, -1.41, 0.7)],
            steer(v2(-5.0, 0.3), 6, 1, 1.0, 0)),
            &["Above the centre line the flow is thrown up, below it down. The probe starts just above it.", "The dividing line is close now. Every beat the probe drifts further from it."]),
        hints(base("ste-4", Chapter::Steer, "The Long Way", "The flow circles the centre. Pushing straight at the target will not get you there.",
            Field::new().fixed(vortex(0.0, 0.0, 8.5, 0.8)).fixed(radial(0.0, 0.0, -0.45, 0.8)),
            vec![Region::target(3.3, 0.3, 0.5)],
            steer(v2(-2.1, 0.0), 7, 2, 1.0, 0)),
            &["The flow carries the probe around the centre, anticlockwise, and slowly inward.", "The target is on a wider orbit than the probe. Which way does the probe need to move to join it?"]),
        hints(base("ste-5", Chapter::Steer, "Set Up", "The first gust is for the second one. The target is small.",
            Field::new().fixed(uni(0.65, 0.0)).fixed(vortex(-1.9, 0.43, 1.41, 0.7)).fixed(vortex(-0.06, 0.6, -1.21, 0.7)).fixed(radial(1.71, -0.27, 1.37, 0.7)),
            vec![Region::target(0.82, 1.79, 0.3)],
            steer(v2(-5.0, 0.94), 8, 2, 1.0, 0)),
            &["A good first move changes what the second move can do. It may not look like progress.", "Do not just close the distance. Look for the flow that will do the work for you."]),
    ]
}

/// Level 1 of Survey with its original six positions, for the legacy validator only.
pub fn survey1_legacy() -> Level {
    let mut l = survey_levels().remove(0);
    if let Rules::Survey { pool, kinds, .. } = &mut l.rules {
        *pool = SURVEY_LEGACY_POOL.iter().map(|&(x, y)| v2(x, y)).collect();
        *kinds = vec![SourceKind::Repeller; pool.len()];
    }
    l
}

pub fn survey_levels() -> Vec<Level> {
    // Charted background: a steady stream to the right. The hidden source is not part of `field`.
    let mut l = base(
        "sur-1",
        Chapter::Survey,
        "Something in the Stream",
        "Something hidden pushes the flow away from itself. Fly scouts, then mark where it is.",
        Field::new().fixed(uni(1.0, 0.0)),
        vec![],
        Rules::Survey {
            zone: zr(-5.5, -3.5, 5.5, 3.5),
            free: v2(-5.0, 0.0),
            scouts: 2,
            k: SURVEY_K,
            sigma: SURVEY_SIGMA,
            tol: SURVEY_TOL,
            rays: true,
            band: false,
            preview: false,
            pool: SURVEY_POOL.iter().map(|&(x, y)| v2(x, y)).collect(),
            kinds: vec![SourceKind::Repeller; SURVEY_POOL.len()],
            partners: vec![None; SURVEY_POOL.len()],
            goal: SurveyGoal::Finds(1),
        },
    );
    l.max_time = SURVEY_TRAIL;
    let one = hints(l, &[
        "The grey arrows show the flow with nothing hidden. Where a scout's path bends away from them, something hidden is pushing it.",
        "The small arrows on a trail show that push, and they point away from the source. Extend them backwards in your head, or follow the faint lines: where they cross is the source.",
        "A scout only feels the source from about 2.4 away. If your first scouts felt nothing, release the next one nearer to where you suspect it.",
    ]);

    // One Look Is a Line: same repeller, narrower and weaker (peak push 0.8, below the stream speed so nothing
    // stalls), and the hidden positions are chosen so that the free scout never settles one. Graze: at the edge of
    // the free scout's reach, so it shows one or two weak arrows (a long thin wedge). Head-on: just beyond the end of
    // its trail, on its own line, so its arrows are collinear and say "on this line", not where. Quiet: out of reach,
    // so it shows nothing. The shaded band turns that silence into evidence.
    let mut two = base("sur-3", Chapter::Survey, "One Look Is a Line",
        "Something hidden pushes the flow. One scout may only tell you which line it is on. No guide lines now.",
        Field::new().fixed(uni(1.0, 0.0)),
        vec![],
        Rules::Survey { zone: zr(-5.5, -3.5, 5.5, 3.5), free: v2(-5.0, 0.0), scouts: 2, k: SURVEY3_K, sigma: SURVEY3_SIGMA, tol: SURVEY3_TOL, rays: false, band: true, preview: false, pool: SURVEY3_POOL.iter().map(|&(x, y, _)| v2(x, y)).collect(), kinds: SURVEY3_POOL.iter().map(|&(_, _, k)| k).collect(), partners: vec![None; SURVEY3_POOL.len()], goal: SurveyGoal::Finds(3) });
    two.max_time = SURVEY_TRAIL;
    // The hints for this level are chosen by what the free scout showed (see `survey_hint_texts` in app.rs);
    // these are the general ones, shown if that cannot be told.
    let two = hints(two, &[
        "Extend the small arrows: the source is somewhere along each of those lines (at right angles to them, if it is a swirl).",
        "To find where along the line, send a scout from a bit above or below it. Its arrows give a different line, and where the lines cross is the source.",
        "The shaded ground is where a source would already have been felt. It is not there.",
    ]);

    // Which Way It Pushes: the source may push away, pull in, or push sideways, and the marker must say which.
    // Peak push 0.79 (below the stream speed, so nothing stalls). All six sources are within reach of the free
    // scout and its type is readable from it alone: this is the level where the player learns the kinds.
    let mut three = base("sur-2", Chapter::Survey, "Which Way It Pushes",
        "Something hidden pushes the flow. It may push away, pull in, or push sideways. Say which, and where.",
        Field::new().fixed(uni(1.0, 0.0)),
        vec![],
        Rules::Survey { zone: zr(-5.5, -3.5, 5.5, 3.5), free: v2(-5.0, 0.0), scouts: 2, k: SURVEY2_K, sigma: SURVEY2_SIGMA, tol: SURVEY2_TOL, rays: true, band: false, preview: false,
            pool: SURVEY2_POOL.iter().map(|&(x, y, _)| v2(x, y)).collect(), kinds: SURVEY2_POOL.iter().map(|&(_, _, k)| k).collect(), partners: vec![None; SURVEY2_POOL.len()], goal: SurveyGoal::EachKind });
    three.max_time = SURVEY_TRAIL;
    let three = hints(three, &[
        "Look at the small arrows around the spot where the trails bend. Pointing away from it: a repeller. Pointing toward it: an attractor. Going around it: a swirl. The key under the field shows all three.",
        "The faint lines run along each arrow. For a repeller or an attractor they cross at the source. A swirl's arrows point sideways, so those lines never cross: press P to switch to lines at right angles, which cross at a swirl's centre.",
        "Set the marker's kind with Y, then test it. A marker of the wrong kind never matches the dashes, even in the right place.",
    ]);
    // Two of Them: two identical repellers are hidden together and the player has two markers. Seen from beyond their
    // separation two sources look like one in the middle, so a marker pair stacked at the midpoint fits the trails of
    // scouts that pass to the side (to within 0.14 of the real trail on every side-by-side pair measured); only a
    // scout that flies along the line between them, or passes close to one, tells "one here" from "two flanking it".
    // The first two attempts have the sources one above the other or on a diagonal (the stacked pair does not fit),
    // the rest side by side. Each pair is solvable with three paid scouts and no two pairs are within twice the
    // tolerance of each other (tested). The level is complete after four finds, so it reaches separation 2.8. The
    // guide lines are on, as in the roadmap's description of the trap ("the rays converge there"): the first version
    // switched them off, and a player found it too hard. Rays from a scout beside one source meet at that source,
    // from far away at the midpoint.
    let mut four = base(
        "sur-4",
        Chapter::Survey,
        "Two of Them",
        "Two sources are hidden, and from far away they can look like one. Mark both.",
        Field::new().fixed(uni(1.0, 0.0)),
        vec![],
        Rules::Survey {
            zone: zr(-5.5, -3.5, 5.5, 3.5),
            free: v2(-5.0, 0.0),
            scouts: 3,
            k: SURVEY4_K,
            sigma: SURVEY4_SIGMA,
            tol: SURVEY4_TOL,
            rays: true,
            band: false,
            preview: false,
            pool: SURVEY4_POOL.iter().map(|&(a, _)| v2(a.0, a.1)).collect(),
            kinds: vec![SourceKind::Repeller; SURVEY4_POOL.len()],
            partners: SURVEY4_POOL
                .iter()
                .map(|&(_, b)| Some(v2(b.0, b.1)))
                .collect(),
            goal: SurveyGoal::Finds(4),
        },
    );
    four.max_time = SURVEY_TRAIL;
    let four = hints(four, &[
        "From far away two sources can look like one in the middle. If both markers sitting on one spot fit the dashes, that may be what you are seeing.",
        "A scout that passes close to one source feels it almost alone, and one that flies along the line between them feels both in turn. Release one there.",
        "Test your markers. If no pair of markers on one spot matches the dashes, pull them apart until it does.",
    ]);
    // The Gyre: the charted background circulates (a counter-clockwise vortex with a slow inward drift), so a scout curves
    // as it flies and the question becomes where to release it so that its path will pass the source. One repeller, the
    // level 3 source; the band is back (one source, so silence is evidence again) and so is the free scout's lead only as
    // a graze, and a charted-path preview follows the cursor while aiming. No guide lines: reading the arrows is level 3.
    let mut five = base("sur-5", Chapter::Survey, "The Gyre",
        "The flow circulates, so a scout curves as it flies. Something hidden pushes it. Release scouts where their paths will pass it.",
        Field::new().fixed(vortex(0.0, 0.0, GYRE_K, GYRE_SOFT)).fixed(radial(0.0, 0.0, -GYRE_DRIFT, GYRE_SOFT)),
        vec![],
        Rules::Survey { zone: zr(-5.5, -3.5, 5.5, 3.5), free: v2(-4.0, 0.0), scouts: 2, k: SURVEY3_K, sigma: SURVEY3_SIGMA, tol: SURVEY3_TOL, rays: false, band: true, preview: true,
            pool: SURVEY5_POOL.iter().map(|&(x, y)| v2(x, y)).collect(), kinds: vec![SourceKind::Repeller; SURVEY5_POOL.len()], partners: vec![None; SURVEY5_POOL.len()], goal: SurveyGoal::Finds(3) });
    five.max_time = SURVEY_TRAIL;
    let five = hints(five, &[
        "The pale line that follows your cursor is where a scout released there would fly with nothing hidden. A scout only feels the source if that line passes within about 2 of it.",
        "The shaded ground is where a source would already have been felt. Release the next scout so its pale line crosses ground that is not shaded: the part of a line the free scout already flew will show you nothing new.",
        "The flow carries every scout round the same way, and the free scout stopped after five seconds. Its pale line from where it stopped shows where the flow takes a scout next.",
    ]);
    // The order is the teaching order: what the kinds look like (sur-2), where to send scouts (sur-3), then two sources
    // at once (sur-4), then a background that curves the scouts' paths (sur-5).
    vec![one, three, two, four, five]
}

pub fn combo_levels() -> Vec<Level> {
    let w = TAU / T12;
    vec![
        hints(base("cmb-1", Chapter::Combo, "Through the Gate", "Position + corridor: pass the gate, then reach the target.",
            Field::new().fixed(uni(0.7, 0.0)).fixed(vortex(0.0, 0.0, -2.5, 0.8)),
            vec![Region::target(4.0, -2.0, 0.6), Region::gate(0.0, 2.0, 0.4)],
            pos(vec![slot(zr(-5.5, -3.0, -4.0, 3.0), 0)])),
            &["The gate sits on a route you already know from Two Roads."]),
        hints(base("cmb-2", Chapter::Combo, "Early Clue", "Predict + partial visibility: use the revealed opening of the path.",
            Field::new().fixed(uni(0.9, 0.0)).fixed(vortex(1.0, 0.0, 3.0, 0.5)), right_exits(),
            pred(zr(-5.5, -3.0, -4.0, 3.0), Reveal::Partial(2.5))),
            &["The first seconds are shown. Extrapolate from the heading it has when it ends."]),
        hints(base("cmb-3", Chapter::Combo, "Dodge", "Tempo + constraint: reach the target without touching the hazard.",
            Field::new().fixed(uni(2.5, 0.0)).fixed(Term::Modulated { inner: Box::new(uni(0.0, 1.0)), offset: 0.0, amp: 1.0, omega: w, phase: 0.0 }),
            vec![Region::target(4.5, 2.6, 0.9), Region::hazard(0.5, 0.8, 0.7)],
            Rules::Tempo { start: v2(-4.5, 0.0), period: T12, goal: 0, dial: None }),
            &["Some launch times lift the probe straight through the hazard. Later ones pass above it."]),
        hints(base("cmb-4", Chapter::Combo, "Three Exits", "Experiment + multiple exits: find where exit C turns into exit D.",
            spiral(), ring_exits(),
            Rules::Experiment { start: v2(1.4, 0.0), range: (0.0, 3.0), challenge: Challenge::Threshold { from: 2, to: 3, tol: 0.15 } }),
            &["Both exits must be seen before you can report the change."]),
        hints(base("cmb-5", Chapter::Combo, "Right Place, Right Time", "Position + Tempo: choose where and when.",
            Field::new().fixed(uni(2.5, 0.0)).fixed(Term::Orbiting { kind: SwirlKind::Vortex, c: v2(0.0, 0.0), orbit: 2.0, omega: w, phase: 0.0, k: 3.0, soft: 0.6 }),
            vec![Region::target(4.5, 2.0, 0.7)],
            Rules::Position { slots: vec![slot(zr(-5.5, -1.0, -4.0, 1.0), 0)], period: Some(T12), intensity: None }),
            &["Both choices matter: a good place may need a different time than a poor one."]),
        hints(base("cmb-6", Chapter::Combo, "Shared Dial", "Two probes, one intensity: make both reach their targets.",
            Field::new().fixed(uni(0.75, 0.0)).tunable(vortex(-0.63, -0.94, 2.7, 0.8)),
            vec![Region::target(4.0, 3.02, 0.6), Region::target(4.0, -2.13, 0.6)],
            Rules::Position { slots: vec![slot(zr(-5.5, 0.3, -4.0, 1.33), 0), slot(zr(-5.5, -1.33, -4.0, -0.3), 1)], period: None, intensity: Some((0.0, 3.0)) }),
            &["One dial bends both paths at once. A setting that suits the top probe can strand the bottom one.", "Try a dial value first, then look for a release point in each zone."]),
        // cmb-7 Shape + Tempo. The background pulses sideways; a source is steady. The pulse lifts the probe but never
        // high enough, and only some launch times leave the probe where one source can finish the job. The pulse's
        // phase is set so the default launch (0 s) is in the dead part of the cycle (see docs/07-combinations.md).
        hints(combo_shape("cmb-7", "Source on the Tide", "Shape + Tempo: the flow pulses. Choose when to launch and where to put one source.",
            Field::new().fixed(uni(1.0, 0.0)).fixed(Term::Modulated { inner: Box::new(uni(0.0, 1.0)), offset: 0.0, amp: 0.6, omega: w, phase: w * 4.25 }),
            vec![Region::target(4.5, 2.7, 0.6), Region::hazard(-0.5, 0.0, 0.6)],
            Rules::Shape { start: SHAPE_START, goal: 0, zones: vec![zr(-4.0, -3.0, 3.0, 3.0)], kinds: vec![SourceKind::Repeller, SourceKind::Attractor], sources: 1, k: SHAPE_K, sigma: SHAPE_SIGMA, period: Some(T12), intensity: None }),
            &[
                "Launch with nothing placed and scrub the timeline: the pulse lifts the probe at some times and not at others, but never onto the target.",
                "A source does not pulse. It pushes the same way whenever the probe passes it, so the same source can help at one launch time and be useless at another.",
                "Find a launch time that gets the probe close, then place the source where it adds the rest. Moving the launch time moves where the probe is when it reaches the source.",
            ]),
        // cmb-8 Shape + dial. The dial scales a strong vortex in the background. At the right strength it carries the
        // probe round the big obstacle, a job no local source can do (the placement area starts downstream of it);
        // the source then finishes the approach. At the default dial the vortex is too strong.
        hints(combo_shape("cmb-8", "Compensate", "Shape + dial: set the strength of the current, then place one source to finish the approach.",
            Field::new().fixed(uni(1.0, 0.0)).tunable(vortex(-0.7, 2.5, -3.6, 0.8)),
            vec![Region::target(4.5, 0.9, 0.6), Region::hazard(-0.15, 0.45, 1.25)],
            Rules::Shape { start: SHAPE_START, goal: 0, zones: vec![zr(0.0, -3.0, 3.5, 3.0)], kinds: vec![SourceKind::Repeller, SourceKind::Attractor], sources: 1, k: SHAPE_K, sigma: SHAPE_SIGMA, period: None, intensity: Some((0.0, 3.0)) }),
            &[
                "Launch with nothing placed at a few dial settings. The big obstacle sits on the stream: the current has to carry the probe round it.",
                "A source only acts near where you put it, downstream of the obstacle. It cannot carry the probe round the obstacle; the dial has to.",
                "Too weak and the probe meets the obstacle; too strong and it is flung past the target by more than one source can undo. Find the band between, then place the source.",
            ]),
        // cmb-9 Shape + Synchronize. Each probe reaches the target only in certain release windows, and with no source
        // the two arrival windows never come within the rendezvous tolerance of each other. One source, placed above the
        // lane, reshapes one probe's route and moves its window; the delays then have to line the two arrivals up.
        hints(combo_sync("cmb-9", "Make Them Meet", "Shape + Synchronize: place one source, then time both probes to arrive together.",
            Field::new().fixed(uni(1.90, 0.0)).fixed(mgauss(-0.08, -1.67, 5.43)).fixed(mgauss(0.97, 1.85, 1.47)),
            vec![Region::target(4.5, 0.0, 0.9)],
            Rules::Synchronize { starts: vec![v2(-5.0, -1.46), v2(-2.69, 1.13)], goals: vec![0, 0], period: T12, max_delay: T12, conds: vec![SyncCond::meet(0, 1, 0, -0.6, 0.6)], start_slots: None,
                sources: Some(SteerSources { zones: vec![zr(-4.0, 0.3, 3.0, 3.0)], kinds: vec![SourceKind::Repeller, SourceKind::Attractor], max: 1, k: SHAPE_K, sigma: SHAPE_SIGMA }),
 dial: None,
 }),
            &[
                "Launch both probes with no source and read the timeline. Each can reach the target at some release times, but their arrival times never come within the allowed band of each other.",
                "A source changes the route of any probe that passes near it, and so changes when that probe arrives. It does not pulse: it pushes the same way at every release time.",
                "Place the source first, then try release times. A source that moves one probe's arrival into reach of the other's is the start; the delays decide the rest.",
            ]),
        // cmb-10 Synchronize + Position. Probe B is released from one of five numbered slots; the delays are free. At
        // the default slot the two arrival windows never come within the rendezvous band whatever the delays, and no
        // pair of delays near a simultaneous release works at any slot, so the slot and the delays are both
        // necessary. The choice of slot is categorical (docs/07-combinations.md: a start line failed because one
        // window of the delays always reached a neighbouring start). Found by a random search (docs/07-combinations.md).
        hints(combo_sync("cmb-10", "Route and Rendezvous", "Synchronize + Position: choose where probe B starts, then time both probes to arrive together.",
            Field::new().fixed(uni(1.26, -0.16)).fixed(mgauss(-2.58, -0.68, 5.47)).fixed(mgauss(-1.06, -1.1, 0.2)),
            vec![Region::target(4.5, 0.25, 0.9)],
            Rules::Synchronize { starts: vec![v2(-5.0, 1.05), v2(-4.58, -2.14)], goals: vec![0, 0], period: T12, max_delay: T12, conds: vec![SyncCond::meet(0, 1, 0, -0.6, 0.6)], sources: None,
                start_slots: Some((1, vec![v2(-4.58, -2.14), v2(-2.18, 1.09), v2(-3.05, 0.51), v2(-2.52, -1.15), v2(-4.83, -1.2)])),
 dial: None,
 }),
            &[
                "Probe B can start from any of the numbered markers. Launch both probes from the first marker and read the timeline: B can reach the target at some release times, but never within the allowed band of A's arrival, whatever you do with the delays.",
                "Each start gives B a different route through the pulsing field, and so different release times at which it arrives, and different arrival times. Try the other markers and see which of them puts B's arrival into reach of A's.",
                "Two of the five starts can work. Choose one, then use the delays to line the arrivals up; releasing both together does not work from any start.",
            ]),
        // cmb-23 Synchronize + Experiment. The dial scales the tunable layers (two pulsing lanes and a sideways drift) and
        // is set before the delays. At the default setting the two probes' arrival windows never come within the
        // rendezvous band whatever the delays, and no near-simultaneous release works at any setting, so the dial and
        // the delays are both necessary. Of thirteen settings three adjacent ones admit a win. Found by a random
        // search with the protocol as the filter (docs/07-combinations.md).
        hints(combo_sync("cmb-23", "Dial and Rendezvous", "Synchronize + Experiment: set the dial, then time both probes to arrive together.",
            Field::new().fixed(uni(1.41, 0.05)).tunable(mgauss(2.33, 0.89, 5.77)).tunable(mgauss(-2.71, -1.6, 4.88)).tunable(uni(-0.43, 0.34)),
            vec![Region::target(4.5, 0.85, 0.9)],
            Rules::Synchronize { starts: vec![v2(-5.0, -1.28), v2(-5.0, 0.08)], goals: vec![0, 0], period: T12, max_delay: T12, conds: vec![SyncCond::meet(0, 1, 0, -0.6, 0.6)], sources: None, start_slots: None, dial: Some(dial13()) }),
            &[
                "The dial scales only part of the field: the two pulsing lanes and a sideways drift. Launch both probes at the first setting and read the timeline: with this setting they can never be made to arrive together, whatever the delays.",
                "Each setting changes when and where each probe reaches the target, so it changes the times at which they can arrive. Try the other settings, and watch which of them brings the two arrival windows into reach of each other.",
                "Three neighbouring settings work. Choose one, then use the delays to line the arrivals up; releasing both together does not work at any setting.",
            ]),
        // cmb-24 Tempo + Experiment. The dial scales two pulsing sideways pushes (the steady stream ignores it) and is
        // set before the launch phase. At the default setting no launch phase brings the probe to the target, and at no
        // setting does launching at phase 0 or within 1.5 s of it, so the dial and the phase are both necessary. Of five
        // settings two admit a window of launch phases. Found by a random search (docs/07-combinations.md).
        hints(base("cmb-24", Chapter::Combo, "Dial and Time It", "Tempo + Experiment: set the dial, then choose when to launch.",
            Field::new().fixed(uni(2.45, 0.03)).tunable(Term::Modulated { inner: Box::new(uni(0.0, 0.77)), offset: 0.0, amp: 0.92, omega: w, phase: 1.02 }).tunable(Term::Modulated { inner: Box::new(uni(-0.42, -0.77)), offset: 0.0, amp: 0.66, omega: w, phase: 0.6 }),
            vec![Region::target(4.5, -1.93, 0.7)],
            Rules::Tempo { start: v2(-4.5, 0.0), period: T12, goal: 0, dial: Some(vec![1.0, 0.0, 0.5, 1.5, 2.0]) }),
            &[
                "The dial scales only the two pulsing pushes; the steady stream ignores it. At the first setting there is no launch time that reaches the target: scrub the whole cycle and see.",
                "Each setting changes how far the pulses push the probe, so it changes which part of the cycle carries it to the target. Try the other settings and scrub the clock again to find where the probe gets through.",
                "Two of the five settings work. Launching at phase 0, or close to it, never works at any setting.",
            ]),
        // cmb-28 Steer + Synchronize. A drone flies the unsteered pulsing field by itself, released at one of 25 delays
        // (0 to 12 s in half seconds); the steered probe has one strong gust and must reach the target within 0.6 s of
        // the drone's arrival. The drone reaches the target only at some delays, and the gust cannot move the steered
        // probe's arrival far, so only a few adjacent releases give a meeting: none works with no gust or at the
        // default release. Found by a random search with the protocol as the filter (docs/07-combinations.md).
        hints(base("cmb-28", Chapter::Combo, "Meet the Drone", "Steer + Synchronize: release the drone, then steer to arrive with it.",
            Field::new().fixed(uni(1.14, 0.2)).fixed(Term::Modulated { inner: Box::new(uni(0.0, 0.84)), offset: 0.0, amp: 0.95, omega: w, phase: 6.14 }),
            vec![Region::target(4.4, 0.5, 0.6)],
            Rules::Steer { start: v2(-5.0, -2.38), beats: 10, gust_budget: 1, gust_strength: 2.0, dirs: GustDirs::Eight, beat_time: 1.0, goal: 0, period: None, start_zone: None, sources: None, dial: None, companion: Some(Companion { start: v2(-5.0, -2.41), delays: (0..=24).map(|d| d as f64 * 0.5).collect(), band: 0.6 }) }),
            &[
                "The drone (A) flies on its own and its path and arrival time are shown for the release you choose. At some releases it never reaches the target: choose another.",
                "Your probe needs its gust to reach the target at all, and the gust shifts when it arrives only a little. Compare when the drone arrives with when your probe would arrive with no gust, and pick the release that brings the two closest.",
                "The release cannot be changed once the first beat is played. Spend the gust on the beat that brings your probe to the target at the drone's time.",
            ]),
        // cmb-22 Steer + Experiment. The dial scales the tunable layers of the field and is set before the first beat;
        // the single gust is too little to reach the target at the default dial, and the right setting lets the field
        // do most of the work. Of thirteen settings three adjacent ones admit a winning gust, and no gust-free run wins
        // at any setting. Found by a random search with the protocol as the filter (docs/07-combinations.md).
        hints(base("cmb-22", Chapter::Combo, "Set the Dial, Then Steer", "Steer + Experiment: set the dial, then steer with one gust.",
            Field::new().fixed(uni(1.02, -0.26)).tunable(uni(-0.22, 0.39)).tunable(vortex(2.17, -1.32, -1.6, 0.8)),
            vec![Region::target(4.4, -0.47, 0.7)],
            Rules::Steer { start: v2(-5.0, 0.0), beats: 8, gust_budget: 1, gust_strength: 1.0, dirs: GustDirs::Eight, beat_time: 1.0, goal: 0, period: None, start_zone: None, sources: None, dial: Some(dial13()), companion: None }),
            &[
                "One gust is not enough to reach the target at the dial setting you start with. Try the other settings and watch how the arrows change: the dial scales only part of the field.",
                "Pick the setting whose flow carries the probe closest to the target on its own (a gust-free run shows it), then spend the gust where the field still falls short. The dial cannot be changed once the first beat is played.",
            ]),
        // cmb-11 Steer + Tempo. The flow pulses sideways and two gusts are too few to lift the probe to the target
        // from the default launch; the right launch time lets the pulse do most of the lifting.
        hints(base("cmb-11", Chapter::Combo, "Catch the Gust", "Steer + Tempo: choose when to start, then steer with two gusts.",
            Field::new().fixed(uni(1.2, 0.0)).fixed(Term::Modulated { inner: Box::new(uni(0.0, 1.0)), offset: 0.0, amp: 0.6, omega: w, phase: w * 5.0 }),
            vec![Region::target(4.4, 3.1, 0.5)],
            Rules::Steer { start: v2(-5.0, 0.0), beats: 8, gust_budget: 2, gust_strength: 1.0, dirs: GustDirs::Eight, beat_time: 1.0, goal: 0, period: Some(T12), start_zone: None, sources: None, dial: None, companion: None }),
            &[
                "Two gusts are not enough to reach the target from the start you are given. Scrub the start time and watch which way the pulse pushes the probe.",
                "Start when the pulse carries the probe up for most of the run, then use the gusts for the rest. The start time cannot be changed once the first beat is played.",
            ]),
        // cmb-12 Steer + Position. The start point is a choice on a line. Two gusts cannot lift the probe far enough
        // from the default start, and starting too high puts the probe in the way of the upper obstacle.
        hints(base("cmb-12", Chapter::Combo, "Fewer Gusts", "Steer + Position: choose where to start, then steer with two gusts.",
            Field::new().fixed(uni(1.15, 0.0)).fixed(vortex(-2.7, 0.0, -2.1, 0.8)),
            vec![Region::target(4.0, 2.85, 0.5), Region::hazard(-0.15, 3.35, 0.9)],
            Rules::Steer { start: v2(-5.0, 0.0), beats: 8, gust_budget: 2, gust_strength: 1.0, dirs: GustDirs::Eight, beat_time: 1.0, goal: 0, period: None, start_zone: Some(Rect::new(-5.0, -3.0, -5.0, 3.0)), sources: None, dial: None, companion: None }),
            &[
                "Two gusts are not enough from the start you are given. Click on the start line to move where the probe begins, before the first beat.",
                "A higher start saves gusts, but the obstacle above the lane punishes a start that is too high. Look for the band between.",
            ]),
        // cmb-13 Steer + Shape. One gust is not enough to bring the probe down to the target past the obstacle. A source
        // placed first changes the flow along the whole run, so the gust only has to finish the job.
        hints(base("cmb-13", Chapter::Combo, "Pave the Way", "Steer + Shape: place one source first, then steer with a single gust.",
            Field::new().fixed(uni(1.12, 0.0)).fixed(vortex(-2.0, 0.97, 1.05, 0.8)),
            vec![Region::target(3.76, -2.05, 0.5), Region::hazard(-0.94, -0.9, 0.6)],
            Rules::Steer { start: v2(-5.0, 0.0), beats: 8, gust_budget: 1, gust_strength: 1.0, dirs: GustDirs::Eight, beat_time: 1.0, goal: 0, period: None, start_zone: None,
                sources: Some(SteerSources { zones: vec![zr(-4.0, -3.0, 2.5, 3.0)], kinds: vec![SourceKind::Repeller, SourceKind::Attractor], max: 1, k: SHAPE_K, sigma: SHAPE_SIGMA }),
                    dial: None,
 companion: None,
 }),
            &[
                "With no source, play the run and watch where the field takes the probe: one gust cannot bring it to the target.",
                "A source acts the whole way along the run, not just on one beat. Place it where it bends the flow toward the target and away from the obstacle.",
                "You cannot change the source once the first beat is played. Place it, then spend the single gust where the field still falls short.",
            ]),
        // cmb-14 Survey + Position, scout then release. The attempt ends with a *choice of release slot*, not a free
        // start height: a start height needs a precision a person cannot read off scout trails and, with wide
        // windows, one start serves several hidden sources (docs/07-combinations.md). Seven numbered slots, each
        // hidden source has its own winning slots and no slot wins for two of them. Found by a random search (docs/07-combinations.md).
        hints(combo_scout("cmb-14", "Scout, Then Release", "Survey + Position: scout the hidden source, then choose where to release the probe.",
            Field::new().fixed(uni(1.22, 0.11)).fixed(vortex(-0.75, -2.3, 1.9, 0.8)),
            vec![Region::target(4.5, 1.25, 0.8)],
            Rules::Scout(ScoutRules {
                act: ScoutAct::Release { starts: [-2.52, -1.74, -0.86, 0.12, 0.88, 1.57, 2.51].iter().map(|&y| v2(-5.0, y)).collect(), goal: 0 },
                start: v2(-5.0, 0.0),
                exits: vec![],
                scouts: 3,
                zone: Rect::new(-5.6, -3.0, -4.4, 3.0),
                scout_time: 6.0,
                pool: vec![(SourceKind::Attractor, v2(-1.4, 2.48)), (SourceKind::Attractor, v2(-2.28, -1.02)), (SourceKind::Attractor, v2(-0.57, -2.33))],
                k: 2.01,
                sigma: 0.8,
                streak: 3,
            })),
            &[
                "The probe can be released from any of the numbered diamonds, but only some of them reach the target, and which ones depends on the hidden source. Scouts are optional, but the source changes every attempt.",
                "A scout starts in the shaded strip and shows you what the source does to a flight: the yellow arrows are its push. Work out where the source is and which way it bends the flow the probe will ride.",
                "Release from a slot on the side the source would push the probe away from, so that it is bent back onto the target. Three right choices in a row.",
            ]),
        // cmb-15 Survey + Shape, scout then cancel. The attempt ends with a *choice of source to add* from eight
        // numbered options (a repeller or an attractor at each of four places), not a free placement: a free
        // placement needs a precision a person cannot read off scout trails. A hidden source sits at three of the
        // four places; the source of the opposite kind at the same place cancels it.
        hints(combo_scout("cmb-15", "Cancel It", "Survey + Shape: scout the hidden source, then choose a source to add that cancels it.",
            Field::new().fixed(uni(1.06, -0.05)).fixed(vortex(1.78, 0.94, -0.86, 0.8)),
            vec![Region::target(4.5, 0.11, 0.9)],
            Rules::Scout(ScoutRules {
                act: ScoutAct::Cancel {
                    sites: vec![
                        (SourceKind::Repeller, v2(-2.17, 1.26)), (SourceKind::Repeller, v2(-0.74, -0.26)), (SourceKind::Repeller, v2(0.59, 0.63)), (SourceKind::Repeller, v2(2.22, 0.56)),
                        (SourceKind::Attractor, v2(-2.17, 1.26)), (SourceKind::Attractor, v2(-0.74, -0.26)), (SourceKind::Attractor, v2(0.59, 0.63)), (SourceKind::Attractor, v2(2.22, 0.56)),
                    ],
                    goal: 0,
                },
                start: v2(-5.0, 0.0),
                exits: vec![],
                scouts: 3,
                zone: Rect::new(-5.6, -3.0, -4.4, 3.0),
                scout_time: 6.0,
                pool: vec![(SourceKind::Attractor, v2(-2.17, 1.26)), (SourceKind::Repeller, v2(0.59, 0.63)), (SourceKind::Attractor, v2(2.22, 0.56))],
                k: 2.01,
                sigma: 0.8,
                streak: 3,
            })),
            &[
                "A hidden source bends the probe off the target. You may add one source of your own: pick one of the eight marked options (orange plus: a repeller, blue minus: an attractor). The same kind of source in the same place would double the push.",
                "Scout the hidden source and read its kind from the yellow arrows: they point away from a repeller and toward an attractor. Then add the opposite kind in the same place, so that the two push cancel.",
                "The hidden source is in one of four places and changes every attempt. Three right choices in a row.",
            ]),
        // cmb-16 Survey + Predict, the first scout-then-act level. A hidden source (a different one each attempt) bends
        // every flight. Scouts feel it; the main probe's exit is called before it flies. The three hidden sources send
        // the probe to three different exits, so no fixed call can pass three in a row (see validate_scout.rs).
        hints(combo_scout("cmb-16", "Scout, Then Call It", "Survey + Predict: scout the hidden source, then call which exit the probe takes.",
            Field::new().fixed(uni(1.08, 0.09)),
            vec![Region::new(RegionKind::Target, 4.5, 2.7, 1.0, 'A'), Region::new(RegionKind::Target, 4.5, 0.9, 1.0, 'B'), Region::new(RegionKind::Target, 4.5, -0.9, 1.0, 'C'), Region::new(RegionKind::Target, 4.5, -2.7, 1.0, 'D')],
            Rules::Scout(ScoutRules {
                act: ScoutAct::Exit,
                start: v2(-5.0, 0.0),
                exits: vec![0, 1, 2, 3],
                scouts: 3,
                zone: Rect::new(-5.6, -3.0, -4.4, 3.0),
                scout_time: 6.0,
                pool: vec![(SourceKind::Repeller, v2(-0.38, -1.43)), (SourceKind::Attractor, v2(-2.06, 1.64)), (SourceKind::Attractor, v2(0.91, -0.68))],
                k: 2.01,
                sigma: 0.8,
                streak: 3,
            })),
            &[
                "The arrows show the background only. A hidden source is also bending every flight, yours included. Launch scouts from the strip on the left and read the small push arrows: they show what the hidden source did to each scout.",
                "A push points along the line to the source: away from it for a repeller, toward it for an attractor. Two scouts that felt it are enough to see where those lines cross, and the direction tells you the kind.",
                "Once you know where it is and which kind, picture the main probe (the diamond) flying through the same field, then call the exit. The hidden source is different every attempt, so a call that worked once will not work again: three right calls in a row.",
            ]),
        // cmb-17 Survey + Steer, scout then steer. The attempt ends with a *choice of gust plan* from four fixed
        // plans (a gust direction at chosen beats, at most two gusts, six beats), not a free sequence of gusts: a
        // closed-loop steer would show the probe's own flight and make the scouting unnecessary. Each hidden source
        // has exactly one plan that brings the probe to the target and no plan wins for two. Found by
        // a random search (the plans are the robust exclusive winners of the hidden sources, plus a decoy).
        hints(combo_scout("cmb-17", "Scout, Then Steer", "Survey + Steer: scout the hidden source, then choose the gust plan that gets the probe to the target.",
            Field::new().fixed(uni(1.14, 0.13)).fixed(vortex(-1.45, 1.5, 2.85, 0.8)),
            vec![Region::target(4.5, 1.82, 1.0)],
            Rules::Scout(ScoutRules {
                act: ScoutAct::Plan {
                    plans: vec![
                        vec![None, Some(3), None, None, None, None],
                        vec![None, Some(6), None, None, Some(2), None],
                        vec![Some(1), None, None, None, None, None],
                        vec![None, None, None, Some(6), None, None],
                    ],
                    goal: 0,
                    dirs: GustDirs::Eight,
                    strength: 1.0,
                    beat_time: 1.0,
                },
                start: v2(-5.0, 0.0),
                exits: vec![],
                scouts: 3,
                zone: Rect::new(-5.6, -3.0, -4.4, 3.0),
                scout_time: 6.0,
                pool: vec![(SourceKind::Attractor, v2(0.48, -1.83)), (SourceKind::Attractor, v2(0.52, 1.24)), (SourceKind::Repeller, v2(1.71, -1.47))],
                k: 3.0,
                sigma: 0.8,
                streak: 3,
            })),
            &[
                "Four gust plans are on offer. Each gusts in a fixed direction at fixed beats; the probe gets six beats of one second to reach the target. Only one plan suits each hidden source.",
                "The hidden source changes every attempt. A scout shows you where it is and which kind it is (the yellow arrows point away from a repeller and toward an attractor), and so which way it will push the probe.",
                "Read each plan as a push in a direction at a time, and pick the one that cancels the source's effect on the probe. Three right choices in a row.",
            ]),
        // cmb-18 Survey + Predict again, on a curved background. The visible field is a stream plus a swirl, so a
        // scout's path is bent by the swirl as well as by the hidden source, and the push has to be separated from the
        // swirl (the push is observed velocity minus the visible field's, so the arrows already do that). The three
        // hidden sources again send the probe to three different exits (found by random search, see
        // docs/07-combinations.md).
        hints(combo_scout("cmb-18", "Scout Through the Swirl", "Survey + Predict: the background curves every flight. Scout the hidden source, then call the exit.",
            Field::new().fixed(uni(1.29, -0.14)).fixed(vortex(-0.81, -0.8, -2.54, 0.8)),
            vec![Region::new(RegionKind::Target, 4.5, 2.7, 1.0, 'A'), Region::new(RegionKind::Target, 4.5, 0.9, 1.0, 'B'), Region::new(RegionKind::Target, 4.5, -0.9, 1.0, 'C'), Region::new(RegionKind::Target, 4.5, -2.7, 1.0, 'D')],
            Rules::Scout(ScoutRules {
                act: ScoutAct::Exit,
                start: v2(-5.0, 0.0),
                exits: vec![0, 1, 2, 3],
                scouts: 3,
                zone: Rect::new(-5.6, -3.0, -4.4, 3.0),
                scout_time: 6.0,
                pool: vec![(SourceKind::Attractor, v2(1.1, -0.58)), (SourceKind::Repeller, v2(1.16, -1.87)), (SourceKind::Repeller, v2(-1.4, 0.55))],
                k: 2.01,
                sigma: 0.8,
                streak: 3,
            })),
            &[
                "This background is not a straight stream: a swirl bends every flight. The small push arrows already subtract the swirl, so they show only what the hidden source did.",
                "The swirl also bends the main probe, so do not call the exit from a straight line. Locate the source from the push arrows, then picture the probe flying through the swirl and the source together.",
                "The hidden source is different every attempt: three right calls in a row.",
            ]),
        // cmb-19 Survey + Tempo, scout then time it. The background pulses, so when the probe is released decides how
        // the pulse meets the hidden source's push; the attempt ends with a *choice of launch phase* among six (0, 2
        // ... 10 s of a 12 s period), the default being phase 0. Each hidden source has its own winning phase, the
        // default phase wins for none, and no phase wins for two. Found by a random search (docs/07-combinations.md).
        hints(combo_scout("cmb-19", "Scout, Then Time It", "Survey + Tempo: scout the hidden source, then choose when to launch the probe.",
            Field::new().fixed(uni(1.13, 0.2)).fixed(Term::Modulated { inner: Box::new(uni(0.0, 0.88)), offset: 0.0, amp: 0.8, omega: TAU / T12, phase: 0.0 }),
            vec![Region::target(4.5, -0.29, 1.0)],
            Rules::Scout(ScoutRules {
                act: ScoutAct::Phase { phases: vec![0.0, 2.0, 4.0, 6.0, 8.0, 10.0], goal: 0 },
                start: v2(-5.0, 0.0),
                exits: vec![],
                scouts: 3,
                zone: Rect::new(-5.6, -3.0, -4.4, 3.0),
                scout_time: 6.0,
                pool: vec![(SourceKind::Attractor, v2(-1.84, 2.21)), (SourceKind::Attractor, v2(-1.81, 0.61)), (SourceKind::Attractor, v2(-2.68, -2.06))],
                k: 2.01,
                sigma: 0.8,
                streak: 3,
            })),
            &[
                "The background pulses with a twelve-second rhythm, and the probe is released at the phase you choose: 1 is phase 0, 2 is 2 s into the rhythm, and so on. Launching at phase 0 never works, whatever the hidden source.",
                "Scouts fly from phase 0 and show you the hidden source (yellow arrows point toward an attractor). Work out which way it will pull the probe, then think about which part of the pulse leaves the probe where the source's pull can finish the job.",
                "The hidden source changes every attempt, and each one has its own right phase. Three right choices in a row.",
            ]),
        // cmb-29 Survey + Tempo II. Like cmb-19 (launch phase after scouting a hidden source in a pulsing field), but the
        // sources are of both kinds and the phases are 24, half a second apart, so a source's window is several phases
        // wide. The reader that guesses the wrong kind is right for at most one source in three. Found by selection:
        // for each sampled field the winning phases of every candidate source were computed, and three with wide,
        // disjoint windows and both kinds were taken (docs/07-combinations.md).
        hints(combo_scout("cmb-29", "Scout, Then Time It II", "Survey + Tempo: scout the hidden source (an attractor or a repeller), then choose when to launch the probe.",
            Field::new().fixed(uni(1.37, 0.2)).fixed(Term::Modulated { inner: Box::new(uni(0.0, 1.14)), offset: 0.0, amp: 0.66, omega: TAU / T12, phase: 3.09 }),
            vec![Region::target(4.5, -0.66, 1.0)],
            Rules::Scout(ScoutRules {
                act: ScoutAct::Phase { phases: (0..24).map(|k| k as f64 * 0.5).collect(), goal: 0 },
                start: v2(-5.0, 0.0),
                exits: vec![],
                scouts: 3,
                zone: Rect::new(-5.6, -3.0, -4.4, 3.0),
                scout_time: 6.0,
                pool: vec![(SourceKind::Repeller, v2(-2.0, 1.0)), (SourceKind::Repeller, v2(2.0, -2.0)), (SourceKind::Attractor, v2(-3.0, 0.0))],
                k: 3.0,
                sigma: 0.8,
                streak: 3,
            })),
            &[
                "The background pulses with a twelve-second rhythm, and the probe is released at the phase you choose: 1 is phase 0, 2 is half a second into the rhythm, and so on. Launching at phase 0 never works, whatever the hidden source.",
                "Scouts fly from phase 0 and show you the hidden source. The yellow arrows point toward an attractor and away from a repeller, and the kind changes which way the source pushes the probe, so it changes which part of the pulse can make up for it.",
                "The hidden source changes every attempt, and each one has its own stretch of good phases, several launches wide. Aim for the middle of it. Three right choices in a row.",
            ]),
        // cmb-20 Survey + Shape + Position, the first level with three decisions: scout, then add a source and
        // choose a release slot, judged together by one flight. The hidden source sits on one of four places; the
        // opposite kind there cancels it (as in cmb-15), after which the probe still has to be released from one of the
        // slots that reach the target through the visible swirl (the default slot, far off the lane, never does).
        // Each control is needed: no cancel (none) and the default slot win for no hidden source, and no hidden
        // source shares a winning cancel with another. Found by a random search (docs/07-combinations.md).
        hints(combo_scout("cmb-20", "Cancel and Launch", "Survey + Shape + Position: scout the hidden source, add a source to cancel it, and choose where to release the probe.",
            Field::new().fixed(uni(1.2, 0.21)).fixed(vortex(0.94, -1.43, 0.46, 0.8)),
            vec![Region::target(4.5, 1.63, 0.9)],
            Rules::Scout(ScoutRules {
                act: ScoutAct::Seq(vec![
                    ScoutAct::Cancel { sites: vec![(SourceKind::Repeller, v2(-2.29, 1.2)), (SourceKind::Repeller, v2(-0.88, 0.28)), (SourceKind::Repeller, v2(0.61, -1.56)), (SourceKind::Repeller, v2(1.96, 1.4)), (SourceKind::Attractor, v2(-2.29, 1.2)), (SourceKind::Attractor, v2(-0.88, 0.28)), (SourceKind::Attractor, v2(0.61, -1.56)), (SourceKind::Attractor, v2(1.96, 1.4))], goal: 0 },
                    ScoutAct::Release { starts: [-3.3, -2.47, -1.65, -0.71, 0.07, 0.77, 1.59, 2.36].iter().map(|&y| v2(-5.0, y)).collect(), goal: 0 },
                ]),
                start: v2(-5.0, 0.0),
                exits: vec![],
                scouts: 3,
                zone: Rect::new(-5.6, -3.0, -4.4, 3.0),
                scout_time: 6.0,
                pool: vec![(SourceKind::Attractor, v2(-2.29, 1.2)), (SourceKind::Repeller, v2(-0.88, 0.28)), (SourceKind::Attractor, v2(1.96, 1.4))],
                k: 3.0,
                sigma: 0.8,
                streak: 3,
            })),
            &[
                "Two groups of choices: a source to add (orange plus: a repeller, blue minus: an attractor, or none) and a release slot (numbered diamonds). Commit them together. The first source and the first slot are the defaults and are never enough.",
                "Scout the hidden source and read its kind from the yellow arrows, then add the opposite kind in the same place so that the two push cancel. That alone does not get the probe to the target.",
                "With the hidden push gone, the swirl and the stream still decide where the probe ends up. Release it from a slot that lets them carry it onto the target; fly a scout from a slot to see. Three right commits in a row.",
            ]),
        // cmb-21 Survey + Shape + Steer, built like cmb-20: scout, then add a source (or none) and choose a gust
        // plan (or none), judged together by one flight. The opposite kind at the hidden source's place cancels it
        // exactly (as in cmb-15); the visible swirl then still leaves the probe off the target, and a gust fixes
        // that. No cancel (none) and no gust (none) each win for no hidden source. Found by
        // a random search: 1 design in 11,600 samples passed everything.
        hints(combo_scout("cmb-21", "Cancel and Steer", "Survey + Shape + Steer: scout the hidden source, add a source to cancel it, and choose the gust plan that finishes the job.",
            Field::new().fixed(uni(1.0, 0.23)).fixed(vortex(-0.35, 1.03, 2.86, 0.8)),
            vec![Region::target(4.5, 0.27, 0.8)],
            Rules::Scout(ScoutRules {
                act: ScoutAct::Seq(vec![
                    ScoutAct::Cancel { sites: vec![(SourceKind::Repeller, v2(-2.13, -1.45)), (SourceKind::Repeller, v2(-1.0, -1.34)), (SourceKind::Repeller, v2(0.48, -1.51)), (SourceKind::Repeller, v2(2.33, 0.0)), (SourceKind::Attractor, v2(-2.13, -1.45)), (SourceKind::Attractor, v2(-1.0, -1.34)), (SourceKind::Attractor, v2(0.48, -1.51)), (SourceKind::Attractor, v2(2.33, 0.0))], goal: 0 },
                    ScoutAct::Plan {
                        plans: vec![
                            vec![None, None, Some(2), None, None, None],
                            vec![Some(0), None, None, None, None, None],
                            vec![None, None, None, Some(7), None, None],
                            vec![Some(6), None, None, None, None, None],
                        ],
                        goal: 0,
                        dirs: GustDirs::Eight,
                        strength: 1.0,
                        beat_time: 1.0,
                    },
                ]),
                start: v2(-5.0, 0.0),
                exits: vec![],
                scouts: 3,
                zone: Rect::new(-5.6, -3.0, -4.4, 3.0),
                scout_time: 6.0,
                pool: vec![(SourceKind::Repeller, v2(-2.13, -1.45)), (SourceKind::Repeller, v2(0.48, -1.51)), (SourceKind::Attractor, v2(2.33, 0.0))],
                k: 3.0,
                sigma: 0.8,
                streak: 3,
            })),
            &[
                "Two groups of choices: a source to add (orange plus: a repeller, blue minus: an attractor, or none) and a gust plan (or none). Commit them together. Neither none is ever enough.",
                "Scout the hidden source, read its kind from the yellow arrows, and add the opposite kind in the same place to cancel it. That alone leaves the probe off the target.",
                "With the hidden push gone, the swirl and the stream still carry the probe past the target. Of the four gust plans, some finish the job. Fly a scout to see where the visible field takes a probe, then pick the plan that corrects it. Three right commits in a row.",
            ]),
        // cmb-25 Survey + Shape + Experiment, built like cmb-20 and cmb-21: scout, then add a source (or none) and choose
        // a dial setting, judged together by one flight. The opposite kind at the hidden source's place cancels it
        // exactly (as in cmb-15), after which the probe flies the visible field, whose tunable drift the dial scales;
        // only some settings bring it onto the target (the default does not). No cancel (none) and the default setting
        // each win for no hidden source. Found by a random search (docs/07-combinations.md).
        hints(combo_scout("cmb-25", "Cancel and Dial", "Survey + Shape + Experiment: scout the hidden source, add a source to cancel it, and set the dial that brings the probe to the target.",
            Field::new().fixed(uni(1.16, 0.15)).tunable(uni(-0.21, -0.38)).tunable(vortex(0.09, 0.08, 0.13, 0.8)),
            vec![Region::target(4.5, -0.68, 1.0)],
            Rules::Scout(ScoutRules {
                act: ScoutAct::Seq(vec![
                    ScoutAct::Cancel { sites: vec![(SourceKind::Repeller, v2(-2.33, -1.31)), (SourceKind::Repeller, v2(-1.06, 0.73)), (SourceKind::Repeller, v2(0.82, 0.11)), (SourceKind::Repeller, v2(1.89, -1.74)), (SourceKind::Attractor, v2(-2.33, -1.31)), (SourceKind::Attractor, v2(-1.06, 0.73)), (SourceKind::Attractor, v2(0.82, 0.11)), (SourceKind::Attractor, v2(1.89, -1.74))], goal: 0 },
                    ScoutAct::Dial { levels: vec![1.0, 0.5, 0.75, 1.25, 1.5, 2.0], goal: 0 },
                ]),
                start: v2(-5.0, 0.0),
                exits: vec![],
                scouts: 3,
                zone: Rect::new(-5.6, -3.0, -4.4, 3.0),
                scout_time: 6.0,
                pool: vec![(SourceKind::Attractor, v2(-1.06, 0.73)), (SourceKind::Repeller, v2(0.82, 0.11)), (SourceKind::Attractor, v2(1.89, -1.74))],
                k: 3.0,
                sigma: 0.8,
                streak: 3,
            })),
            &[
                "Two groups of choices: a source to add (orange plus: a repeller, blue minus: an attractor, or none) and a dial setting. Commit them together. The first dial setting and \"none\" are never enough.",
                "Scout the hidden source, read its kind from the yellow arrows, and add the opposite kind in the same place to cancel it. That alone leaves the probe off the target at the first dial setting.",
                "With the hidden push gone the visible field decides. The dial scales only its tunable drift (the hidden source and the one you add ignore it), so some settings carry the probe onto the target and others do not. Fly a scout to see where the visible field takes a probe. Three right commits in a row.",
            ]),
    ]
}

/// Shape levels. Every source uses the Survey source's profile (strength 1.65, width 0.8: peak push 0.8, felt out to
/// 1.87), so a player who has met a hidden source has met this one. The probe is released at (-5, 0) and every level
/// ends a run after 25 s (a probe that has not arrived by then is not going to).
/// The 13 dial settings of the dial levels: the default (1.0) first, then 0.0 to 2.4 in steps of 0.2 without 1.0. A winning
/// window of three adjacent settings is a quarter of them, which is what the protocol asks for (docs/07-combinations.md).
fn dial13() -> Vec<f64> {
    vec![
        1.0, 0.0, 0.2, 0.4, 0.6, 0.8, 1.2, 1.4, 1.6, 1.8, 2.0, 2.2, 2.4,
    ]
}

const SHAPE_K: f64 = 1.65;
const SHAPE_SIGMA: f64 = 0.8;
const SHAPE_START: V2 = v2(-5.0, 0.0);
fn shape_rules(goal: usize, zones: Vec<Zone>, kinds: Vec<SourceKind>, sources: usize) -> Rules {
    Rules::Shape {
        start: SHAPE_START,
        goal,
        zones,
        kinds,
        sources,
        k: SHAPE_K,
        sigma: SHAPE_SIGMA,
        period: None,
        intensity: None,
    }
}
fn combo_scout(
    id: &'static str,
    title: &'static str,
    objective: &'static str,
    field: Field,
    regions: Vec<Region>,
    rules: Rules,
) -> Level {
    let mut l = base(id, Chapter::Combo, title, objective, field, regions, rules);
    l.max_time = 20.0;
    l
}
fn combo_sync(
    id: &'static str,
    title: &'static str,
    objective: &'static str,
    field: Field,
    regions: Vec<Region>,
    rules: Rules,
) -> Level {
    let mut l = base(id, Chapter::Combo, title, objective, field, regions, rules);
    l.max_time = 20.0;
    l
}
fn combo_shape(
    id: &'static str,
    title: &'static str,
    objective: &'static str,
    field: Field,
    regions: Vec<Region>,
    rules: Rules,
) -> Level {
    let mut l = base(id, Chapter::Combo, title, objective, field, regions, rules);
    l.max_time = 25.0;
    l
}
/// Levels that must be finished before a combination level is offered, besides the one before it in the chapter.
/// The headings the combination levels are grouped under on the select screen, in order.
pub const COMBO_FAMILIES: [&str; 5] = [
    "First combinations",
    "Two controls before the run",
    "Set up, then steer",
    "Learn, then act",
    "Three decisions",
];
/// Which of `COMBO_FAMILIES` a combination level belongs to (an unknown id goes with the first).
pub fn combo_family(id: &str) -> usize {
    match id {
        "cmb-7" | "cmb-8" | "cmb-9" | "cmb-10" | "cmb-22" | "cmb-23" | "cmb-24" | "cmb-28" => 1,
        "cmb-11" | "cmb-12" | "cmb-13" => 2,
        "cmb-14" | "cmb-15" | "cmb-16" | "cmb-17" | "cmb-18" | "cmb-19" | "cmb-29" => 3,
        "cmb-20" | "cmb-21" | "cmb-25" => 4,
        _ => 0,
    }
}
pub fn combo_requires(id: &str) -> &'static [&'static str] {
    match id {
        "cmb-7" | "cmb-8" => &["shp-1"],
        "cmb-9" => &["shp-1", "syn-1"],
        "cmb-10" => &["pos-1", "syn-1"],
        "cmb-23" => &["syn-1", "exp-1"],
        "cmb-24" => &["tem-1", "exp-1"],
        "cmb-28" => &["ste-1", "syn-1"],
        "cmb-25" => &["exp-1", "shp-1", "sur-1"],
        "cmb-29" => &["tem-1", "sur-1"],
        "cmb-11" | "cmb-12" => &["ste-1"],
        "cmb-22" => &["ste-1", "exp-1"],
        "cmb-13" => &["shp-1", "ste-1"],
        "cmb-16" | "cmb-18" => &["pre-1", "sur-1"],
        "cmb-14" => &["pos-1", "sur-1"],
        "cmb-17" => &["ste-1", "sur-1"],
        "cmb-19" => &["tem-1", "sur-1"],
        "cmb-20" => &["pos-1", "shp-1", "sur-1"],
        "cmb-21" => &["shp-1", "ste-1", "sur-1"],
        "cmb-15" => &["shp-1", "sur-1"],
        _ => &[],
    }
}
fn shape_level(
    id: &'static str,
    title: &'static str,
    objective: &'static str,
    field: Field,
    regions: Vec<Region>,
    rules: Rules,
) -> Level {
    let mut l = base(id, Chapter::Shape, title, objective, field, regions, rules);
    l.max_time = 25.0;
    l
}

pub fn shape_levels() -> Vec<Level> {
    vec![
        // Deflect: a uniform stream carries the probe straight into a small obstacle; the target sits above the
        // stream. Only a repeller is offered, and one is enough: it must be placed *below and upstream* of the
        // obstacle, so that it pushes the probe up before it arrives. Placed beside or beyond the obstacle it does
        // nothing useful. About 9% of the placement area wins (see docs/10-shape.md).
        hints(shape_level("shp-1", "Deflect", "Place a repeller so the flow carries the probe around the obstacle and onto the target.",
            Field::new().fixed(uni(1.0, 0.0)),
            vec![Region::target(4.5, 1.3, 0.7), Region::hazard(-0.5, 0.0, 0.6)],
            shape_rules(0, vec![Zone::Rect(Rect::new(-4.0, -3.0, 3.0, 3.0))], vec![SourceKind::Repeller], 1)),
            &[
                "Press Launch before placing anything: the probe flies the field as it is, and you see where it goes wrong.",
                "A repeller pushes the flow away from itself, so the probe is pushed away from wherever you put it. The arrows redraw as you move it.",
                "A source only changes the path while the probe is near it. Think about which side of the path the push has to come from, and whether it has to act before or after the obstacle.",
            ]),
        // Recover: the stream drifts slowly downward, so the probe passes well below the target. Only an attractor
        // helps (a repeller in the permitted area can only push the probe further down): it pulls the probe toward
        // its own height, so it has to sit at the target's height. An obstacle stands in that lane before the
        // target, so an attractor placed upstream of it pulls the probe into it; it has to act after the obstacle.
        // The placement area stops short of the target (no placing it on or beside the target).
        hints(shape_level("shp-2", "Recover", "The current drifts the probe below the target. Draw it back with a source.",
            Field::new().fixed(uni(1.0, -0.08)),
            vec![Region::target(3.8, 1.2, 0.7), Region::hazard(0.2, 1.2, 0.5)],
            shape_rules(0, vec![Zone::Rect(Rect::new(-1.0, 0.2, 2.8, 2.2))], vec![SourceKind::Repeller, SourceKind::Attractor], 1)),
            &[
                "Launch with nothing placed: the drift carries the probe past the target. Which way does it have to be moved?",
                "An attractor pulls the flow toward itself, so a probe passing near it is drawn sideways toward it. Place one and watch how the arrows around it change.",
                "A pull does not know about the obstacle. Check what lies between your source and the target, and where the probe will be while the pull acts.",
            ]),
        // Turn: an obstacle sits right on the stream and the target straight behind it. A repeller or attractor
        // moves the path to one side for good, so the probe misses the target. A swirl pushes it sideways on the way
        // in and back on the way out: a bump in the path, no net shift. The swirl has to sit beside the path, close
        // to the obstacle, and which way it turns decides which side the bump goes. The stream is slower (0.7), so a
        // source has time to move the probe.
        hints(shape_level("shp-3", "Turn", "The obstacle is on the stream and the target is right behind it. Get past it and come back.",
            Field::new().fixed(uni(0.7, 0.0)),
            vec![Region::target(4.5, 0.0, 0.5), Region::hazard(0.0, 0.0, 0.55)],
            shape_rules(0, vec![Zone::Rect(Rect::new(-2.2, -1.8, 2.2, 1.8))], vec![SourceKind::Repeller, SourceKind::Attractor, SourceKind::SwirlCcw, SourceKind::SwirlCw], 1)),
            &[
                "Try a repeller and an attractor beside the path. The probe goes around, but does it come back to the line of the target?",
                "A swirl pushes the flow around itself rather than away or toward it. Look at which way its arrows point on the side where the probe arrives, and on the side where it leaves.",
                "Which way the swirl turns decides which way it pushes the probe first. The probe has to be moved clear of the obstacle while it passes it.",
            ]),
        // Compose: the target is higher than one source can lift the probe (a source is felt out to about 1.9). Two
        // sources must act in turn: the first lifts the probe into the reach of the second, so where the second can
        // go depends on where the first put the probe. No single source wins; of the pairs about 3% do.
        hints(shape_level("shp-4", "Compose", "The target is higher than one source can reach. Use two, one after the other.",
            Field::new().fixed(uni(1.0, 0.0)),
            vec![Region::target(4.5, 2.8, 0.7), Region::hazard(-0.5, 0.0, 0.55)],
            shape_rules(0, vec![Zone::Rect(Rect::new(-3.0, 0.3, 3.0, 3.2))], vec![SourceKind::Repeller, SourceKind::Attractor], 2)),
            &[
                "Place one source and launch. Where does it leave the probe, and is the target in reach from there?",
                "The second source acts on the probe where the first left it, not where it started. Moving the first one changes where the second one can be useful.",
                "The fields add. Each source only matters near the path, so the order along the stream matters too.",
            ]),
    ]
}

/// Synchronize levels. Every probe is released at a delay in 0..=12 s (one field period, the same clock as Tempo), in
/// steps of 0.1 s, into the same fixed field; the player chooses nothing else.
///
/// Why the fields are what they are (measured with `validate_sync`, see docs/11-synchronize.md):
/// - a *static* field (level 1) makes a probe's travel time independent of its release: success depends only on the
///   difference of the delays, a subtraction. It is kept as the on-ramp and is not by itself a distinct mechanic;
/// - a field with a *local periodic push* on each probe's route (levels 2 and 3) lets a probe reach the target only if
///   it passes the push at the right phase, so each probe can arrive only during certain times and the two sets of
///   possible arrival times have to be matched: the probes cannot be solved one at a time;
/// - a field whose *speed pulses* (levels 4 and 5) makes the time between two places depend on when a probe passes
///   them, so what happens at the gate and what happens at the target are two different conditions on the same delays.
///
/// Where in its cycle the field is at t = 0, in radians. Without it the first window of every level opens at once and
/// a launch with the controls untouched, or nudged a little, is close to a solution (a test refuses that).
const SYNC_PHASE: f64 = 4.0;
fn mgauss(x: f64, y: f64, phase: f64) -> Term {
    Term::Modulated {
        inner: Box::new(Term::Gauss {
            c: v2(x, y),
            k: SHAPE_K,
            sigma: SHAPE_SIGMA,
        }),
        offset: 0.0,
        amp: 1.0,
        omega: TAU / T12,
        phase: phase + SYNC_PHASE,
    }
}
fn pulses() -> Field {
    let w = TAU / T12;
    Field::new()
        .fixed(uni(1.0, 0.0))
        .fixed(Term::Modulated {
            inner: Box::new(uni(1.2, 0.0)),
            offset: 0.0,
            amp: 0.8,
            omega: w,
            phase: SYNC_PHASE,
        })
        .fixed(Term::Modulated {
            inner: Box::new(uni(0.0, 1.0)),
            offset: 0.0,
            amp: 0.8,
            omega: w,
            phase: 2.0 + SYNC_PHASE,
        })
        .fixed(radial(4.5, 0.0, -5.0, 1.0))
}
fn sync_rules(starts: Vec<V2>, conds: Vec<SyncCond>) -> Rules {
    let goals = vec![0; starts.len()];
    Rules::Synchronize {
        starts,
        goals,
        period: T12,
        max_delay: T12,
        conds,
        sources: None,
        start_slots: None,
        dial: None,
    }
}
fn sync_level(
    id: &'static str,
    title: &'static str,
    objective: &'static str,
    field: Field,
    regions: Vec<Region>,
    rules: Rules,
) -> Level {
    let mut l = base(
        id,
        Chapter::Synchronize,
        title,
        objective,
        field,
        regions,
        rules,
    );
    l.max_time = 20.0;
    l
}

pub fn sync_levels() -> Vec<Level> {
    let windows = |phase_b: f64| {
        Field::new()
            .fixed(uni(2.0, 0.0))
            .fixed(mgauss(-1.5, -1.9, 0.0))
            .fixed(mgauss(1.0, 1.9, phase_b))
    };
    vec![
        // Rendezvous: the on-ramp. Nothing depends on when a probe leaves except when it arrives. The probes' routes differ in
        // length (4.0 s and 1.8 s), so releasing both at once misses by 2.2 s: the player has to read the timeline.
        hints(sync_level("syn-1", "Rendezvous", "Two probes must reach the target together. Choose when to release each.",
            Field::new().fixed(uni(1.2, 0.0)).fixed(radial(4.5, 0.0, -5.0, 1.0)),
            vec![Region::target(4.5, 0.0, 0.8)],
            sync_rules(vec![v2(-4.5, -2.5), v2(-1.0, 1.0)], vec![SyncCond::meet(0, 1, 0, -0.4, 0.4)])),
            &[
                "Launch both once and watch the timeline: each bar is how long that probe takes to arrive.",
                "In this field a probe takes the same time whenever it leaves. What matters is how much later one leaves than the other.",
                "The shaded band on the timeline is how far apart the two arrivals may be.",
            ]),
        // Windows: each probe is lifted into the target only if it passes its own pulsing push at the right moment.
        hints(sync_level("syn-2", "Windows", "Each probe can reach the target only at certain times. Find a time they can share.",
            windows(0.0), vec![Region::target(4.5, 0.0, 0.9)],
            sync_rules(vec![v2(-5.0, -1.2), v2(-3.5, 1.2)], vec![SyncCond::meet(0, 1, 0, -0.6, 0.6)])),
            &[
                "Send each probe on its own first and see which release times get it to the target at all.",
                "A probe that can only leave in a window arrives in a window too. The two arrival windows have to overlap.",
                "The shaded band on the timeline is how far apart the arrivals may be; a probe that misses the target has no arrival to compare.",
            ]),
        // Order: the same kind of windows, but B must come after A rather than with it.
        hints(sync_level("syn-3", "Order", "The second probe must arrive after the first, not with it.",
            windows(-0.5), vec![Region::target(4.5, 0.0, 0.9)],
            sync_rules(vec![v2(-5.0, -1.2), v2(-3.5, 1.2)], vec![SyncCond::meet(0, 1, 0, 1.0, 2.2)])),
            &[
                "The band on the timeline now sits after the first arrival: the second probe has to be late by a small, particular amount.",
                "Each probe still has its own times at which it can reach the target at all.",
                "Choose the first arrival you want, then see whether the second probe can arrive 1 to 2 seconds later.",
            ]),
        // Meet, then part: the probes must meet at the gate and then arrive at the target one after the other. Meeting
        // twice would cost nothing (two probes at one place at one time share a future), so the second condition asks for
        // the opposite: the field has to open a gap between probes that were together at the gate.
        hints(sync_level("syn-4", "Meet, Then Part", "The probes must meet at the gate, then reach the target one after the other.",
            pulses(), vec![Region::target(4.5, 0.0, 0.8), Region::gate(0.5, 0.0, 1.3)],
            sync_rules(vec![v2(-4.5, -2.0), v2(-4.5, 0.8)], vec![SyncCond::meet(0, 1, 1, -1.0, 1.0), SyncCond::meet(0, 1, 0, 0.4, 1.2)])),
            &[
                "Meeting at the gate and arriving apart at the target are two conditions on the same two release times.",
                "The field speeds probes up and slows them down in a cycle, so how far apart they arrive depends on when they pass through it.",
                "A pair that meets at the gate can still arrive together at the target. Check both bands on the timeline after every launch.",
            ]),
        // Chain: three probes; the middle one is the link.
        hints(sync_level("syn-5", "Chain", "The first and second meet at the gate. The second and third meet at the target.",
            pulses(), vec![Region::target(4.5, 0.0, 0.8), Region::gate(0.5, 0.0, 1.3)],
            sync_rules(vec![v2(-4.5, -2.0), v2(-4.5, 0.8), v2(-4.5, 3.0)], vec![SyncCond::meet(0, 1, 1, -0.8, 0.8), SyncCond::meet(1, 2, 0, -0.8, 0.8)])),
            &[
                "Two pairs have to line up, and the middle probe belongs to both.",
                "Decide when the middle probe goes, then fit the other two around it.",
                "The third probe cannot reach the target at every time. Try it alone first.",
            ]),
    ]
}

pub fn all() -> Vec<Level> {
    let mut v = position_levels();
    v.extend(predict_levels());
    v.extend(tempo_levels());
    v.extend(experiment_levels());
    v.extend(steer_levels());
    v.extend(survey_levels());
    v.extend(shape_levels());
    v.extend(sync_levels());
    v.extend(combo_levels());
    v
}

pub fn chapter(c: Chapter) -> Vec<Level> {
    all().into_iter().filter(|l| l.chapter == c).collect()
}
