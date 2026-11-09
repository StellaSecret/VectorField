//! SHAPE - "How should I change the field?"  The player does not move the probe: they add local sources to the
//! field (repeller, attractor, swirl) and then launch it. The background, the start, the obstacles and the target
//! never change. Survey asks the player to infer a field from its effects; Shape asks them to create effects by
//! changing the field.
//!
//! Everything is built from what the engine already has: a source is `SourceKind::term` (the same Gaussian profile
//! as a hidden Survey source), the combined field is the background plus those terms (`Field::fixed`), an obstacle
//! is a hazard region and the goal a target region, and a launch is `Level::run`. There is deliberately no
//! function here that flies a path without recording it as a launch: the UI cannot preview a trajectory.
use crate::field::{Field, SourceKind, V2};
use crate::level::{Level, Rules, Zone};
use crate::modes::survey::PUSH_MIN;
use crate::sim::{Outcome, RegionKind, Trajectory};

/// Two sources closer than this would just be one stronger source: refuse it.
pub const MIN_SEPARATION: f64 = 0.5;

/// One placed source.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Source {
    pub kind: SourceKind,
    pub pos: V2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaceError {
    /// Outside every legal placement region.
    OutsideZone,
    /// On the target or an obstacle (the character is its label).
    OnRegion(char),
    /// Within `MIN_SEPARATION` of another source.
    TooClose,
    /// The level's source limit is reached (move or remove one).
    Full,
    /// This level does not offer that kind.
    KindUnavailable,
}
impl PlaceError {
    pub fn message(self) -> String {
        match self {
            PlaceError::OutsideZone => "Sources can only go inside the shaded placement area.".into(),
            PlaceError::OnRegion('T') => "Not on the target.".into(),
            PlaceError::OnRegion(_) => "Not on an obstacle.".into(),
            PlaceError::TooClose => "Too close to another source.".into(),
            PlaceError::Full => "No sources left: move or remove one.".into(),
            PlaceError::KindUnavailable => "This level does not offer that kind of source.".into(),
        }
    }
}

/// The fixed part of a Shape level, with the physics: everything needed to fly a configuration of sources,
/// with no player state. Validation and the game both go through this.
#[derive(Clone, Debug)]
pub struct ShapeSpec {
    pub level: Level,
    pub start: V2,
    pub goal: usize,
    pub zones: Vec<Zone>,
    pub kinds: Vec<SourceKind>,
    pub max_sources: usize,
    pub k: f64,
    pub sigma: f64,
}

impl ShapeSpec {
    pub fn from_level(l: &Level) -> ShapeSpec {
        let Rules::Shape { start, goal, zones, kinds, sources, k, sigma } = &l.rules else { panic!("not a Shape level") };
        ShapeSpec { level: l.clone(), start: *start, goal: *goal, zones: zones.clone(), kinds: kinds.clone(), max_sources: *sources, k: *k, sigma: *sigma }
    }

    /// Where `p` would be refused for a new source, ignoring the other sources.
    pub fn site_error(&self, p: V2) -> Option<PlaceError> {
        if !self.zones.iter().any(|z| z.contains(p)) {
            return Some(PlaceError::OutsideZone);
        }
        for r in &self.level.regions {
            if matches!(r.kind, RegionKind::Target | RegionKind::Hazard) && p.dist(r.center) <= r.radius {
                return Some(PlaceError::OnRegion(r.label));
            }
        }
        None
    }

    /// The field the probe flies: the background plus one term per source.
    pub fn field_with(&self, sources: &[Source]) -> Field {
        let mut f = self.level.field.clone();
        for s in sources {
            f = f.fixed(s.kind.term(s.pos, self.k, self.sigma));
        }
        f
    }

    /// Fly the probe through the field made by `sources` (one simulation).
    pub fn fly(&self, sources: &[Source]) -> Trajectory {
        self.level.run(&self.field_with(sources), self.start, 0.0)
    }

    pub fn reached(&self, t: &Trajectory) -> bool {
        self.level.reached_goal(t, self.goal)
    }

    /// The distance from a source at which its push falls to `PUSH_MIN` (the same measure Survey uses): the
    /// radius drawn around a placed source, beyond which it does not visibly change the field.
    pub fn influence_radius(&self) -> f64 {
        let push = |d: f64| self.k * d * (-d * d / (2.0 * self.sigma * self.sigma)).exp();
        if push(self.sigma) < PUSH_MIN {
            return self.sigma;
        }
        let (mut lo, mut hi) = (self.sigma, 12.0 * self.sigma);
        for _ in 0..60 {
            let mid = 0.5 * (lo + hi);
            if push(mid) >= PUSH_MIN { lo = mid } else { hi = mid }
        }
        lo
    }

    /// Every legal site on a lattice of spacing `step` (inside a zone, not on a region).
    pub fn sites(&self, step: f64) -> Vec<V2> {
        let mut out = vec![];
        for z in &self.zones {
            let b = z.bbox();
            let (nx, ny) = ((b.width() / step).floor() as i64, (b.height() / step).floor() as i64);
            for i in 0..=nx {
                for j in 0..=ny {
                    let p = V2 { x: b.min.x + i as f64 * step, y: b.min.y + j as f64 * step };
                    if self.site_error(p).is_none() && !out.iter().any(|q: &V2| q.dist(p) < 1e-9) {
                        out.push(p);
                    }
                }
            }
        }
        out
    }
}

/// One launch: the sources as they stood, and what the probe did.
#[derive(Clone, Debug)]
pub struct Attempt {
    pub sources: Vec<Source>,
    pub traj: Trajectory,
    pub success: bool,
}

pub struct ShapeGame {
    pub spec: ShapeSpec,
    pub sources: Vec<Source>,
    /// Index into `spec.kinds` of the kind the next placement uses.
    pub sel: usize,
    pub attempts: Vec<Attempt>,
    pub won: bool,
}

impl ShapeGame {
    pub fn new(level: Level) -> ShapeGame {
        ShapeGame { spec: ShapeSpec::from_level(&level), sources: vec![], sel: 0, attempts: vec![], won: false }
    }
    pub fn level(&self) -> &Level {
        &self.spec.level
    }
    pub fn kind(&self) -> SourceKind {
        self.spec.kinds[self.sel]
    }
    /// Choose the kind of the next placement by its index in `spec.kinds`.
    pub fn select(&mut self, i: usize) -> Result<(), PlaceError> {
        if i >= self.spec.kinds.len() {
            return Err(PlaceError::KindUnavailable);
        }
        self.sel = i;
        Ok(())
    }
    /// Select the next offered kind (wrapping).
    pub fn cycle_kind(&mut self) {
        self.sel = (self.sel + 1) % self.spec.kinds.len();
    }
    pub fn remaining(&self) -> usize {
        self.spec.max_sources - self.sources.len()
    }

    fn check(&self, p: V2, ignore: Option<usize>) -> Result<(), PlaceError> {
        if let Some(e) = self.spec.site_error(p) {
            return Err(e);
        }
        let near = self.sources.iter().enumerate().any(|(i, s)| Some(i) != ignore && s.pos.dist(p) < MIN_SEPARATION);
        if near {
            return Err(PlaceError::TooClose);
        }
        Ok(())
    }
    /// Would a new source of the selected kind be accepted at `p`? (Used for the placement cursor.)
    pub fn can_place(&self, p: V2) -> Result<(), PlaceError> {
        if self.sources.len() >= self.spec.max_sources {
            return Err(PlaceError::Full);
        }
        self.check(p, None)
    }
    /// Add a source of the selected kind. Returns its index.
    pub fn place(&mut self, p: V2) -> Result<usize, PlaceError> {
        self.can_place(p)?;
        self.sources.push(Source { kind: self.kind(), pos: p });
        Ok(self.sources.len() - 1)
    }
    /// Index of the source whose marker is under `p` (within `radius`), the nearest first.
    pub fn source_at(&self, p: V2, radius: f64) -> Option<usize> {
        self.sources.iter().enumerate().filter(|(_, s)| s.pos.dist(p) <= radius).min_by(|a, b| a.1.pos.dist(p).total_cmp(&b.1.pos.dist(p))).map(|(i, _)| i)
    }
    /// Move source `i` to `p` (same rules as placing, except it does not count against the limit or itself).
    pub fn move_source(&mut self, i: usize, p: V2) -> Result<(), PlaceError> {
        self.check(p, Some(i))?;
        self.sources[i].pos = p;
        Ok(())
    }
    pub fn remove(&mut self, i: usize) -> bool {
        if i < self.sources.len() {
            self.sources.remove(i);
            true
        } else {
            false
        }
    }
    pub fn clear(&mut self) {
        self.sources.clear();
    }
    /// The field as it stands now (background plus the placed sources): what the arrows show.
    pub fn field(&self) -> Field {
        self.spec.field_with(&self.sources)
    }
    /// Launch the probe through the combined field. Free and repeatable (even with no sources, which flies the
    /// background alone). This is the only way a path is ever computed for the player.
    pub fn launch(&mut self) -> &Attempt {
        let traj = self.spec.fly(&self.sources);
        let success = self.spec.reached(&traj);
        self.won |= success;
        self.attempts.push(Attempt { sources: self.sources.clone(), traj, success });
        self.attempts.last().unwrap()
    }
    /// A short sentence on how the last launch ended, for the message line.
    pub fn describe(&self, o: Outcome) -> String {
        self.spec.level.describe(o)
    }
}
