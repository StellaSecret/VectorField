//! SYNCHRONIZE - "How must these probes be coordinated?"  Several probes, each with a fixed start, fly the same
//! fixed field. The player chooses only the moment each is released. Success is a *relationship* between the probes:
//! every probe reaches its goal and every `SyncCond` (for example "B enters the rendezvous within 0.4 s of A") holds.
//!
//! Nothing new was needed in the engine: a probe is `Level::run(field, start, t0 = delay)` exactly as in Tempo
//! (the delay is the field clock at release), several probes are several independent runs, and the only addition
//! is `entry_time`, the moment a trajectory first enters a region (the engine records only the final outcome and which
//! gates were passed).
use crate::field::{Field, V2};
use crate::level::{Level, Rules, SteerSources, SyncCond};
use crate::modes::shape::{PlaceError, Source, MIN_SEPARATION};
use crate::sim::{Outcome, RegionKind, Trajectory};

/// Delays are chosen in steps of this many seconds (the interface snaps to it; validation searches on it).
pub const DELAY_STEP: f64 = 0.1;

/// The time (field clock) at which the trajectory first enters region `region`, or `None` if it never does.
pub fn entry_time(level: &Level, t: &Trajectory, region: usize) -> Option<f64> {
    let r = &level.regions[region];
    t.points
        .iter()
        .position(|p| p.dist(r.center) <= r.radius)
        .map(|k| t.t0 + k as f64 * t.dt)
}

/// How one condition came out.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CondResult {
    pub cond: SyncCond,
    /// Entry times of `a` at `ra` and of `b` at `rb`, if they got there.
    pub at_a: Option<f64>,
    pub at_b: Option<f64>,
}
impl CondResult {
    /// `b - a`, if both arrived.
    pub fn offset(&self) -> Option<f64> {
        Some(self.at_b? - self.at_a?)
    }
    pub fn ok(&self) -> bool {
        self.offset()
            .is_some_and(|o| o >= self.cond.lo - 1e-9 && o <= self.cond.hi + 1e-9)
    }
    /// How far outside its allowed range the offset is, in seconds: negative means `b` was too early (or `a` too
    /// late), positive that `b` was too late; 0 when it is inside.
    pub fn miss(&self) -> Option<f64> {
        let o = self.offset()?;
        Some(if o < self.cond.lo {
            o - self.cond.lo
        } else if o > self.cond.hi {
            o - self.cond.hi
        } else {
            0.0
        })
    }
}

/// The result of releasing every probe at its delay.
#[derive(Clone, Debug)]
pub struct Flight {
    pub delays: Vec<f64>,
    pub trajs: Vec<Trajectory>,
    /// Did probe `i` reach its goal (through every gate)?
    pub reached: Vec<bool>,
    pub conds: Vec<CondResult>,
    pub success: bool,
}

#[derive(Clone, Debug)]
pub struct SyncSpec {
    pub level: Level,
    pub starts: Vec<V2>,
    pub goals: Vec<usize>,
    pub period: f64,
    pub max_delay: f64,
    pub conds: Vec<SyncCond>,
    /// Combination levels: the sources the player may place (they steadily change the field every probe flies), the
    /// ones placed, and the level's own field without them (`field` is this plus one term per placed source).
    pub source_rules: Option<SteerSources>,
    pub sources: Vec<Source>,
    pub field0: Field,
    pub field: Field,
    /// Combination levels: which probe may choose its start from numbered slots, and the slots (slot 0 is the default).
    pub slots: Option<(usize, Vec<V2>)>,
    /// The slot chosen now (`starts[probe]` is always that slot's place).
    pub slot: usize,
}

impl SyncSpec {
    pub fn from_level(l: &Level) -> SyncSpec {
        let Rules::Synchronize {
            starts,
            goals,
            period,
            max_delay,
            conds,
            sources,
            start_slots,
        } = &l.rules
        else {
            panic!("not a Synchronize level")
        };
        SyncSpec {
            level: l.clone(),
            starts: starts.clone(),
            goals: goals.clone(),
            period: *period,
            max_delay: *max_delay,
            conds: conds.clone(),
            source_rules: sources.clone(),
            sources: vec![],
            field0: l.field.clone(),
            field: l.field.clone(),
            slots: start_slots.clone(),
            slot: 0,
        }
    }
    /// Where `p` would be refused for a new source, ignoring the other sources.
    pub fn site_error(&self, p: V2) -> Option<PlaceError> {
        let Some(r) = &self.source_rules else {
            return Some(PlaceError::KindUnavailable);
        };
        if !r.zones.iter().any(|z| z.contains(p)) {
            return Some(PlaceError::OutsideZone);
        }
        for g in &self.level.regions {
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
    /// The same level with exactly these sources placed (a steady term each, added to the level's own field; never
    /// stacked). A level that offers no sources ignores them.
    pub fn with_sources(&self, sources: &[Source]) -> SyncSpec {
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
    /// The same level with the slotted probe started from slot `i` (a level without slots, or an unknown slot, is unchanged).
    pub fn with_slot(&self, i: usize) -> SyncSpec {
        let mut s = self.clone();
        if let Some((probe, slots)) = &self.slots {
            if let Some(p) = slots.get(i) {
                s.starts[*probe] = *p;
                s.slot = i;
            }
        }
        s
    }
    pub fn probes(&self) -> usize {
        self.starts.len()
    }
    /// Snap a delay to the step and the allowed range.
    pub fn snap(&self, d: f64) -> f64 {
        ((d / DELAY_STEP).round() * DELAY_STEP).clamp(0.0, self.max_delay)
    }
    /// Probe `i` released at `delay`: a single run of the real simulation.
    pub fn fly_one(&self, i: usize, delay: f64) -> Trajectory {
        self.level.run(&self.field, self.starts[i], delay)
    }
    /// Evaluate a set of already-flown trajectories.
    pub fn judge(&self, delays: &[f64], trajs: Vec<Trajectory>) -> Flight {
        // A probe has arrived when its run ended in its goal region. Gates are not required of every probe (a checkpoint
        // can belong to one probe's route); a condition that mentions a gate needs its entry time, which is the same thing.
        let reached: Vec<bool> = trajs
            .iter()
            .zip(&self.goals)
            .map(|(t, g)| t.outcome == Outcome::Reached(*g))
            .collect();
        let conds: Vec<CondResult> = self
            .conds
            .iter()
            .map(|c| CondResult {
                cond: *c,
                at_a: entry_time(&self.level, &trajs[c.a], c.ra),
                at_b: entry_time(&self.level, &trajs[c.b], c.rb),
            })
            .collect();
        let success = reached.iter().all(|r| *r) && conds.iter().all(|c| c.ok());
        Flight {
            delays: delays.to_vec(),
            trajs,
            reached,
            conds,
            success,
        }
    }
    /// Release every probe at its delay and judge the result.
    pub fn fly(&self, delays: &[f64]) -> Flight {
        let trajs = delays
            .iter()
            .enumerate()
            .map(|(i, d)| self.fly_one(i, *d))
            .collect();
        self.judge(delays, trajs)
    }
}

pub struct SyncGame {
    pub spec: SyncSpec,
    /// The delay chosen for each probe.
    pub delays: Vec<f64>,
    pub flights: Vec<Flight>,
    pub won: bool,
}

impl SyncGame {
    pub fn new(level: Level) -> SyncGame {
        let spec = SyncSpec::from_level(&level);
        let n = spec.probes();
        SyncGame {
            spec,
            delays: vec![0.0; n],
            flights: vec![],
            won: false,
        }
    }
    pub fn level(&self) -> &Level {
        &self.spec.level
    }
    // ---- sources (levels with `sources` only): placed between launches, as in a Shape level
    pub fn sources(&self) -> &[Source] {
        &self.spec.sources
    }
    pub fn place_source(
        &mut self,
        kind: crate::field::SourceKind,
        pos: V2,
    ) -> Result<(), PlaceError> {
        let Some(r) = self.spec.source_rules.clone() else {
            return Err(PlaceError::KindUnavailable);
        };
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
    pub fn remove_source(&mut self, i: usize) -> bool {
        if i >= self.spec.sources.len() {
            return false;
        }
        let mut v = self.spec.sources.clone();
        v.remove(i);
        self.spec = self.spec.with_sources(&v);
        true
    }
    pub fn clear_sources(&mut self) -> bool {
        if self.spec.sources.is_empty() {
            return false;
        }
        self.spec = self.spec.with_sources(&[]);
        true
    }
    /// Choose the slotted probe's start (levels with `start_slots` only). Returns whether the slot changed.
    pub fn set_slot(&mut self, i: usize) -> bool {
        let n = self.spec.slots.as_ref().map_or(0, |(_, v)| v.len());
        if i >= n || i == self.spec.slot {
            return false;
        }
        self.spec = self.spec.with_slot(i);
        true
    }
    /// Set probe `i`'s delay (snapped and clamped). Setting a delay flies nothing.
    pub fn set_delay(&mut self, i: usize, d: f64) {
        self.delays[i] = self.spec.snap(d);
    }
    pub fn nudge(&mut self, i: usize, by: f64) {
        self.set_delay(i, self.delays[i] + by);
    }
    /// Release the probes. Free and repeatable; the only place a trajectory is computed for the player.
    pub fn launch(&mut self) -> &Flight {
        let f = self.spec.fly(&self.delays);
        self.won |= f.success;
        self.flights.push(f);
        self.flights.last().unwrap()
    }
    /// A short sentence on how a flight ended for probe `i`.
    pub fn describe(&self, o: Outcome) -> String {
        self.spec.level.describe(o)
    }
}
