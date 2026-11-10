//! Automated evidence for Synchronize levels. Every number comes from the real simulation: each probe is flown once
//! for every delay on a grid (`DELAY_STEP`), and the joint condition is then evaluated on the recorded arrival times,
//! which is exact because the probes do not interact (see `docs/11-synchronize.md` for why that matters).
//!
//! What is measured, and why:
//! - *solvable*: some combination of delays satisfies the condition;
//! - *blind fraction*: the share of random delay combinations that win;
//! - *Tempo-only fraction*: the share that win when each probe is given a delay at which it can reach its goal on its
//!   own. This is what a player who solves each probe separately, as in Tempo, would get: it is the test of whether the
//!   relationship between the probes is a real constraint;
//! - *anchor fraction*: of the delays at which probe 0 can reach its goal, how many have a partner delay for the other
//!   probes that wins (if nearly all do, the first probe can be released anywhere and only the others need care);
//! - *shift-break*: of the winning combinations, how many stop winning when every probe is released 1 s (or 2 s) later.
//!   0% means success depends only on the *relative* delays (a static field always gives 0%): the problem has one degree
//!   of freedom per extra probe and is a subtraction. Above 0% the absolute launch phase matters too;
//! - *width*: how far one probe's delay can move (others fixed) before a winning combination stops winning.
use crate::level::Level;
use crate::modes::synchronize::{entry_time, SyncSpec, DELAY_STEP};
use crate::sim::Outcome;

/// What one probe does when released at one delay, as far as the conditions are concerned.
#[derive(Clone, Debug)]
struct One {
    reached: bool,
    /// Entry time at each region of the level.
    entries: Vec<Option<f64>>,
}

pub struct Table {
    pub spec: SyncSpec,
    /// `info[probe][k]` for delay `k * DELAY_STEP`.
    info: Vec<Vec<One>>,
    pub n_delays: usize,
}

impl Table {
    pub fn new(spec: &SyncSpec) -> Table {
        let n_delays = (spec.max_delay / DELAY_STEP).round() as usize + 1;
        let info = (0..spec.probes())
            .map(|i| {
                (0..n_delays)
                    .map(|k| {
                        let t = spec.fly_one(i, k as f64 * DELAY_STEP);
                        One { reached: t.outcome == Outcome::Reached(spec.goals[i]), entries: (0..spec.level.regions.len()).map(|r| entry_time(&spec.level, &t, r)).collect() }
                    })
                    .collect()
            })
            .collect();
        Table { spec: spec.clone(), info, n_delays }
    }
    /// Does the combination of delay indices win?
    pub fn wins(&self, ks: &[usize]) -> bool {
        if !ks.iter().enumerate().all(|(i, k)| self.info[i][*k].reached) {
            return false;
        }
        self.spec.conds.iter().all(|c| match (self.info[c.a][ks[c.a]].entries[c.ra], self.info[c.b][ks[c.b]].entries[c.rb]) {
            (Some(a), Some(b)) => (b - a) >= c.lo - 1e-9 && (b - a) <= c.hi + 1e-9,
            _ => false,
        })
    }
    pub fn reaches(&self, probe: usize, k: usize) -> bool {
        self.info[probe][k].reached
    }
    /// Every winning combination of delay indices (exhaustive; fine for up to three probes).
    pub fn winners(&self) -> Vec<Vec<usize>> {
        let n = self.spec.probes();
        let mut out = vec![];
        let mut ks = vec![0usize; n];
        loop {
            if self.wins(&ks) {
                out.push(ks.clone());
            }
            let mut i = 0;
            loop {
                ks[i] += 1;
                if ks[i] < self.n_delays {
                    break;
                }
                ks[i] = 0;
                i += 1;
                if i == n {
                    return out;
                }
            }
        }
    }
}

/// Numbers describing a level; see the module comment.
#[derive(Clone, Debug)]
pub struct Stats {
    pub combos: usize,
    pub winners: usize,
    /// Per probe: fraction of delays at which it reaches its goal alone.
    pub alone: Vec<f64>,
    /// Winners / (combinations where every probe can reach its goal alone).
    pub tempo_only: f64,
    /// Fraction of delays of probe 0 (among those where it can reach its goal) that have a winning partner.
    pub anchor: f64,
    /// Of the winners, the fraction that lose when everything is released `1 s` later (among those that stay in range).
    pub shift_break_1s: f64,
    pub shift_break_2s: f64,
    /// Median, over winners, of how many seconds probe 1's delay can vary (others fixed) with every step still winning.
    pub width: f64,
}

impl Stats {
    pub fn blind(&self) -> f64 {
        self.winners as f64 / self.combos.max(1) as f64
    }
}

pub fn stats(spec: &SyncSpec) -> Stats {
    let t = Table::new(spec);
    let n = spec.probes();
    let combos = t.n_delays.pow(n as u32);
    let win = t.winners();
    let alone: Vec<f64> = (0..n).map(|i| (0..t.n_delays).filter(|k| t.reaches(i, *k)).count() as f64 / t.n_delays as f64).collect();
    let feasible: usize = (0..n).map(|i| (0..t.n_delays).filter(|k| t.reaches(i, *k)).count()).product();
    let tempo_only = win.len() as f64 / feasible.max(1) as f64;
    // anchor: probe 0's feasible delays that appear in some winner
    let feas0: Vec<usize> = (0..t.n_delays).filter(|k| t.reaches(0, *k)).collect();
    let anchored = feas0.iter().filter(|k| win.iter().any(|w| w[0] == **k)).count();
    let anchor = anchored as f64 / feas0.len().max(1) as f64;
    let shift = |secs: f64| -> f64 {
        let by = (secs / DELAY_STEP).round() as usize;
        let (mut tot, mut brk) = (0usize, 0usize);
        for w in &win {
            if w.iter().all(|k| k + by < t.n_delays) {
                tot += 1;
                let moved: Vec<usize> = w.iter().map(|k| k + by).collect();
                brk += (!t.wins(&moved)) as usize;
            }
        }
        if tot == 0 { 0.0 } else { brk as f64 / tot as f64 }
    };
    let mut widths: Vec<f64> = win
        .iter()
        .filter(|_| n >= 2)
        .map(|w| {
            let mut run = 1usize;
            let (mut lo, mut hi) = (w.clone(), w.clone());
            while lo[1] > 0 {
                lo[1] -= 1;
                if t.wins(&lo) { run += 1 } else { break }
            }
            while hi[1] + 1 < t.n_delays {
                hi[1] += 1;
                if t.wins(&hi) { run += 1 } else { break }
            }
            run as f64 * DELAY_STEP
        })
        .collect();
    widths.sort_by(|a, b| a.total_cmp(b));
    let width = if widths.is_empty() { 0.0 } else { widths[widths.len() / 2] };
    Stats { combos, winners: win.len(), alone, tempo_only, anchor, shift_break_1s: shift(1.0), shift_break_2s: shift(2.0), width }
}

pub fn solvable(level: &Level) -> bool {
    let spec = SyncSpec::from_level(level);
    !Table::new(&spec).winners().is_empty()
}

pub fn summarize(level: &Level) -> String {
    let s = stats(&SyncSpec::from_level(level));
    format!("blind {:.2}%, Tempo-only {:.1}%, anchor {:.0}%, shift-break {:.0}%, width {:.1} s", 100.0 * s.blind(), 100.0 * s.tempo_only, 100.0 * s.anchor, 100.0 * s.shift_break_1s, s.width)
}

pub fn report(levels: &[Level]) -> String {
    let mut out = String::from("# Synchronize validation report\n\nGenerated by `cargo run --release --bin report -- --sync > docs/SYNC_VALIDATION.md`. See `docs/11-synchronize.md` for the method.\n\nEvery probe is flown through the real simulation once per delay (0.1 s grid) and the joint condition is evaluated on the recorded arrival times.\n\n");
    out.push_str("| level | probes | window(s) | each probe alone (delays that reach) | blind win % | Tempo-only win % | anchor % | shift-break 1 s / 2 s | width (s) |\n|---|---|---|---|---|---|---|---|---|\n");
    for l in levels {
        let spec = SyncSpec::from_level(l);
        let s = stats(&spec);
        let wins = spec.conds.iter().map(|c| format!("{}@{} -> {}@{} in [{:+.1}, {:+.1}]", c.a, c.ra, c.b, c.rb, c.lo, c.hi)).collect::<Vec<_>>().join("; ");
        out.push_str(&format!(
            "| {} {} | {} | {} | {} | {:.2} | {:.1} | {:.0} | {:.0}% / {:.0}% | {:.1} |\n",
            l.id,
            l.title,
            spec.probes(),
            wins,
            s.alone.iter().map(|a| format!("{:.0}%", 100.0 * a)).collect::<Vec<_>>().join(", "),
            100.0 * s.blind(),
            100.0 * s.tempo_only,
            100.0 * s.anchor,
            100.0 * s.shift_break_1s,
            100.0 * s.shift_break_2s,
            s.width,
        ));
    }
    out
}
