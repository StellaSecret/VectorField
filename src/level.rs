//! Level definitions are plain data. Rendering and UI only read them.
use crate::field::{v2, Field, SourceKind, V2};
use crate::sim::{Outcome, Rect, Region, RegionKind, SimConfig, Trajectory};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Chapter {
    Position,
    Predict,
    Tempo,
    Experiment,
    Steer,
    Survey,
    Shape,
    Synchronize,
    Combo,
}
impl Chapter {
    pub const ALL: [Chapter; 9] = [
        Chapter::Position,
        Chapter::Predict,
        Chapter::Tempo,
        Chapter::Experiment,
        Chapter::Steer,
        Chapter::Survey,
        Chapter::Shape,
        Chapter::Synchronize,
        Chapter::Combo,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Chapter::Position => "Position",
            Chapter::Predict => "Predict",
            Chapter::Tempo => "Tempo",
            Chapter::Experiment => "Experiment",
            Chapter::Steer => "Steer",
            Chapter::Survey => "Survey",
            Chapter::Shape => "Shape",
            Chapter::Synchronize => "Synchronize",
            Chapter::Combo => "Combinations",
        }
    }
    pub fn question(self) -> &'static str {
        match self {
            Chapter::Position => "Where should I start?",
            Chapter::Predict => "Where will it go?",
            Chapter::Tempo => "When should I launch?",
            Chapter::Experiment => "How does the system behave?",
            Chapter::Steer => "What should I do next?",
            Chapter::Survey => "Where is the hidden source?",
            Chapter::Shape => "How should I change the field?",
            Chapter::Synchronize => "How must these probes be coordinated?",
            Chapter::Combo => "Mix what you have learned.",
        }
    }
}

/// Permitted gust directions. Index 0 is East and indices run counter-clockwise.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GustDirs {
    /// E, N, W, S
    Four,
    /// E, NE, N, NW, W, SW, S, SE
    Eight,
}
impl GustDirs {
    pub fn count(self) -> usize {
        match self {
            GustDirs::Four => 4,
            GustDirs::Eight => 8,
        }
    }
    /// Unit vector of direction `i` (exact for the cardinal directions).
    pub fn unit(self, i: usize) -> V2 {
        const S: f64 = std::f64::consts::FRAC_1_SQRT_2;
        const T: [V2; 8] = [
            v2(1.0, 0.0),
            v2(S, S),
            v2(0.0, 1.0),
            v2(-S, S),
            v2(-1.0, 0.0),
            v2(-S, -S),
            v2(0.0, -1.0),
            v2(S, -S),
        ];
        match self {
            GustDirs::Eight => T[i % 8],
            GustDirs::Four => T[(i % 4) * 2],
        }
    }
    pub fn name(self, i: usize) -> &'static str {
        const N: [&str; 8] = ["E", "NE", "N", "NW", "W", "SW", "S", "SE"];
        match self {
            GustDirs::Eight => N[i % 8],
            GustDirs::Four => N[(i % 4) * 2],
        }
    }
}

/// A region of the plane (release zones).
#[derive(Clone, Debug)]
pub enum Zone {
    Circle(V2, f64),
    Rect(Rect),
}
impl Zone {
    pub fn contains(&self, p: V2) -> bool {
        match self {
            Zone::Circle(c, r) => p.dist(*c) <= *r,
            Zone::Rect(r) => r.contains(p),
        }
    }
    pub fn clamp(&self, p: V2) -> V2 {
        match self {
            Zone::Circle(c, r) => {
                let d = p - *c;
                if d.len() <= *r {
                    p
                } else {
                    *c + d * (*r / d.len())
                }
            }
            Zone::Rect(r) => r.clamp(p),
        }
    }
    pub fn center(&self) -> V2 {
        match self {
            Zone::Circle(c, _) => *c,
            Zone::Rect(r) => v2((r.min.x + r.max.x) * 0.5, (r.min.y + r.max.y) * 0.5),
        }
    }
    pub fn bbox(&self) -> Rect {
        match self {
            Zone::Circle(c, r) => Rect::new(c.x - r, c.y - r, c.x + r, c.y + r),
            Zone::Rect(r) => *r,
        }
    }
    /// n x n sample grid (edges included) restricted to the zone.
    pub fn grid(&self, n: usize) -> Vec<V2> {
        let b = self.bbox();
        let mut out = vec![];
        for i in 0..n {
            for j in 0..n {
                let p = v2(
                    b.min.x + b.width() * i as f64 / (n - 1) as f64,
                    b.min.y + b.height() * j as f64 / (n - 1) as f64,
                );
                if self.contains(p) {
                    out.push(p);
                }
            }
        }
        out
    }
}

#[derive(Clone, Debug)]
pub struct Slot {
    pub zone: Zone,
    /// Index into `Level::regions` this probe must reach.
    pub goal: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Reveal {
    Hidden,
    /// Show the first N seconds of the true path before the player commits.
    Partial(f64),
}

/// What an Experiment level asks the player to discover.
#[derive(Clone, Debug)]
pub enum Challenge {
    /// As intensity rises, does the exit point travel clockwise or counter-clockwise around `center`?
    ExitShift { center: V2 },
    /// Sweeping intensity upward: does the probe ever return to an exit it already left?
    Return,
    /// The test rig only runs up to `limit`; predict the outcome at the untestable `target` beyond it.
    Extrapolate { limit: f64, target: f64 },
    /// Find the intensity where the exit changes from `from` to `to`.
    Threshold { from: usize, to: usize, tol: f64 },
    /// Find the interval of intensity for which the probe leaves through `exit`.
    Interval { exit: usize, tol: f64 },
}

/// What completes a Survey level.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SurveyGoal {
    /// This many attempts found (not necessarily of different kinds).
    Finds(usize),
    /// One attempt of each kind of source the level hides (repeller, attractor, swirl) found. The level then chooses
    /// each attempt from the kinds still missing.
    EachKind,
}

/// A timing relationship between two probes: probe `b` must *enter* region `rb` between `lo` and `hi` seconds after
/// probe `a` enters region `ra`. With `ra == rb` and `lo = -w, hi = w` it is a rendezvous within `w` at that region;
/// with `0 < lo < hi` it is "b arrives shortly after a"; with different regions it orders two checkpoints, one on each
/// route. A region can be a gate in the middle of a route or the final target.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SyncCond {
    pub a: usize,
    pub ra: usize,
    pub b: usize,
    pub rb: usize,
    pub lo: f64,
    pub hi: f64,
}
impl SyncCond {
    /// The common case: both probes at one region.
    pub const fn meet(a: usize, b: usize, region: usize, lo: f64, hi: f64) -> SyncCond {
        SyncCond {
            a,
            ra: region,
            b,
            rb: region,
            lo,
            hi,
        }
    }
}

/// Sources the player places before a Steer run (combination levels only), as in a Shape level.
#[derive(Clone, Debug)]
pub struct SteerSources {
    pub zones: Vec<Zone>,
    pub kinds: Vec<SourceKind>,
    pub max: usize,
    pub k: f64,
    pub sigma: f64,
}

/// The categorical act that ends an attempt of a scout-then-act level. Every act is a choice among a few wide,
/// labelled options (docs/07-combinations.md: a continuous act, a placement or a start height, needs a precision a
/// person cannot read off scout trails, so those designs failed).
#[derive(Clone, Debug)]
pub enum ScoutAct {
    /// Call which of `ScoutRules::exits` the main probe leaves through (Survey + Predict).
    Exit,
    /// Choose which of `starts` the main probe is released from; right if it then reaches region `goal` (Survey + Position).
    Release { starts: Vec<V2>, goal: usize },
    /// Choose one of `sites` (a kind of source at a place) to add to the field before the probe flies from
    /// `ScoutRules::start`; right if it then reaches region `goal` (Survey + Shape).
    Cancel {
        sites: Vec<(SourceKind, V2)>,
        goal: usize,
    },
    /// Choose one of `plans` (per beat: a gust direction index, or `None` to wait) to steer the probe from
    /// `ScoutRules::start`; right if it then reaches region `goal` (Survey + Steer).
    Plan {
        plans: Vec<Vec<Option<usize>>>,
        goal: usize,
        dirs: GustDirs,
        strength: f64,
        beat_time: f64,
    },
    /// Choose the launch phase (the field clock at release, as in Tempo) of the main probe from `phases` (phase 0
    /// is the default, listed first); right if it reaches region `goal` (Survey + Tempo).
    Phase { phases: Vec<f64>, goal: usize },
    /// Choose the dial setting (the intensity of the tunable layers, as in Experiment) from `levels` (the default
    /// first); right if the probe reaches region `goal` (Survey + Experiment). Hidden sources ignore the dial.
    Dial { levels: Vec<f64>, goal: usize },
    /// Several of the acts above, chosen one after another and judged together by one flight (three mechanics in
    /// one level). The options are every combination; the first act is the most significant. In a sequence a
    /// `Cancel` and a `Plan` gain a last option "none" (no source added, no gust). Allowed: `Release`, `Cancel`,
    /// `Plan`, `Phase`, `Dial`; a `Plan` cannot be combined with `Phase` (the steer engine has no clock).
    Seq(Vec<ScoutAct>),
}

/// Combination levels that scout first and then act. A hidden source (one of `pool`, a different one each attempt)
/// is added to the visible field every flight sees. The player launches a few scouts, reads the push they feel, then
/// calls which exit the main probe will leave through. `streak` correct calls in a row complete the level.
#[derive(Clone, Debug)]
pub struct ScoutRules {
    /// What the player commits to after scouting.
    pub act: ScoutAct,
    /// Where the main probe starts.
    pub start: V2,
    /// The exits, as indices into `Level::regions` (in the order the call buttons are labelled).
    pub exits: Vec<usize>,
    /// How many scouts an attempt may launch, where they may start, and how many seconds each flies.
    pub scouts: usize,
    pub zone: Rect,
    pub scout_time: f64,
    /// The hidden sources: attempt `i` uses `pool[i % pool.len()]`. All share one strength and width.
    pub pool: Vec<(SourceKind, V2)>,
    pub k: f64,
    pub sigma: f64,
    /// Correct calls in a row needed to complete the level.
    pub streak: usize,
}

#[derive(Clone, Debug)]
pub enum Rules {
    /// Choose release position(s). Optional phase / shared-intensity controls are for combination levels.
    Position {
        slots: Vec<Slot>,
        period: Option<f64>,
        intensity: Option<(f64, f64)>,
    },
    /// Commit to a forecast of the exit; `rounds` rounds, `pass` correct needed.
    Predict {
        zone: Zone,
        rounds: usize,
        pass: usize,
        reveal: Reveal,
    },
    /// Turn-based control: after every beat the player picks a gust (or waits). The field, start and
    /// target are fixed and fully visible; a gust adds `gust_strength` to the velocity for one beat.
    ///
    /// `period`: combination levels only. The field is periodic and the player also chooses the field clock at
    /// which the run starts (a launch phase in `0..period`), before the first beat.
    /// `start_zone`: combination levels only. The player also chooses the start point inside this zone (`start` is
    /// where the run begins until they do), before the first beat.
    /// `sources`: combination levels only. The player also places sources before the first beat; they are steady
    /// terms added to the field, and the gusts are played in the field they make.
    Steer {
        start: V2,
        beats: usize,
        gust_budget: usize,
        gust_strength: f64,
        dirs: GustDirs,
        beat_time: f64,
        goal: usize,
        period: Option<f64>,
        start_zone: Option<Rect>,
        sources: Option<SteerSources>,
    },
    /// Hidden-source survey. `Level::field` is the *charted* background. A hidden local repeller
    /// (`Term::Gauss`, known strength `k` and width `sigma`) adds to it at one position, drawn from
    /// `pool` (a different one each attempt). One scout is flown for free from `free`; the player
    /// places up to `scouts` more inside `zone` (each leaves a `Level::max_time`-second trail), then
    /// commits one estimate of the source position, correct within `tol`.
    /// `rays`: draw each push arrow as a long faint line back toward the source, so where they cross can be read
    /// off directly. On in the teaching level; later levels turn it off and the player must extend the arrows
    /// by eye.
    ///
    /// `band`: shade, around each scout's trail, the places a source would already have been felt (see
    /// `SurveySpec::cleared`), so a scout that sees nothing still rules something out.
    ///
    /// `kinds[i]` is what the source at `pool[i]` does (all `Repeller` unless the level is about telling kinds
    /// apart): the marker then carries a kind too, and a commit is right only if both agree.
    ///
    /// `preview`: while the player aims a scout, draw where it would fly under the charted background alone (no\n    /// hidden source), so in a background that curves the player can see where a release would carry the scout\n    /// before spending it. It depends only on what is already charted and reveals nothing about the source.\n    ///\n    /// `partners[i]`: a second source of the same kind hidden together with `pool[i]` (`None` for one source). In a
    /// level where any attempt has a partner the player has two markers, and a commit is found only when each marker is
    /// within the tolerance of a different source, in either assignment.
    ///
    /// `goal`: what completes the level (see `SurveyGoal`).
    Survey {
        zone: Zone,
        free: V2,
        scouts: usize,
        k: f64,
        sigma: f64,
        tol: f64,
        rays: bool,
        band: bool,
        preview: bool,
        pool: Vec<V2>,
        kinds: Vec<SourceKind>,
        partners: Vec<Option<V2>>,
        goal: SurveyGoal,
    },
    /// Shape the field. The background (`Level::field`), the probe's start, the obstacles and the target
    /// (`Level::regions`) are fixed and fully visible. The player places up to `sources` local sources of the
    /// permitted `kinds` (all with strength `k` and width `sigma`: the same Gaussian profile a Survey source has,
    /// `SourceKind::term`) inside `zones`, then launches the probe; the probe flies the *combined* field and must
    /// reach region `goal`. A source cannot be placed on a target or an obstacle, nor within `MIN_SEPARATION` of
    /// another source. Sources can be moved and removed freely between launches, and launching is free and
    /// repeatable: the only way to see a path is to fly it (there is no preview).
    ///
    /// `period` and `intensity`: combination levels only. `period` adds a launch-phase control (the field clock at
    /// release, as in Tempo); `intensity` adds a dial over the tunable layers of the background (as in a Position
    /// level with a shared dial). Sources never depend on either: they are steady and ignore the dial.
    Shape {
        start: V2,
        goal: usize,
        zones: Vec<Zone>,
        kinds: Vec<SourceKind>,
        sources: usize,
        k: f64,
        sigma: f64,
        period: Option<f64>,
        intensity: Option<(f64, f64)>,
    },
    /// Coordinate several probes. Every probe has a fixed start and flies the same fixed field (`Level::field`,
    /// periodic in most levels, `period`); the player chooses only *when each one is released*, a delay between 0 and
    /// `max_delay` (the field clock at release, as in Tempo). Every probe must reach its goal region (`goals[i]`, passing
    /// every gate), and every condition in `conds` must hold. The relationship between the probes, not any one probe,
    /// decides success.
    ///
    /// `start_slots`: combination levels only. `(probe, slots)`: the player also chooses which of the numbered `slots`
    /// probe `probe` starts from (slot 0 is its default, equal to `starts[probe]`). A choice among a few labelled
    /// places, not a free point: see docs/07-combinations.md for why a start line failed.
    Synchronize {
        starts: Vec<V2>,
        goals: Vec<usize>,
        period: f64,
        max_delay: f64,
        conds: Vec<SyncCond>,
        sources: Option<SteerSources>,
        start_slots: Option<(usize, Vec<V2>)>,
    },
    /// Scout, then call (combination levels): see `ScoutRules`.
    Scout(ScoutRules),
    /// Choose the launch phase of a periodic field.
    Tempo { start: V2, period: f64, goal: usize },
    /// Probe the intensity knob, then report the rule you found.
    Experiment {
        start: V2,
        range: (f64, f64),
        challenge: Challenge,
    },
}

#[derive(Clone, Debug)]
pub struct Level {
    pub id: &'static str,
    pub chapter: Chapter,
    pub title: &'static str,
    pub objective: &'static str,
    /// Progressive hints: explain field behaviour, never the answer.
    pub hints: Vec<&'static str>,
    pub field: Field,
    pub bounds: Rect,
    pub regions: Vec<Region>,
    pub rules: Rules,
    pub max_time: f64,
    pub seed: u64,
}

impl Level {
    pub fn sim_config(&self) -> SimConfig {
        SimConfig::new(self.bounds, self.max_time)
    }
    pub fn run(&self, field: &Field, start: V2, t0: f64) -> Trajectory {
        crate::sim::simulate(field, start, t0, &self.regions, &self.sim_config())
    }
    pub fn gates(&self) -> Vec<usize> {
        self.regions
            .iter()
            .enumerate()
            .filter(|(_, r)| r.kind == RegionKind::Gate)
            .map(|(i, _)| i)
            .collect()
    }
    pub fn exits(&self) -> Vec<usize> {
        self.regions
            .iter()
            .enumerate()
            .filter(|(_, r)| r.kind == RegionKind::Exit)
            .map(|(i, _)| i)
            .collect()
    }
    /// Success = reached `goal` after passing every gate.
    pub fn reached_goal(&self, t: &Trajectory, goal: usize) -> bool {
        t.outcome == Outcome::Reached(goal) && self.gates().iter().all(|g| t.gates.contains(g))
    }
    pub fn describe(&self, o: Outcome) -> String {
        match o {
            Outcome::Reached(i) => match self.regions[i].kind {
                RegionKind::Exit => format!("Left through exit {}", self.regions[i].label),
                _ => "Reached the target".to_string(),
            },
            Outcome::Hazard(_) => "Hit the hazard".to_string(),
            Outcome::Boundary => "Drifted out of the field".to_string(),
            Outcome::Timeout => "Ran out of time".to_string(),
            Outcome::Stalled => "Came to rest".to_string(),
            Outcome::Invalid => "Simulation error".to_string(),
        }
    }
}
