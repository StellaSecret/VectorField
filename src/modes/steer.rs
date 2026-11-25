//! STEER - "What should I do next?"  Closed-loop, turn-based control.
//!
//! After every beat the simulation pauses and the player picks a *gust* (a one-beat velocity
//! contribution in one of the permitted directions) or waits. The probe obeys
//!
//! ```text
//! dp/dt = v(p) + u,   u = gust vector during the current beat, 0 otherwise,
//! ```
//!
//! integrated by the same RK4 sub-step (`sim::substep`) as every other mode. A gust never moves the
//! probe directly and never persists past its beat.
//!
//! State machine (spec section 3) and where each state lives:
//!   Ready            `Phase::Ready`: initial state, nothing chosen yet.
//!   AwaitingDecision `Phase::Awaiting`: probe stationary, input accepted.
//!   Advancing        `Phase::Advancing`: `choose` was accepted; input is rejected until `finish_beat`.
//!   Evaluating       resolved atomically inside `finish_beat` (success / bounds / validity / beats).
//!   Completed        `Phase::Completed(End)`: frozen.
//!   Retry            `retry()` restores the exact initial state and keeps the finished run as a ghost.
use crate::field::{Field, V2};

use crate::level::{Companion, GustDirs, Level, Rules, SteerSources};
use crate::modes::shape::{PlaceError, Source, MIN_SEPARATION};
use crate::sim::{advance_beat, Outcome, Rect, Region, RegionKind, SimConfig};

/// Number of earlier attempts kept as faint ghosts (the current run is never truncated).
pub const MAX_GHOSTS: usize = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Wait,
    /// Gust in direction index `i` of the level's `GustDirs`.
    Gust(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FailReason {
    Boundary,
    BeatsExhausted,
    Invalid,
    Hazard,
    MissedGate,
    WrongRegion,
    /// Reached the goal, but not within the band of the companion drone's arrival.
    MissedRendezvous,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum End {
    Success,
    Failed(FailReason),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChooseError {
    /// A beat is being advanced (or the run is not accepting input): duplicate input is rejected.
    Busy,
    Finished,
    NoBudget,
    BadDirection,
}

/// Everything about a run that is needed to continue it. Cheap to clone (used by exhaustive search).
#[derive(Clone, Debug)]
pub struct State {
    pub pos: V2,
    pub beat: usize,
    pub gusts_left: usize,
    pub gates: Vec<usize>,
    pub end: Option<End>,
}

/// One resolved beat.
#[derive(Clone, Debug)]
pub struct Beat {
    pub action: Action,
    pub from: V2,
    /// Natural field velocity at `from` (what the player saw before deciding).
    pub field_v: V2,
    /// Gust velocity used during this beat (zero for `Wait`).
    pub gust: V2,
    /// Sub-step positions during the beat (excluding `from`).
    pub points: Vec<V2>,
    pub next: State,
}
impl Beat {
    pub fn to(&self) -> V2 {
        self.next.pos
    }
}

/// An (attempted) run: the start plus the beats that were played.
#[derive(Clone, Debug)]
pub struct Run {
    pub start: V2,
    pub beats: Vec<Beat>,
    pub end: Option<End>,
}
impl Run {
    pub fn actions(&self) -> Vec<Action> {
        self.beats.iter().map(|b| b.action).collect()
    }
    /// Full polyline: start followed by every sub-step.
    pub fn path(&self) -> Vec<V2> {
        let mut v = vec![self.start];
        for b in &self.beats {
            v.extend_from_slice(&b.points);
        }
        v
    }
    pub fn gusts_used(&self) -> usize {
        self.beats
            .iter()
            .filter(|b| matches!(b.action, Action::Gust(_)))
            .count()
    }
}

/// Immutable description of a Steer level; `advance` is a pure function of (spec, state, action).
#[derive(Clone, Debug)]
pub struct SteerSpec {
    pub field: Field,
    pub regions: Vec<Region>,
    pub cfg: SimConfig,
    pub start: V2,
    pub beats: usize,
    pub budget: usize,
    pub strength: f64,
    pub dirs: GustDirs,
    pub beat_time: f64,
    pub goal: usize,
    pub gates: Vec<usize>,
    /// Combination levels: the field's period when the player also picks the launch phase.
    pub period: Option<f64>,
    /// Field clock at beat 0 (the chosen launch phase; 0 when the level has no `period`).
    pub clock0: f64,
    /// Combination levels: the zone the player may choose the start point in (`start` is the default).
    pub start_zone: Option<Rect>,
    /// Combination levels: the sources the player may place, and the ones placed (the field already includes them).
    pub source_rules: Option<SteerSources>,
    pub sources: Vec<Source>,
    /// The level's own field, without sources: `field` is this plus one steady term per placed source.
    pub field0: Field,
    /// Combination levels: the dial settings the player may choose from (the first is the default) and the one chosen.
    pub dial_levels: Option<Vec<f64>>,
    pub dial: usize,
    /// Combination levels: the companion drone, the delay chosen for it, and when it reaches the goal (`None` if it
    /// never does, in which case the steered probe cannot succeed).
    pub companion: Option<Companion>,
    pub drone: usize,
    pub drone_arrival: Option<f64>,
}

impl SteerSpec {
    pub fn from_level(l: &Level) -> SteerSpec {
        let Rules::Steer {
            start,
            beats,
            gust_budget,
            gust_strength,
            dirs,
            beat_time,
            goal,
            period,
            start_zone,
            sources,
            dial,
            companion,
        } = &l.rules
        else {
            panic!("not a Steer level")
        };
        let mut sp = SteerSpec {
            field: l.field.clone(),
            regions: l.regions.clone(),
            cfg: l.sim_config(),
            start: *start,
            beats: *beats,
            budget: *gust_budget,
            strength: *gust_strength,
            dirs: *dirs,
            beat_time: *beat_time,
            goal: *goal,
            gates: l.gates(),
            period: *period,
            clock0: 0.0,
            start_zone: *start_zone,
            source_rules: sources.clone(),
            sources: vec![],
            field0: l.field.clone(),
            dial_levels: dial.clone(),
            dial: 0,
            companion: companion.clone(),
            drone: 0,
            drone_arrival: None,
        };
        sp.drone_arrival = sp.compute_drone();
        sp
    }
    /// When the drone reaches the goal at the delay chosen now, or `None` if it does not (or there is no drone).
    pub fn compute_drone(&self) -> Option<f64> {
        let c = self.companion.as_ref()?;
        let t = crate::sim::simulate(
            &self.field,
            c.start,
            *c.delays.get(self.drone)?,
            &self.regions,
            &self.cfg,
        );
        if t.outcome != Outcome::Reached(self.goal) {
            return None;
        }
        let r = &self.regions[self.goal];
        t.points
            .iter()
            .position(|p| p.dist(r.center) <= r.radius)
            .map(|k| t.t0 + k as f64 * t.dt)
    }
    /// The path the drone flies at the delay chosen now (empty without a drone), for drawing.
    pub fn drone_path(&self) -> Vec<V2> {
        let Some(c) = &self.companion else {
            return vec![];
        };
        let Some(&d) = c.delays.get(self.drone) else {
            return vec![];
        };
        crate::sim::simulate(&self.field, c.start, d, &self.regions, &self.cfg).points
    }
    /// The same level with the drone released at delay `i` (a level without a drone, or an unknown delay, is unchanged).
    pub fn with_drone(&self, i: usize) -> SteerSpec {
        let mut s = self.clone();
        if self.companion.as_ref().is_some_and(|c| i < c.delays.len()) {
            s.drone = i;
            s.drone_arrival = s.compute_drone();
        }
        s
    }
    /// The same level with dial setting `i` (a level without a dial, or an unknown setting, is unchanged). The dial
    /// scales the tunable layers of the field; placed sources and the level's fixed layers ignore it.
    pub fn with_dial(&self, i: usize) -> SteerSpec {
        let mut s = self.clone();
        if let Some(v) = &self.dial_levels {
            if let Some(&level) = v.get(i) {
                s.field = self.field0.with_intensity(level);
                for q in &self.sources {
                    if let Some(r) = &self.source_rules {
                        s.field = s.field.fixed(q.kind.term(q.pos, r.k, r.sigma));
                    }
                }
                s.dial = i;
            }
        }
        s
    }
    /// Where `p` would be refused for a new source, ignoring the other sources.
    pub fn site_error(&self, p: V2) -> Option<PlaceError> {
        let Some(r) = &self.source_rules else {
            return Some(PlaceError::KindUnavailable);
        };
        if !r.zones.iter().any(|z| z.contains(p)) {
            return Some(PlaceError::OutsideZone);
        }
        for g in &self.regions {
            if matches!(g.kind, RegionKind::Target | RegionKind::Hazard)
                && p.dist(g.center) <= g.radius
            {
                return Some(PlaceError::OnRegion(g.label));
            }
        }
        None
    }
    /// Every legal site on a lattice of spacing `step`.
    pub fn sites(&self, step: f64) -> Vec<V2> {
        let mut out: Vec<V2> = vec![];
        if let Some(r) = &self.source_rules {
            for z in &r.zones {
                let b = z.bbox();
                let (nx, ny) = (
                    (b.width() / step).floor() as i64,
                    (b.height() / step).floor() as i64,
                );
                for i in 0..=nx {
                    for j in 0..=ny {
                        let p = V2 {
                            x: b.min.x + i as f64 * step,
                            y: b.min.y + j as f64 * step,
                        };
                        if self.site_error(p).is_none() && !out.iter().any(|q| q.dist(p) < 1e-9) {
                            out.push(p);
                        }
                    }
                }
            }
        }
        out
    }
    /// The same level with exactly these sources placed (the field is the level's own plus one steady term each).
    /// A level that offers no sources ignores them.
    pub fn with_sources(&self, sources: &[Source]) -> SteerSpec {
        let mut s = self.clone();
        if let Some(r) = &self.source_rules {
            let mut f = self.field0.clone();
            for q in sources {
                f = f.fixed(q.kind.term(q.pos, r.k, r.sigma));
            }
            s.field = f;
            s.sources = sources.to_vec();
        }
        s
    }
    /// Is `p` somewhere the run may start: inside the zone and not inside any region?
    pub fn start_legal(&self, p: V2) -> bool {
        match self.start_zone {
            Some(z) => {
                p.x >= z.min.x - 1e-9
                    && p.x <= z.max.x + 1e-9
                    && p.y >= z.min.y - 1e-9
                    && p.y <= z.max.y + 1e-9
                    && self.regions.iter().all(|r| r.center.dist(p) > r.radius)
            }
            None => false,
        }
    }
    /// The same level started from `p` (ignored unless the level has a start zone and `p` is legal).
    pub fn with_start(&self, p: V2) -> SteerSpec {
        let mut s = self.clone();
        if self.start_legal(p) {
            s.start = p;
        }
        s
    }
    /// The same level run from field clock `phase` (wrapped into the period). A level without a period ignores it.
    pub fn with_clock(&self, phase: f64) -> SteerSpec {
        let mut s = self.clone();
        if let Some(p) = self.period {
            s.clock0 = phase.rem_euclid(p);
        }
        s
    }
    pub fn initial(&self) -> State {
        State {
            pos: self.start,
            beat: 0,
            gusts_left: self.budget,
            gates: vec![],
            end: None,
        }
    }
    /// Gust velocity vector for an action.
    pub fn gust(&self, a: Action) -> V2 {
        match a {
            Action::Wait => V2::ZERO,
            Action::Gust(i) => self.dirs.unit(i) * self.strength,
        }
    }
    /// Natural field velocity at a position during beat `beat`.
    pub fn field_velocity(&self, p: V2, beat: usize) -> V2 {
        self.field
            .eval(p, self.clock0 + beat as f64 * self.beat_time)
    }
    pub fn legal(&self, s: &State, a: Action) -> Result<(), ChooseError> {
        if s.end.is_some() {
            return Err(ChooseError::Finished);
        }
        match a {
            Action::Wait => Ok(()),
            Action::Gust(i) if i >= self.dirs.count() => Err(ChooseError::BadDirection),
            Action::Gust(_) if s.gusts_left == 0 => Err(ChooseError::NoBudget),
            Action::Gust(_) => Ok(()),
        }
    }
    /// Advance exactly one beat. Deterministic; allocates a new `State`.
    pub fn advance(&self, s: &State, a: Action) -> Result<Beat, ChooseError> {
        self.legal(s, a)?;
        let u = self.gust(a);
        let t = self.clock0 + s.beat as f64 * self.beat_time;
        let mut gates = s.gates.clone();
        let (points, outcome) = advance_beat(
            &self.field,
            s.pos,
            t,
            u,
            self.beat_time,
            &self.regions,
            &self.cfg,
            &mut gates,
        );
        let to = points.last().copied().unwrap_or(s.pos);
        let beat = s.beat + 1;
        let gusts_left = s.gusts_left - usize::from(matches!(a, Action::Gust(_)));
        let end = match outcome {
            Some(Outcome::Reached(i)) if i == self.goal => {
                if !self.gates.iter().all(|g| gates.contains(g)) {
                    Some(End::Failed(FailReason::MissedGate))
                } else if let Some(c) = &self.companion {
                    // the steered probe entered the goal at the last point of this beat
                    let arrived = t + points.len() as f64 * self.cfg.dt;
                    match self.drone_arrival {
                        Some(a) if (arrived - a).abs() <= c.band => Some(End::Success),
                        _ => Some(End::Failed(FailReason::MissedRendezvous)),
                    }
                } else {
                    Some(End::Success)
                }
            }
            Some(Outcome::Reached(_)) => Some(End::Failed(FailReason::WrongRegion)),
            Some(Outcome::Hazard(_)) => Some(End::Failed(FailReason::Hazard)),
            Some(Outcome::Boundary) => Some(End::Failed(FailReason::Boundary)),
            Some(Outcome::Invalid) => Some(End::Failed(FailReason::Invalid)),
            Some(Outcome::Timeout) | Some(Outcome::Stalled) => {
                Some(End::Failed(FailReason::BeatsExhausted))
            }
            None if beat >= self.beats => Some(End::Failed(FailReason::BeatsExhausted)),
            None => None,
        };
        Ok(Beat {
            action: a,
            from: s.pos,
            field_v: self.field_velocity(s.pos, s.beat),
            gust: u,
            points,
            next: State {
                pos: to,
                beat,
                gusts_left,
                gates,
                end,
            },
        })
    }
    /// Re-simulate a recorded action sequence from the initial state (replay).
    pub fn replay(&self, actions: &[Action]) -> Result<Run, ChooseError> {
        let mut s = self.initial();
        let mut beats = vec![];
        for a in actions {
            let b = self.advance(&s, *a)?;
            s = b.next.clone();
            beats.push(b);
        }
        Ok(Run {
            start: self.start,
            beats,
            end: s.end,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Ready,
    Awaiting,
    Advancing,
    Completed(End),
}

pub struct SteerGame {
    pub level: Level,
    pub spec: SteerSpec,
    pub state: State,
    pub phase: Phase,
    pub history: Vec<Beat>,
    pending: Option<Beat>,
    /// Earlier completed attempts, oldest first, at most `MAX_GHOSTS`.
    pub ghosts: Vec<Run>,
    pub attempts: usize,
    pub won: bool,
}

impl SteerGame {
    pub fn new(level: Level) -> SteerGame {
        let spec = SteerSpec::from_level(&level);
        let state = spec.initial();
        SteerGame {
            level,
            spec,
            state,
            phase: Phase::Ready,
            history: vec![],
            pending: None,
            ghosts: vec![],
            attempts: 0,
            won: false,
        }
    }
    /// Choose the launch phase. Only before the first beat, and only in a level that has a period.
    pub fn set_clock(&mut self, phase: f64) -> bool {
        if self.spec.period.is_none() || self.phase != Phase::Ready {
            return false;
        }
        self.spec = self.spec.with_clock(phase);
        true
    }
    pub fn clock(&self) -> f64 {
        self.spec.clock0
    }
    /// Choose when the drone is released. Only before the first beat, and only in a level that has a drone.
    pub fn set_drone(&mut self, i: usize) -> bool {
        let n = self.spec.companion.as_ref().map_or(0, |c| c.delays.len());
        if i >= n || i == self.spec.drone || self.phase != Phase::Ready {
            return false;
        }
        self.spec = self.spec.with_drone(i);
        true
    }
    /// Choose the dial setting. Only before the first beat, and only in a level that has a dial.
    pub fn set_dial(&mut self, i: usize) -> bool {
        let n = self.spec.dial_levels.as_ref().map_or(0, |v| v.len());
        if i >= n || i == self.spec.dial || self.phase != Phase::Ready {
            return false;
        }
        self.spec = self.spec.with_dial(i);
        true
    }
    /// Choose the start point. Only before the first beat, only in a level with a start zone, and only at a legal point.
    pub fn set_start(&mut self, p: V2) -> bool {
        if self.spec.start_zone.is_none() || self.phase != Phase::Ready || !self.spec.start_legal(p)
        {
            return false;
        }
        self.spec = self.spec.with_start(p);
        self.state = self.spec.initial();
        true
    }
    pub fn start(&self) -> V2 {
        self.spec.start
    }
    // ---- sources (levels with `sources` only): placed before the first beat, then locked for the run
    pub fn sources(&self) -> &[Source] {
        &self.spec.sources
    }
    fn sources_editable(&self) -> bool {
        self.spec.source_rules.is_some() && self.phase == Phase::Ready
    }
    /// Place a source. Refused after the first beat, outside the placement area, on a region, too close to another
    /// source, with the limit reached, or of a kind the level does not offer.
    pub fn place_source(
        &mut self,
        kind: crate::field::SourceKind,
        pos: V2,
    ) -> Result<(), PlaceError> {
        let Some(r) = self.spec.source_rules.clone() else {
            return Err(PlaceError::KindUnavailable);
        };
        if !self.sources_editable() {
            return Err(PlaceError::Full);
        }
        if !r.kinds.contains(&kind) {
            return Err(PlaceError::KindUnavailable);
        }
        if let Some(e) = self.spec.site_error(pos) {
            return Err(e);
        }
        if self
            .spec
            .sources
            .iter()
            .any(|s| s.pos.dist(pos) < MIN_SEPARATION)
        {
            return Err(PlaceError::TooClose);
        }
        if self.spec.sources.len() >= r.max {
            return Err(PlaceError::Full);
        }
        let mut v = self.spec.sources.clone();
        v.push(Source { kind, pos });
        self.spec = self.spec.with_sources(&v);
        Ok(())
    }
    /// Remove source `i`. Only before the first beat.
    pub fn remove_source(&mut self, i: usize) -> bool {
        if !self.sources_editable() || i >= self.spec.sources.len() {
            return false;
        }
        let mut v = self.spec.sources.clone();
        v.remove(i);
        self.spec = self.spec.with_sources(&v);
        true
    }
    pub fn clear_sources(&mut self) -> bool {
        if !self.sources_editable() || self.spec.sources.is_empty() {
            return false;
        }
        self.spec = self.spec.with_sources(&[]);
        true
    }
    pub fn accepting_input(&self) -> bool {
        matches!(self.phase, Phase::Ready | Phase::Awaiting)
    }
    pub fn beat(&self) -> usize {
        self.state.beat
    }
    pub fn gusts_left(&self) -> usize {
        self.state.gusts_left
    }
    pub fn pos(&self) -> V2 {
        self.state.pos
    }
    /// Natural field velocity at the probe's current position.
    pub fn field_velocity(&self) -> V2 {
        self.spec.field_velocity(self.state.pos, self.state.beat)
    }
    /// Instantaneous velocity if `a` were chosen now: field + gust (a vector sum, not a trajectory).
    pub fn net_velocity(&self, a: Action) -> V2 {
        self.field_velocity() + self.spec.gust(a)
    }
    pub fn gust_vector(&self, a: Action) -> V2 {
        self.spec.gust(a)
    }
    /// Commit a decision and start advancing one beat. Rejected while a beat is in progress.
    pub fn choose(&mut self, a: Action) -> Result<(), ChooseError> {
        if !self.accepting_input() {
            return Err(if matches!(self.phase, Phase::Completed(_)) {
                ChooseError::Finished
            } else {
                ChooseError::Busy
            });
        }
        let beat = self.spec.advance(&self.state, a)?;
        // The budget is spent the moment the decision is committed, so the HUD shows it at once.
        self.state.gusts_left = beat.next.gusts_left;
        self.pending = Some(beat);
        self.phase = Phase::Advancing;
        Ok(())
    }
    /// Points of the beat currently being advanced (for animation). Empty when idle.
    pub fn pending_points(&self) -> &[V2] {
        self.pending.as_ref().map_or(&[], |b| &b.points)
    }
    pub fn pending_beat(&self) -> Option<&Beat> {
        self.pending.as_ref()
    }
    /// Finish the beat that `choose` started and evaluate success / failure.
    pub fn finish_beat(&mut self) -> Option<End> {
        if self.phase != Phase::Advancing {
            return None;
        }
        let beat = self.pending.take()?;
        self.state = beat.next.clone();
        let end = self.state.end;
        self.history.push(beat);
        match end {
            Some(e) => {
                self.phase = Phase::Completed(e);
                if e == End::Success {
                    self.won = true;
                }
            }
            None => self.phase = Phase::Awaiting,
        }
        end
    }
    /// choose + finish in one call (headless play, search, replay).
    pub fn step(&mut self, a: Action) -> Result<Option<End>, ChooseError> {
        self.choose(a)?;
        Ok(self.finish_beat())
    }
    pub fn run(&self) -> Run {
        Run {
            start: self.spec.start,
            beats: self.history.clone(),
            end: self.state.end,
        }
    }
    /// Restart from the exact initial state. A *completed* run becomes a ghost.
    pub fn retry(&mut self) -> Result<(), ChooseError> {
        if self.phase == Phase::Advancing {
            return Err(ChooseError::Busy);
        }
        if matches!(self.phase, Phase::Completed(_)) {
            self.ghosts.push(self.run());
            if self.ghosts.len() > MAX_GHOSTS {
                self.ghosts.remove(0);
            }
        }
        self.state = self.spec.initial();
        self.history.clear();
        self.pending = None;
        self.phase = Phase::Ready;
        self.attempts += 1;
        Ok(())
    }
    /// The run to replay: the finished current run, else the latest ghost.
    pub fn replayable(&self) -> Option<Run> {
        if matches!(self.phase, Phase::Completed(_)) {
            Some(self.run())
        } else {
            self.ghosts.last().cloned()
        }
    }
}
