//! Automated evidence for *two-decision* combination levels (docs/07-combinations.md). A level qualifies only if
//! two decisions from different mechanics are each necessary, and every number here comes from the real simulation.
//!
//! The levels covered are a Shape level with a launch phase or a background dial (`Rules::Shape { period | intensity }`)
//! and a Steer level with a launch phase (`Rules::Steer { period }`). Each has a *control* decision with a numeric
//! setting (a phase in seconds or a dial value) and a *plan* decision (which source to place, or which gusts to play).
//!
//! What is measured, and the bar it is held to (see `ablation_holds`):
//! - *ablation of the plan*: no control setting wins with an empty plan (no source, no gusts);
//! - *ablation of the control*: at the default setting no plan wins, and none wins at any setting within the *dead
//!   band* of the default (1.5 s of phase, or 15% of the dial range), so a player who never touches the control
//!   and a player who nudges it a little both fail;
//! - *both must be possible*: some setting admits a win;
//! - *complementarity*: the plans that win at one setting differ from those at another (`coupling` is the mean
//!   Jaccard distance between those sets; 0 would mean the two decisions are independent of each other);
//! - *informed against blind*: a player who scans the control with free empty launches and takes the setting that
//!   gets closest to the target finds a win more often than a random setting does;
//! - *robustness*: the ablations still hold when the target and obstacle are moved by 0.3 either way.
use crate::field::V2;
use crate::level::{Level, Rules};
use crate::modes::shape::{ShapeSpec, Source};
use crate::modes::steer::{Action, SteerSpec};
use crate::modes::synchronize::{entry_time, SyncSpec};
use crate::sim::Outcome;
use crate::sim::Rect;
use crate::validate_shape::{reader_sites, shifted};
use crate::validate_steer::enumerate;
use std::collections::BTreeSet;

/// Settings tried per axis (a phase grid of 0.5 s for a 12 s period; a dial grid of range/24).
pub const PHASE_SETTINGS: usize = 24;
pub const DIAL_SETTINGS: usize = 25;
/// The dead band around the default control setting: seconds of phase, or the fraction of the dial range.
pub const PHASE_DEAD_S: f64 = 1.5;
pub const DIAL_DEAD_FRAC: f64 = 0.15;
/// For a start-point control: how far (world units) from the default start counts as "near it", and the lattice spacing.
pub const START_DEAD: f64 = 1.0;
pub const START_STEP: f64 = 0.25;
/// Lattice spacing of the source search (coarser than the Shape chapter's 0.25, because every setting repeats it).
pub const SITE_STEP: f64 = 0.3;
/// How far the target and obstacle are moved in the robustness check.
pub const MOVE: f64 = 0.3;

/// The control a combination level adds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Axis {
    /// Launch phase in `0..period`.
    Phase(f64),
    /// Background intensity in `lo..=hi`.
    Dial(f64, f64),
    /// Start point inside a zone; the second field is the default start. A setting is an index into `start_points`.
    Start(Rect, V2),
    /// Sources placed before a Steer run. A setting is an index into the candidate list built by `steer_report`:
    /// 0 is no source (the default), then every kind at every legal site of a lattice.
    Source,
    /// One probe of a Synchronize level starts from one of this many numbered slots; setting 0 is the default slot.
    Slots(usize, &'static str),
}

/// The start points tried: the default first, then a lattice over the zone (every `START_STEP` in y, and in x when the
/// zone has width), skipping the default's own lattice site.
pub fn start_points(zone: Rect, default: V2) -> Vec<V2> {
    let mut v = vec![default];
    let nx = ((zone.max.x - zone.min.x) / START_STEP).round() as usize;
    let ny = ((zone.max.y - zone.min.y) / START_STEP).round() as usize;
    for i in 0..=nx {
        for j in 0..=ny {
            let p = V2 {
                x: zone.min.x + i as f64 * START_STEP,
                y: zone.min.y + j as f64 * START_STEP,
            };
            if p.dist(default) > 1e-9 {
                v.push(p);
            }
        }
    }
    v
}
impl Axis {
    pub fn name(self) -> &'static str {
        match self {
            Axis::Phase(_) => "launch phase",
            Axis::Dial(..) => "dial",
            Axis::Start(..) => "start point",
            Axis::Source => "source",
            Axis::Slots(_, name) => name,
        }
    }
    /// The start point of setting `i` (only for `Axis::Start`).
    pub fn point(self, i: usize) -> Option<V2> {
        match self {
            Axis::Start(z, d) => start_points(z, d).get(i).copied(),
            _ => None,
        }
    }
    pub fn settings(self) -> Vec<f64> {
        match self {
            Axis::Phase(p) => (0..PHASE_SETTINGS)
                .map(|i| p * i as f64 / PHASE_SETTINGS as f64)
                .collect(),
            Axis::Dial(a, b) => (0..DIAL_SETTINGS)
                .map(|i| a + (b - a) * i as f64 / (DIAL_SETTINGS - 1) as f64)
                .collect(),
            Axis::Start(z, d) => (0..start_points(z, d).len()).map(|i| i as f64).collect(),
            // the candidate list depends on the level's placement area, so `steer_report` builds the indices itself
            Axis::Source => vec![],
            Axis::Slots(n, _) => (0..n).map(|i| i as f64).collect(),
        }
    }
    /// Where the control starts when the level opens (phase 0, or the middle of the dial).
    pub fn default_setting(self) -> f64 {
        match self {
            Axis::Phase(_) => 0.0,
            Axis::Dial(a, b) => (a + b) * 0.5,
            Axis::Start(..) | Axis::Source | Axis::Slots(..) => 0.0,
        }
    }
    /// Is `s` within the dead band of the default?
    pub fn in_dead_band(self, s: f64) -> bool {
        match self {
            Axis::Phase(p) => {
                let d = s.rem_euclid(p);
                d.min(p - d) <= PHASE_DEAD_S + 1e-9
            }
            Axis::Dial(a, b) => {
                (s - self.default_setting()).abs() <= DIAL_DEAD_FRAC * (b - a) + 1e-9
            }
            Axis::Start(_, d) => self
                .point(s as usize)
                .is_some_and(|p| p.dist(d) <= START_DEAD + 1e-9),
            // no distance is meaningful for a source: only "no source at all" is the default
            Axis::Source | Axis::Slots(..) => s == 0.0,
        }
    }
}

/// The axis a level adds, or `None` for a level with no extra control. A level with two is not a combination of
/// this kind and is rejected.
pub fn axis_of(level: &Level) -> Option<Axis> {
    match &level.rules {
        Rules::Shape {
            period, intensity, ..
        } => match (period, intensity) {
            (Some(p), None) => Some(Axis::Phase(*p)),
            (None, Some((a, b))) => Some(Axis::Dial(*a, *b)),
            _ => None,
        },
        Rules::Steer {
            period: Some(p),
            start_zone: None,
            ..
        } => Some(Axis::Phase(*p)),
        Rules::Steer {
            period: None,
            start_zone: Some(z),
            start,
            sources: None,
            ..
        } => Some(Axis::Start(*z, *start)),
        Rules::Steer {
            period: None,
            start_zone: None,
            sources: Some(_),
            ..
        } => Some(Axis::Source),
        Rules::Steer {
            period: None,
            start_zone: None,
            sources: None,
            dial: Some(v),
            ..
        } => Some(Axis::Slots(v.len(), "dial setting")),
        Rules::Steer {
            period: None,
            start_zone: None,
            sources: None,
            dial: None,
            companion: Some(c),
            ..
        } => Some(Axis::Slots(c.delays.len(), "drone release")),
        Rules::Tempo { dial: Some(v), .. } => Some(Axis::Slots(v.len(), "dial setting")),
        Rules::Synchronize {
            sources: Some(_), ..
        } => Some(Axis::Source),
        Rules::Synchronize {
            start_slots: Some((_, v)),
            sources: None,
            ..
        } => Some(Axis::Slots(v.len(), "start slot")),
        Rules::Synchronize {
            dial: Some(v),
            start_slots: None,
            sources: None,
            ..
        } => Some(Axis::Slots(v.len(), "dial setting")),
        _ => None,
    }
}

/// Everything measured about one two-decision level.
#[derive(Clone, Debug, Default)]
pub struct ComboReport {
    /// For each setting, the winning plans (an encoded gust plan, delay pair or launch phase); empty for Shape reports.
    pub win_sets: Vec<BTreeSet<Vec<u8>>>,
    /// The ordinal value of each setting (a dial value, a phase), in the order of `win_sets`.
    pub setting_values: Vec<f64>,
    pub axis: &'static str,
    pub settings: usize,
    pub default_setting: f64,
    /// Settings at which the empty plan (no source, no gusts) wins. Must be 0.
    pub none_wins: usize,
    /// Winning plans at the default setting. Must be 0.
    pub default_wins: usize,
    /// Winning plans summed over every setting in the dead band of the default. Must be 0.
    pub dead_wins: usize,
    /// Settings that admit at least one winning plan, and the width of the dead band in settings.
    pub settings_with_wins: usize,
    pub dead_settings: usize,
    /// Winning (setting, plan) pairs as a fraction of all pairs.
    pub win_fraction: f64,
    /// Mean Jaccard distance between the win sets of different settings (0: independent decisions).
    pub coupling: f64,
    /// The setting an informed player picks by scanning with empty launches, and whether it admits a win.
    pub informed_setting: f64,
    pub informed_admits_win: bool,
    /// The share of plans at that setting that win, against the share of settings that admit any win (a random pick).
    pub informed_plan_fraction: f64,
    pub blind_setting_fraction: f64,
    /// The most plans that win at any one setting.
    pub peak_wins: usize,
    /// Was the informed-versus-blind comparison measured for this kind of level? (Not yet for Synchronize.)
    pub informed_measured: bool,
    /// Synchronize only: the smallest `max(delay)` over every winning (source, delays) pair, in seconds, i.e. how close
    /// to releasing everything at once any solution comes (`None` for other levels, or when nothing wins).
    pub nearest_win_s: Option<f64>,
}

impl ComboReport {
    /// Both decisions are necessary: the three ablations hold and a win exists.
    pub fn ablation_holds(&self) -> bool {
        self.none_wins == 0
            && self.default_wins == 0
            && self.dead_wins == 0
            && self.settings_with_wins > 0
    }
}

/// Mean Jaccard distance between the sets (a deterministic sample of at most ~60 sets).
fn mean_jaccard<T: Ord>(sets: &[BTreeSet<T>]) -> f64 {
    let sets: Vec<&BTreeSet<T>> = sets.iter().filter(|s| !s.is_empty()).collect();
    let stride = (sets.len() / 60).max(1);
    let (mut sum, mut n) = (0.0, 0usize);
    for a in (0..sets.len()).step_by(stride) {
        for b in ((a + 1)..sets.len()).step_by(stride) {
            let inter = sets[a].intersection(sets[b]).count();
            let union = sets[a].len() + sets[b].len() - inter;
            sum += 1.0 - inter as f64 / union.max(1) as f64;
            n += 1;
        }
    }
    if n == 0 {
        0.0
    } else {
        sum / n as f64
    }
}

// ------------------------------------------------------------------ Shape levels
fn shape_fly(
    spec: &ShapeSpec,
    axis: Axis,
    setting: f64,
    plan: &[Source],
) -> crate::sim::Trajectory {
    match axis {
        Axis::Phase(_) => spec.fly_at(plan, setting, spec.default_intensity()),
        Axis::Dial(..) => spec.fly_at(plan, 0.0, setting),
        Axis::Start(..) | Axis::Source | Axis::Slots(..) => {
            unreachable!("a Shape level has no start-point, slot or source control")
        }
    }
}

/// Closest the path comes to the target (the reader's "how near did it get" read-out of a free launch).
fn closest_approach(spec: &ShapeSpec, t: &crate::sim::Trajectory) -> f64 {
    let c = spec.level.regions[spec.goal].center;
    t.points
        .iter()
        .map(|p| p.dist(c))
        .fold(f64::INFINITY, f64::min)
}

pub fn shape_report(level: &Level, step: f64) -> ComboReport {
    let spec = ShapeSpec::from_level(level);
    let axis = axis_of(level).expect("a Shape combination level has exactly one extra control");
    let settings = axis.settings();
    let plans: Vec<Source> = {
        let sites = spec.sites(step);
        spec.kinds
            .iter()
            .flat_map(|&k| sites.iter().map(move |&p| Source { kind: k, pos: p }))
            .collect()
    };
    let mut r = ComboReport {
        axis: axis.name(),
        settings: settings.len(),
        default_setting: axis.default_setting(),
        ..Default::default()
    };
    let mut win_sets: Vec<BTreeSet<usize>> = vec![];
    let mut total_wins = 0usize;
    let mut per_setting = vec![0usize; settings.len()];
    for (si, &s) in settings.iter().enumerate() {
        if spec.reached(&shape_fly(&spec, axis, s, &[])) {
            r.none_wins += 1;
        }
        let set: BTreeSet<usize> = (0..plans.len())
            .filter(|&i| spec.reached(&shape_fly(&spec, axis, s, &[plans[i]])))
            .collect();
        per_setting[si] = set.len();
        total_wins += set.len();
        if axis.in_dead_band(s) {
            r.dead_settings += 1;
            r.dead_wins += set.len();
        }
        win_sets.push(set);
    }
    // The default setting is on the grid for both axes (phase 0; the dial's middle is the 13th of 25).
    let di = settings
        .iter()
        .position(|s| (s - axis.default_setting()).abs() < 1e-9)
        .expect("default is a grid point");
    r.default_wins = per_setting[di];
    r.settings_with_wins = per_setting.iter().filter(|n| **n > 0).count();
    r.win_fraction = total_wins as f64 / (settings.len() * plans.len()).max(1) as f64;
    r.coupling = mean_jaccard(&win_sets);
    r.peak_wins = per_setting.iter().copied().max().unwrap_or(0);
    // informed: the setting whose empty launch gets closest to the target; then a reader chooses the source
    let (ii, _) = settings
        .iter()
        .enumerate()
        .map(|(i, &s)| (i, closest_approach(&spec, &shape_fly(&spec, axis, s, &[]))))
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .unwrap();
    r.informed_setting = settings[ii];
    r.informed_admits_win = per_setting[ii] > 0;
    let path = shape_fly(&spec, axis, settings[ii], &[]);
    let cands = reader_sites(&spec, &path, step);
    let w = cands
        .iter()
        .filter(|(k, p)| {
            spec.reached(&shape_fly(
                &spec,
                axis,
                settings[ii],
                &[Source { kind: *k, pos: *p }],
            ))
        })
        .count();
    r.informed_plan_fraction = w as f64 / cands.len().max(1) as f64;
    r.blind_setting_fraction = r.settings_with_wins as f64 / settings.len() as f64;
    r.informed_measured = true;
    r
}

// ------------------------------------------------------------------ Steer levels
fn encode(a: &[Action]) -> Vec<u8> {
    a.iter()
        .map(|x| match x {
            Action::Wait => 0,
            Action::Gust(i) => *i as u8 + 1,
        })
        .collect()
}

pub fn steer_report(level: &Level) -> ComboReport {
    let base = SteerSpec::from_level(level);
    let axis =
        axis_of(level).expect("a Steer combination level has a launch phase or a start zone");
    // The spec for one setting; a start point that is not legal (outside the zone or inside a region) is skipped.
    // for a source control: no source, then every kind at every legal site
    let candidates: Vec<Source> = match (&axis, &base.source_rules) {
        (Axis::Source, Some(r)) => {
            let sites = base.sites(SITE_STEP);
            r.kinds
                .iter()
                .flat_map(|&k| sites.iter().map(move |&p| Source { kind: k, pos: p }))
                .collect()
        }
        _ => vec![],
    };
    let spec_at = |s: f64| -> Option<SteerSpec> {
        match axis {
            Axis::Phase(_) => Some(base.with_clock(s)),
            Axis::Start(..) => axis
                .point(s as usize)
                .filter(|p| base.start_legal(*p))
                .map(|p| base.with_start(p)),
            Axis::Source => {
                if s == 0.0 {
                    Some(base.clone())
                } else {
                    candidates
                        .get(s as usize - 1)
                        .map(|c| base.with_sources(&[*c]))
                }
            }
            Axis::Slots(..) if base.companion.is_some() => Some(base.with_drone(s as usize)),
            Axis::Slots(..) => Some(base.with_dial(s as usize)),
            Axis::Dial(..) => None,
        }
    };
    let all_settings: Vec<f64> = match axis {
        Axis::Source => (0..=candidates.len()).map(|i| i as f64).collect(),
        _ => axis.settings(),
    };
    let settings: Vec<f64> = all_settings
        .into_iter()
        .filter(|&s| spec_at(s).is_some())
        .collect();
    let mut r = ComboReport {
        axis: axis.name(),
        settings: settings.len(),
        default_setting: axis.default_setting(),
        ..Default::default()
    };
    let mut win_sets: Vec<BTreeSet<Vec<u8>>> = vec![];
    let mut per_setting = vec![0usize; settings.len()];
    let mut total = 0usize;
    let mut runs = 0usize;
    for (si, &s) in settings.iter().enumerate() {
        let e = enumerate(&spec_at(s).unwrap());
        if e.no_gust_success {
            r.none_wins += 1;
        }
        per_setting[si] = e.successes;
        total += e.successes;
        runs += e.runs;
        if axis.in_dead_band(s) {
            r.dead_settings += 1;
            r.dead_wins += e.successes;
        }
        win_sets.push(e.solutions.iter().map(|a| encode(a)).collect());
    }
    r.default_wins = per_setting[0];
    r.settings_with_wins = per_setting.iter().filter(|n| **n > 0).count();
    r.win_fraction = total as f64 / runs.max(1) as f64;
    r.coupling = mean_jaccard(&win_sets);
    r.win_sets = win_sets.clone();
    r.setting_values = match &level.rules {
        Rules::Steer { dial: Some(v), .. } => v.clone(),
        Rules::Steer {
            companion: Some(c), ..
        } => c.delays.clone(),
        _ => settings.clone(),
    };
    r.peak_wins = per_setting.iter().copied().max().unwrap_or(0);
    // informed: scan with free natural (no-gust) runs and take the setting where the probe ends nearest the target
    let c = base.regions[base.goal].center;
    let nearest = |s: f64| {
        crate::validate_steer::natural_run(&spec_at(s).unwrap())
            .path()
            .iter()
            .map(|p| p.dist(c))
            .fold(f64::INFINITY, f64::min)
    };
    // With a drone the scan is the anchor reader: read the drone's arrival time at each release and the time at which a
    // gust-free run of the steered probe is nearest the goal, and take the release at which the two are closest (a
    // person compares the two times, not the distance the probe ends at).
    let anchor = |s: f64| {
        let sp = spec_at(s).unwrap();
        let Some(a) = sp.drone_arrival else {
            return f64::INFINITY;
        };
        let path = crate::validate_steer::natural_run(&sp).path();
        let k = path
            .iter()
            .enumerate()
            .min_by(|x, y| x.1.dist(c).total_cmp(&y.1.dist(c)))
            .map(|(k, _)| k)
            .unwrap_or(0);
        (a - k as f64 * sp.cfg.dt).abs()
    };
    let (ii, _) = settings
        .iter()
        .enumerate()
        .map(|(i, &s)| {
            (
                i,
                if base.companion.is_some() {
                    anchor(s)
                } else {
                    nearest(s)
                },
            )
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .unwrap();
    r.informed_setting = settings[ii];
    r.informed_admits_win = per_setting[ii] > 0;
    r.informed_plan_fraction = per_setting[ii] as f64 / runs.max(1) as f64 * settings.len() as f64;
    r.blind_setting_fraction = r.settings_with_wins as f64 / settings.len() as f64;
    r.informed_measured = true;
    r
}

// ------------------------------------------------------------------ Synchronize levels
/// Delay grid of the Synchronize combination validation: coarser than the interface's 0.1 s, because every source
/// repeats the whole search.
pub const COMBO_DELAY_STEP: f64 = 0.2;
/// A pair of release delays is "near the defaults" when both are within this many seconds of 0.
pub const DELAY_DEAD_S: f64 = 1.5;

/// Two probes, a placed source: the control is the source (setting 0 is no source), the plan is the pair of release
/// delays. Probes do not interact, so each is flown once per delay and every pair is judged on the recorded entry
/// times, which is exact.
pub fn sync_report(level: &Level, step: f64) -> ComboReport {
    let base = SyncSpec::from_level(level);
    assert_eq!(
        base.probes(),
        2,
        "the combination validation handles two probes"
    );
    // The control: a source placed first (setting 0 is no source) or the start slot of one probe (setting 0 is its
    // default slot). Either way a setting is one `SyncSpec`, and the plan is the pair of delays.
    let specs: Vec<SyncSpec> = if base.slots.is_some() {
        (0..base.slots.as_ref().map_or(0, |(_, v)| v.len()))
            .map(|i| base.with_slot(i))
            .collect()
    } else if let Some(v) = &base.dial_levels {
        (0..v.len()).map(|i| base.with_dial(i)).collect()
    } else {
        let kinds = base
            .source_rules
            .as_ref()
            .expect("a Synchronize combination level offers sources, start slots or a dial")
            .kinds
            .clone();
        let sites = base.sites(step);
        let candidates: Vec<Source> = kinds
            .iter()
            .flat_map(|&k| sites.iter().map(move |&p| Source { kind: k, pos: p }))
            .collect();
        std::iter::once(base.clone())
            .chain(candidates.iter().map(|c| base.with_sources(&[*c])))
            .collect()
    };
    let nd = (base.max_delay / COMBO_DELAY_STEP).round() as usize + 1;
    let nreg = level.regions.len();
    let near = |i: usize| i as f64 * COMBO_DELAY_STEP <= DELAY_DEAD_S + 1e-9;
    let wins_of = |spec: &SyncSpec| -> BTreeSet<(u8, u8)> {
        // per probe, per delay: did it reach its goal, and when did it enter each region?
        let data: Vec<Vec<(bool, Vec<Option<f64>>)>> = (0..2)
            .map(|p| {
                (0..nd)
                    .map(|d| {
                        let t = spec.fly_one(p, d as f64 * COMBO_DELAY_STEP);
                        (
                            t.outcome == Outcome::Reached(spec.goals[p]),
                            (0..nreg).map(|r| entry_time(&spec.level, &t, r)).collect(),
                        )
                    })
                    .collect()
            })
            .collect();
        let mut set = BTreeSet::new();
        for a in 0..nd {
            if !data[0][a].0 {
                continue;
            }
            for b in 0..nd {
                if !data[1][b].0 {
                    continue;
                }
                let idx = [a, b];
                let ok = spec.conds.iter().all(|c| {
                    match (data[c.a][idx[c.a]].1[c.ra], data[c.b][idx[c.b]].1[c.rb]) {
                        (Some(ta), Some(tb)) => tb - ta >= c.lo - 1e-9 && tb - ta <= c.hi + 1e-9,
                        _ => false,
                    }
                });
                if ok {
                    set.insert((a as u8, b as u8));
                }
            }
        }
        set
    };
    let axis = axis_of(level).expect("a Synchronize combination level has one extra control");
    let mut r = ComboReport {
        axis: axis.name(),
        settings: specs.len(),
        default_setting: 0.0,
        dead_settings: 1,
        ..Default::default()
    };
    let mut sets: Vec<BTreeSet<(u8, u8)>> = vec![];
    let mut total = 0usize;
    for (s, spec) in specs.iter().enumerate() {
        let set = wins_of(spec);
        if set.contains(&(0, 0)) {
            r.none_wins += 1;
        }
        r.dead_wins += set
            .iter()
            .filter(|(a, b)| near(*a as usize) && near(*b as usize))
            .count();
        if let Some(m) = set
            .iter()
            .map(|(a, b)| (*a).max(*b) as f64 * COMBO_DELAY_STEP)
            .reduce(f64::min)
        {
            r.nearest_win_s = Some(r.nearest_win_s.map_or(m, |c| c.min(m)));
        }
        if s == 0 {
            r.default_wins = set.len();
        }
        if !set.is_empty() {
            r.settings_with_wins += 1;
        }
        r.peak_wins = r.peak_wins.max(set.len());
        total += set.len();
        sets.push(set);
    }
    r.win_fraction = total as f64 / (r.settings * nd * nd) as f64;
    r.coupling = mean_jaccard(&sets);
    r.win_sets = sets
        .iter()
        .map(|st| st.iter().map(|(a, b)| vec![*a, *b]).collect())
        .collect();
    r.setting_values = match &level.rules {
        Rules::Synchronize { dial: Some(v), .. } => v.clone(),
        _ => (0..specs.len()).map(|i| i as f64).collect(),
    };
    r.blind_setting_fraction = r.settings_with_wins as f64 / r.settings as f64;
    r
}

// ------------------------------------------------------------------ entry points
/// Phases scanned for a Tempo level with a dial: this many evenly spaced launch phases per period.
pub const TEMPO_PHASES: usize = 240;

/// A Tempo level with a dial: the control is the dial setting (setting 0 is the default) and the plan is the launch
/// phase. `none_wins` counts settings at which launching at phase 0 wins (the "plan" nobody chose), `default_wins`
/// the winning phases at the default setting, `dead_wins` the winning phases within 1.5 s of phase 0 at any setting.
pub fn tempo_report(level: &Level) -> ComboReport {
    let Rules::Tempo {
        start,
        period,
        goal,
        dial: Some(levels),
    } = &level.rules
    else {
        panic!("not a Tempo level with a dial")
    };
    let n = TEMPO_PHASES;
    let mut r = ComboReport {
        axis: "dial setting",
        settings: levels.len(),
        default_setting: 0.0,
        dead_settings: 1,
        ..Default::default()
    };
    let near = |i: usize| {
        let ph = *period * i as f64 / n as f64;
        ph.min(*period - ph) <= 1.5
    };
    let mut win_sets: Vec<BTreeSet<Vec<u8>>> = vec![];
    let mut per_setting = vec![0usize; levels.len()];
    for (si, &lv) in levels.iter().enumerate() {
        let f = level.field.with_intensity(lv);
        let wins: Vec<usize> = (0..n)
            .filter(|&i| {
                level.reached_goal(&level.run(&f, *start, *period * i as f64 / n as f64), *goal)
            })
            .collect();
        if wins.contains(&0) {
            r.none_wins += 1;
        }
        r.dead_wins += wins.iter().filter(|&&i| near(i)).count();
        per_setting[si] = wins.len();
        win_sets.push(
            wins.iter()
                .map(|&i| vec![(i % 256) as u8, (i / 256) as u8])
                .collect(),
        );
    }
    r.default_wins = per_setting[0];
    r.settings_with_wins = per_setting.iter().filter(|&&c| c > 0).count();
    r.win_fraction = per_setting.iter().sum::<usize>() as f64 / (n * levels.len()) as f64;
    r.coupling = mean_jaccard(&win_sets);
    r.win_sets = win_sets.clone();
    r.setting_values = levels.clone();
    r.peak_wins = per_setting.iter().copied().max().unwrap_or(0);
    r.blind_setting_fraction = r.settings_with_wins as f64 / levels.len() as f64;
    r.informed_measured = false;
    r
}

pub fn report_for(level: &Level, step: f64) -> ComboReport {
    match &level.rules {
        Rules::Shape { .. } => shape_report(level, step),
        Rules::Steer { .. } => steer_report(level),
        Rules::Synchronize { .. } => sync_report(level, step),
        Rules::Tempo { .. } => tempo_report(level),
        _ => panic!("not a two-decision combination level"),
    }
}

/// For each of the four moves of the target and obstacle by `MOVE`: does the ablation still hold?
pub fn robustness(level: &Level, step: f64) -> Vec<((f64, f64), bool)> {
    [(MOVE, 0.0), (-MOVE, 0.0), (0.0, MOVE), (0.0, -MOVE)]
        .iter()
        .map(|&(dx, dy)| {
            (
                (dx, dy),
                report_for(&shifted(level, dx, dy), step).ablation_holds(),
            )
        })
        .collect()
}

pub fn solvable(level: &Level) -> bool {
    report_for(level, SITE_STEP).settings_with_wins > 0
}

pub fn summarize(level: &Level) -> String {
    let r = report_for(level, SITE_STEP);
    format!(
        "{} settings admit a win out of {}; empty-plan wins {}, wins at default {}, wins within the dead band {}; win fraction {:.2}%, coupling {:.2}; informed setting {:.2} admits a win: {}",
        r.settings_with_wins, r.settings, r.none_wins, r.default_wins, r.dead_wins, 100.0 * r.win_fraction, r.coupling, r.informed_setting, r.informed_admits_win
    )
}

/// Markdown report for `cargo run --release --bin report -- --combo`.
pub fn report(levels: &[Level]) -> String {
    let mut s = String::from("# Combination validation report\n\nGenerated by `cargo run --release --bin report -- --combo > docs/COMBO_VALIDATION.md`. See `docs/07-combinations.md` for the bar each level is held to.\n\nEvery number comes from the real simulation. Source searches use a 0.3 lattice of legal sites; phase levels try 24 launch phases (every 0.5 s) and dial levels 25 settings.\n\n");
    s.push_str("| level | control | settings with a win | empty-plan wins | wins at default | wins in dead band | win % of (setting, plan) | coupling | informed setting admits a win | robust to moved geometry |\n|---|---|---|---|---|---|---|---|---|---|\n");
    for l in levels {
        let r = report_for(l, SITE_STEP);
        let rob = robustness(l, SITE_STEP);
        s.push_str(&format!(
            "| {} {} | {} | {}/{} | {} | {} | {} | {:.2} | {:.2} | {} | {}/4 |\n",
            l.id,
            l.title,
            r.axis,
            r.settings_with_wins,
            r.settings,
            r.none_wins,
            r.default_wins,
            r.dead_wins,
            100.0 * r.win_fraction,
            r.coupling,
            if !r.informed_measured {
                "not measured"
            } else if r.informed_admits_win {
                "yes"
            } else {
                "no"
            },
            rob.iter().filter(|(_, ok)| *ok).count()
        ));
    }
    s
}

/// The measures of the combination protocol that follow from a level's win sets (docs/07-combinations.md, "The
/// protocol"): how far the winning (setting, plan) pairs are from a product, how wide and where the winning band of
/// settings is, whether the two controls trade off along a diagonal, and how often a random setting admits a win.
#[derive(Clone, Debug, Default)]
pub struct ProtocolReport {
    /// |W| / (|settings with a win| x |plans that win anywhere|): 1 means W is a product of two independent choices.
    pub fill_ratio: f64,
    /// The longest run of adjacent winning settings, in the order of their values.
    pub longest_window: usize,
    /// No winning setting is the lowest or the highest setting.
    pub interior: bool,
    /// Spearman correlation of the winning (setting value, phase) pairs; only for Tempo levels, where both are ordinal.
    pub spearman: Option<f64>,
    /// Share of settings that admit a win (what a random setting achieves).
    pub blind_fraction: f64,
    /// How many times better an informed scan is than a random setting, when the informed scan was measured.
    pub informed_over_blind: Option<f64>,
    pub coupling: f64,
}

impl ProtocolReport {
    /// The names of the protocol measures that fail, with their thresholds.
    pub fn failures(&self) -> Vec<&'static str> {
        let mut f = vec![];
        if self.fill_ratio > 0.5 {
            f.push("fill ratio above 0.5 (winning pairs look like a product)");
        }
        if self.longest_window < 3 {
            f.push("winning window narrower than 3 adjacent settings");
        }
        if !self.interior {
            f.push("winning band touches an end of the dial");
        }
        if self.spearman.is_some_and(|s| s.abs() > 0.8) {
            f.push("winning pairs lie on a thin diagonal (|Spearman| above 0.8)");
        }
        if self.blind_fraction > 0.5 {
            f.push("a random setting admits a win more than half the time");
        }
        if self.informed_over_blind.is_some_and(|x| x < 4.0) {
            f.push("informed scan less than 4x a random setting");
        }
        if self.coupling < 0.5 {
            f.push("coupling below 0.5");
        }
        f
    }
}

fn ranks(v: &[f64]) -> Vec<f64> {
    let mut idx: Vec<usize> = (0..v.len()).collect();
    idx.sort_by(|&a, &b| v[a].partial_cmp(&v[b]).unwrap());
    let mut r = vec![0.0; v.len()];
    let mut i = 0;
    while i < idx.len() {
        let mut j = i;
        while j + 1 < idx.len() && v[idx[j + 1]] == v[idx[i]] {
            j += 1;
        }
        let avg = (i + j) as f64 / 2.0;
        for k in i..=j {
            r[idx[k]] = avg;
        }
        i = j + 1;
    }
    r
}

fn spearman(x: &[f64], y: &[f64]) -> Option<f64> {
    if x.len() < 3 {
        return None;
    }
    let (rx, ry) = (ranks(x), ranks(y));
    let n = x.len() as f64;
    let (mx, my) = (rx.iter().sum::<f64>() / n, ry.iter().sum::<f64>() / n);
    let (mut sxy, mut sxx, mut syy) = (0.0, 0.0, 0.0);
    for i in 0..x.len() {
        sxy += (rx[i] - mx) * (ry[i] - my);
        sxx += (rx[i] - mx).powi(2);
        syy += (ry[i] - my).powi(2);
    }
    if sxx == 0.0 || syy == 0.0 {
        return None;
    }
    Some(sxy / (sxx * syy).sqrt())
}

/// The protocol measures for a level that has a pre-run control and a plan (a dial, start slots or a launch phase with a
/// plan). Panics for levels whose report has no win sets (Shape levels).
pub fn protocol(level: &Level) -> ProtocolReport {
    let r = report_for(level, SITE_STEP);
    assert!(
        !r.win_sets.is_empty(),
        "this level's report has no win sets"
    );
    let n = r.win_sets.len();
    let values: Vec<f64> = if r.setting_values.len() == n {
        r.setting_values.clone()
    } else {
        (0..n).map(|i| i as f64).collect()
    };
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&a, &b| values[a].partial_cmp(&values[b]).unwrap());
    let wins: Vec<bool> = order.iter().map(|&i| !r.win_sets[i].is_empty()).collect();
    let (mut run, mut longest) = (0usize, 0usize);
    for &w in &wins {
        run = if w { run + 1 } else { 0 };
        longest = longest.max(run);
    }
    let total: usize = r.win_sets.iter().map(|s| s.len()).sum();
    let proj_a = r.win_sets.iter().filter(|s| !s.is_empty()).count();
    let mut plans: BTreeSet<&Vec<u8>> = BTreeSet::new();
    for s in &r.win_sets {
        plans.extend(s.iter());
    }
    let fill = if proj_a * plans.len() == 0 {
        0.0
    } else {
        total as f64 / (proj_a * plans.len()) as f64
    };
    let spearman_v = if matches!(level.rules, Rules::Tempo { .. }) {
        let (mut xs, mut ys) = (vec![], vec![]);
        for (si, s) in r.win_sets.iter().enumerate() {
            for p in s {
                xs.push(values[si]);
                ys.push(p[0] as f64 + 256.0 * p[1] as f64);
            }
        }
        spearman(&xs, &ys)
    } else {
        None
    };
    let blind = r.settings_with_wins as f64 / n as f64;
    ProtocolReport {
        fill_ratio: fill,
        longest_window: longest,
        interior: !wins.first().copied().unwrap_or(false) && !wins.last().copied().unwrap_or(false),
        spearman: spearman_v,
        blind_fraction: blind,
        informed_over_blind: if r.informed_measured && r.informed_admits_win && blind > 0.0 {
            Some(1.0 / blind)
        } else {
            None
        },
        coupling: r.coupling,
    }
}

#[cfg(test)]
mod protocol_tests {
    use super::*;

    #[test]
    fn ranks_average_ties_and_spearman_sees_monotone_and_flat_data() {
        assert_eq!(ranks(&[3.0, 1.0, 2.0, 2.0]), vec![3.0, 0.0, 1.5, 1.5]);
        let x = [1.0, 2.0, 3.0, 4.0, 5.0];
        assert!((spearman(&x, &[10.0, 20.0, 30.0, 40.0, 50.0]).unwrap() - 1.0).abs() < 1e-12);
        assert!((spearman(&x, &[50.0, 40.0, 30.0, 20.0, 10.0]).unwrap() + 1.0).abs() < 1e-12);
        assert!(
            spearman(&x, &[7.0; 5]).is_none(),
            "a constant has no rank correlation"
        );
        assert!(
            spearman(&x[..2], &x[..2]).is_none(),
            "two points say nothing"
        );
    }

    #[test]
    fn failures_name_each_measure_that_misses_its_threshold() {
        let good = ProtocolReport {
            fill_ratio: 0.3,
            longest_window: 4,
            interior: true,
            spearman: Some(0.2),
            blind_fraction: 0.2,
            informed_over_blind: Some(5.0),
            coupling: 0.8,
        };
        assert!(good.failures().is_empty());
        let bad = ProtocolReport {
            fill_ratio: 0.9,
            longest_window: 1,
            interior: false,
            spearman: Some(-0.95),
            blind_fraction: 0.7,
            informed_over_blind: Some(1.4),
            coupling: 0.1,
        };
        assert_eq!(bad.failures().len(), 7);
    }
}
