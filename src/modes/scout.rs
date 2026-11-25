//! Scout, then call it: a two-stage combination of Survey and Predict (docs/07-combinations.md).
//!
//! A hidden source is added to the visible field every flight sees. In stage one the player launches a few scouts and
//! reads the push they feel (as in Survey: observed velocity minus what the visible field alone would give). In stage
//! two the player calls which exit the *main probe* will leave through. The call is committed before the probe flies;
//! the true flight is then shown. `streak` correct calls in a row complete the level. The hidden source is drawn at
//! random from the pool each attempt (seeded, so a play is reproducible) and never repeats the previous one, so the
//! evidence from one attempt is no use in the next and a remembered order is no use at all.
use crate::field::{Field, SourceKind, V2};
use crate::level::{GustDirs, Level, Rules, ScoutAct, ScoutRules};
use crate::modes::steer::{Action, End, SteerSpec};
use crate::modes::survey::{PUSH_EVERY, PUSH_MIN};
use crate::modes::Lcg;
use crate::sim::{Outcome, Rect};

/// One scout: where it started, the trail it flew, and the push it measured along it (position, push vector).
#[derive(Clone, Debug)]
pub struct Scout {
    pub start: V2,
    pub trail: Vec<V2>,
    pub pushes: Vec<(V2, V2)>,
}

#[derive(Clone, Debug)]
pub struct ScoutSpec {
    pub level: Level,
    pub rules: ScoutRules,
}

impl ScoutSpec {
    pub fn from_level(l: &Level) -> ScoutSpec {
        let Rules::Scout(r) = &l.rules else {
            panic!("not a Scout level")
        };
        ScoutSpec {
            level: l.clone(),
            rules: r.clone(),
        }
    }
    /// The hidden source `pool[entry]`.
    pub fn hidden(&self, entry: usize) -> (SourceKind, V2) {
        self.rules.pool[entry]
    }
    /// The field every flight sees while `pool[entry]` is hidden in it: the visible one plus that source.
    pub fn truth_field(&self, entry: usize) -> Field {
        let (kind, pos) = self.hidden(entry);
        self.level
            .field
            .clone()
            .fixed(kind.term(pos, self.rules.k, self.rules.sigma))
    }
    /// The same level with a *guessed* hidden source (what a reader who has located it would simulate).
    pub fn field_with(&self, kind: SourceKind, pos: V2) -> Field {
        self.level
            .field
            .clone()
            .fixed(kind.term(pos, self.rules.k, self.rules.sigma))
    }
    /// Which exit (an index into `rules.exits`) the main probe leaves through in `field`, and the path it flies.
    pub fn exit_in(&self, field: &Field) -> (Option<usize>, Vec<V2>) {
        let t = self.level.run(field, self.rules.start, 0.0);
        let e = match t.outcome {
            Outcome::Reached(region) => self.rules.exits.iter().position(|&r| r == region),
            _ => None,
        };
        (e, t.points)
    }
    /// The true exit when `pool[entry]` is hidden in the field.
    pub fn true_exit(&self, entry: usize) -> (Option<usize>, Vec<V2>) {
        self.exit_in(&self.truth_field(entry))
    }
    /// The exit the visible arrows alone would give (what a player who never scouts would predict by simulation).
    pub fn visible_exit(&self) -> Option<usize> {
        self.exit_in(&self.level.field).0
    }
    /// The acts that make up the final choice (one, or those of a `Seq`) and how many options each offers. In a
    /// sequence a `Cancel` and a `Plan` also offer "none" (last).
    pub fn components(&self) -> Vec<(&ScoutAct, usize)> {
        let (acts, in_seq): (Vec<&ScoutAct>, bool) = match &self.rules.act {
            ScoutAct::Seq(v) => (v.iter().collect(), true),
            a => (vec![a], false),
        };
        acts.into_iter()
            .map(|a| {
                let n = match a {
                    ScoutAct::Exit => self.rules.exits.len(),
                    ScoutAct::Release { starts, .. } => starts.len(),
                    ScoutAct::Cancel { sites, .. } => sites.len() + in_seq as usize,
                    ScoutAct::Plan { plans, .. } => plans.len() + in_seq as usize,
                    ScoutAct::Phase { phases, .. } => phases.len(),
                    ScoutAct::Dial { levels, .. } => levels.len(),
                    ScoutAct::Seq(_) => panic!("a sequence cannot contain a sequence"),
                };
                (a, n)
            })
            .collect()
    }
    /// How many options the final act offers (the product over the components).
    pub fn options(&self) -> usize {
        self.components().iter().map(|(_, n)| n).product()
    }
    /// The choice in each component for option `opt` (the first component is the most significant).
    pub fn decode(&self, opt: usize) -> Vec<usize> {
        let counts: Vec<usize> = self.components().iter().map(|(_, n)| *n).collect();
        let mut rest = opt;
        let mut out = vec![0; counts.len()];
        for i in (0..counts.len()).rev() {
            out[i] = rest % counts[i];
            rest /= counts[i];
        }
        out
    }
    /// The option made of one choice per component.
    pub fn encode(&self, parts: &[usize]) -> usize {
        self.components()
            .iter()
            .zip(parts)
            .fold(0, |acc, ((_, n), p)| acc * n + p)
    }
    /// The choice in component `i` that does nothing: the default slot, phase or dial, "no source", "no gust".
    pub fn neutral(&self, i: usize) -> Option<usize> {
        let comps = self.components();
        let (act, n) = comps[i];
        match act {
            ScoutAct::Release { .. } | ScoutAct::Phase { .. } | ScoutAct::Dial { .. } => Some(0),
            ScoutAct::Cancel { .. } | ScoutAct::Plan { .. }
                if matches!(self.rules.act, ScoutAct::Seq(_)) =>
            {
                Some(n - 1)
            }
            _ => None,
        }
    }
    /// The label of option `opt`: the exit's letter for a call, otherwise its number.
    pub fn option_label(&self, opt: usize) -> String {
        match &self.rules.act {
            ScoutAct::Exit => self.level.regions[self.rules.exits[opt]].label.to_string(),
            ScoutAct::Seq(_) => self
                .decode(opt)
                .iter()
                .map(|p| (p + 1).to_string())
                .collect::<Vec<_>>()
                .join("-"),
            _ => (opt + 1).to_string(),
        }
    }
    /// Does option `opt` succeed when every flight sees `field` (the visible field plus whatever hidden source is
    /// assumed)? Also the path the main probe flies. For `Exit` the "option" is the call, so it succeeds exactly when
    /// the probe leaves through that exit in `field`; the reader uses this on a field with an *estimated* source.
    pub fn try_option(&self, field: &Field, opt: usize) -> (bool, Vec<V2>) {
        if let ScoutAct::Exit = &self.rules.act {
            let (e, path) = self.exit_in(field);
            return (e == Some(opt), path);
        }
        // every other act modifies the flight: where it starts, when, the dial, a source added, a gust plan
        let (mut f, mut start, mut t0) = (field.clone(), self.rules.start, 0.0);
        let mut plan: Option<(&Vec<Option<usize>>, GustDirs, f64, f64)> = None;
        let mut goal = 0;
        let choice = self.decode(opt);
        for ((act, _), &c) in self.components().into_iter().zip(&choice) {
            match act {
                ScoutAct::Release { starts, goal: g } => {
                    start = starts[c];
                    goal = *g;
                }
                ScoutAct::Cancel { sites, goal: g } => {
                    if let Some(&(kind, pos)) = sites.get(c) {
                        f = f.fixed(kind.term(pos, self.rules.k, self.rules.sigma));
                    }
                    goal = *g;
                }
                ScoutAct::Plan {
                    plans,
                    goal: g,
                    dirs,
                    strength,
                    beat_time,
                } => {
                    if let Some(p) = plans.get(c) {
                        plan = Some((p, *dirs, *strength, *beat_time));
                    }
                    goal = *g;
                }
                ScoutAct::Phase { phases, goal: g } => {
                    t0 = phases[c];
                    goal = *g;
                }
                ScoutAct::Dial { levels, goal: g } => {
                    f = f.with_intensity(levels[c]);
                    goal = *g;
                }
                ScoutAct::Exit | ScoutAct::Seq(_) => unreachable!(),
            }
        }
        match plan {
            None => {
                let t = self.level.run(&f, start, t0);
                (self.level.reached_goal(&t, goal), t.points)
            }
            Some((p, dirs, strength, beat_time)) => {
                assert!(
                    t0 == 0.0,
                    "a gust plan cannot be combined with a launch phase"
                );
                let mut l = self.level.clone();
                l.field = f;
                l.rules = Rules::Steer {
                    start,
                    beats: p.len(),
                    gust_budget: p.len(),
                    gust_strength: strength,
                    dirs,
                    beat_time,
                    goal,
                    period: None,
                    start_zone: None,
                    sources: None,
                    dial: None,
                    companion: None,
                };
                let spec = SteerSpec::from_level(&l);
                let acts: Vec<Action> = p
                    .iter()
                    .map(|a| a.map_or(Action::Wait, Action::Gust))
                    .collect();
                match spec.replay(&acts) {
                    Ok(run) => (run.end == Some(End::Success), run.path()),
                    Err(_) => (false, vec![start]),
                }
            }
        }
    }
    /// The options that succeed when `pool[entry]` is hidden in the field, in order.
    pub fn winners(&self, entry: usize) -> Vec<usize> {
        let f = self.truth_field(entry);
        (0..self.options())
            .filter(|&o| self.try_option(&f, o).0)
            .collect()
    }
    /// A careful reader's choice for a source believed to be `kind` at `h`: the option that succeeds under the most
    /// estimates within `radius` of `h` (the estimate and eight points around it), first on ties, and only if it
    /// succeeds under the estimate itself. A person picks the safe choice, not the one that works for the exact guess.
    pub fn pick_robust(&self, kind: SourceKind, h: V2, radius: f64) -> Option<usize> {
        let mut fields = vec![self.field_with(kind, h)];
        for i in 0..8 {
            let a = i as f64 * std::f64::consts::TAU / 8.0;
            fields.push(self.field_with(
                kind,
                V2 {
                    x: h.x + radius * a.cos(),
                    y: h.y + radius * a.sin(),
                },
            ));
        }
        let mut best: Option<(usize, usize)> = None;
        for o in 0..self.options() {
            if !self.try_option(&fields[0], o).0 {
                continue;
            }
            let n = fields.iter().filter(|f| self.try_option(f, o).0).count();
            if best.is_none_or(|(_, c)| n > c) {
                best = Some((o, n));
            }
        }
        best.map(|(o, _)| o)
    }
    /// The choice of a reader who aims at the middle of a window: among the options that succeed in `field`, the middle
    /// of the longest run of adjacent ones (the first such run on a tie). For an ordered control such as the launch
    /// phase, the first working option sits at the edge of its window, where a small error in the estimate of the
    /// source loses it; a person would pick the middle.
    pub fn pick_center(&self, field: &Field) -> Option<usize> {
        let ok: Vec<bool> = (0..self.options())
            .map(|o| self.try_option(field, o).0)
            .collect();
        let (mut best, mut best_len) = (None, 0usize);
        let mut i = 0;
        while i < ok.len() {
            if ok[i] {
                let mut j = i;
                while j + 1 < ok.len() && ok[j + 1] {
                    j += 1;
                }
                if j - i + 1 > best_len {
                    best_len = j - i + 1;
                    best = Some((i + j) / 2);
                }
                i = j + 1;
            } else {
                i += 1;
            }
        }
        best
    }
    /// The option a reader who believes the field is `field` would choose: the first that succeeds there.
    pub fn pick_in(&self, field: &Field) -> Option<usize> {
        (0..self.options()).find(|&o| self.try_option(field, o).0)
    }
    /// May a scout start at `p`?
    pub fn legal_scout(&self, p: V2) -> bool {
        let z: Rect = self.rules.zone;
        p.x >= z.min.x - 1e-9
            && p.x <= z.max.x + 1e-9
            && p.y >= z.min.y - 1e-9
            && p.y <= z.max.y + 1e-9
    }
    /// Fly a scout from `start` with `pool[entry]` hidden in the field, for `scout_time` seconds, and measure the push.
    pub fn scout(&self, entry: usize, start: V2) -> Scout {
        let truth = self.truth_field(entry);
        let t = self.level.run(&truth, start, 0.0);
        let n =
            ((self.rules.scout_time / t.dt).round() as usize).min(t.points.len().saturating_sub(1));
        let p = &t.points[..=n];
        let (k, h) = (
            ((PUSH_EVERY / t.dt).round() as usize).max(1),
            ((0.1 / t.dt).round() as usize).max(1),
        );
        let mut pushes = vec![];
        let mut i = h;
        while i + h < p.len() {
            let v = (p[i + h] - p[i - h]) * (1.0 / (2.0 * h as f64 * t.dt));
            let push = v - self.level.field.eval(p[i], t.t0 + i as f64 * t.dt);
            if push.len() >= PUSH_MIN {
                pushes.push((p[i], push));
            }
            i += k;
        }
        Scout {
            start,
            trail: p.to_vec(),
            pushes,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScoutPhase {
    /// Launching scouts; the call has not been made.
    Scouting,
    /// The call is made and the true flight is shown; `next_attempt` starts the next hidden source.
    Called,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScoutError {
    /// Not in the scouting phase.
    WrongPhase,
    OutsideZone,
    NoScoutsLeft,
    /// The call named an option that does not exist.
    BadExit,
}

/// One committed call.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Call {
    pub attempt: usize,
    /// Which pool entry was hidden in that attempt.
    pub entry: usize,
    pub chosen: usize,
    pub truth: Option<usize>,
    pub correct: bool,
}

#[derive(Clone, Debug)]
pub struct ScoutGame {
    pub spec: ScoutSpec,
    pub attempt: usize,
    /// The pool entry hidden in each attempt so far (drawn when first needed).
    pub order: Vec<usize>,
    seed: u64,
    pub phase: ScoutPhase,
    pub scouts: Vec<Scout>,
    pub calls: Vec<Call>,
    /// Correct calls in a row, now and at best.
    pub streak: usize,
    pub best_streak: usize,
    pub won: bool,
    /// The main probe's true path after a call (empty before).
    pub last_path: Vec<V2>,
    /// The choice made so far in each component of the final choice (before it is committed).
    pub sel: Vec<Option<usize>>,
}

impl ScoutGame {
    /// A play seeded with the level's own seed (reproducible; the app uses `with_seed` with the time).
    pub fn new(level: Level) -> ScoutGame {
        let seed = level.seed;
        ScoutGame::with_seed(level, seed)
    }
    pub fn with_seed(level: Level, seed: u64) -> ScoutGame {
        let mut g = ScoutGame {
            spec: ScoutSpec::from_level(&level),
            attempt: 0,
            order: vec![],
            seed,
            phase: ScoutPhase::Scouting,
            scouts: vec![],
            calls: vec![],
            streak: 0,
            best_streak: 0,
            won: false,
            last_path: vec![],
            sel: vec![],
        };
        g.sel = vec![None; g.spec.components().len()];
        g.draw_entry();
        g
    }
    /// Draw the hidden source of the next attempt: random, but never the one just used.
    fn draw_entry(&mut self) {
        let n = self.spec.rules.pool.len();
        let mut rng =
            Lcg(self.seed ^ (self.order.len() as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15));
        // warm the generator up: nearby seeds give correlated first values
        for _ in 0..4 {
            rng.next();
        }
        let e = match self.order.last() {
            Some(&prev) if n > 1 => {
                let r = rng.below(n - 1);
                if r >= prev {
                    r + 1
                } else {
                    r
                }
            }
            _ => rng.below(n),
        };
        self.order.push(e);
    }
    /// The pool entry hidden in the current attempt.
    pub fn entry(&self) -> usize {
        self.order[self.attempt]
    }
    pub fn level(&self) -> &Level {
        &self.spec.level
    }
    pub fn scouts_left(&self) -> usize {
        self.spec.rules.scouts.saturating_sub(self.scouts.len())
    }
    /// Launch a scout from `p`; it flies at once and its trail and push are recorded.
    pub fn place_scout(&mut self, p: V2) -> Result<(), ScoutError> {
        if self.phase != ScoutPhase::Scouting {
            return Err(ScoutError::WrongPhase);
        }
        if !self.spec.legal_scout(p) {
            return Err(ScoutError::OutsideZone);
        }
        if self.scouts_left() == 0 {
            return Err(ScoutError::NoScoutsLeft);
        }
        let s = self.spec.scout(self.entry(), p);
        self.scouts.push(s);
        Ok(())
    }
    /// Commit the call: the main probe flies and the call is judged. Returns whether it was right.
    pub fn call(&mut self, exit: usize) -> Result<bool, ScoutError> {
        if self.phase != ScoutPhase::Scouting {
            return Err(ScoutError::WrongPhase);
        }
        if exit >= self.spec.options() {
            return Err(ScoutError::BadExit);
        }
        let (correct, path) = self
            .spec
            .try_option(&self.spec.truth_field(self.entry()), exit);
        let truth = self.spec.winners(self.entry()).first().copied();
        self.calls.push(Call {
            attempt: self.attempt,
            entry: self.entry(),
            chosen: exit,
            truth,
            correct,
        });
        self.streak = if correct { self.streak + 1 } else { 0 };
        self.best_streak = self.best_streak.max(self.streak);
        if self.streak >= self.spec.rules.streak {
            self.won = true;
        }
        self.last_path = path;
        self.phase = ScoutPhase::Called;
        Ok(correct)
    }
    /// Move on to the next attempt (the next hidden source) with fresh scouts. Only after a call.
    pub fn next_attempt(&mut self) -> bool {
        if self.phase != ScoutPhase::Called {
            return false;
        }
        self.attempt += 1;
        if self.order.len() <= self.attempt {
            self.draw_entry();
        }
        self.phase = ScoutPhase::Scouting;
        self.scouts.clear();
        self.last_path.clear();
        self.sel = vec![None; self.spec.components().len()];
        true
    }
    /// Pick the choice in component `comp` (before committing). False if there is no such choice.
    pub fn select(&mut self, comp: usize, i: usize) -> bool {
        let ok = self.phase == ScoutPhase::Scouting
            && self
                .spec
                .components()
                .get(comp)
                .is_some_and(|(_, n)| i < *n);
        if ok {
            self.sel[comp] = Some(i);
        }
        ok
    }
    /// Every component has a choice.
    pub fn ready(&self) -> bool {
        self.sel.iter().all(|c| c.is_some())
    }
    /// Commit the selected choices together (the option they make).
    pub fn commit(&mut self) -> Result<bool, ScoutError> {
        if !self.ready() {
            return Err(ScoutError::BadExit);
        }
        let parts: Vec<usize> = self.sel.iter().map(|c| c.unwrap()).collect();
        let opt = self.spec.encode(&parts);
        self.call(opt)
    }
    pub fn last_call(&self) -> Option<&Call> {
        self.calls.last()
    }
}
