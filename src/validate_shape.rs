//! Automated evidence for Shape levels. Everything runs the real deterministic simulation through
//! `ShapeSpec::fly`; nothing here uses geometric shortcuts to decide whether a configuration wins.
//!
//! What is measured, and why (see `docs/10-shape.md`):
//! - *solvable*: some legal configuration (at most the level's source limit) reaches the target;
//! - *blind fraction*: the share of random legal configurations that win. It is anchored on the existing levels:
//!   Position levels win 6-20% of blind releases (the first one 100%) and Tempo levels 9-32% of launch phases;
//! - *source identity*: the win fraction of each kind alone, so a level that teaches a kind can be shown to need it;
//! - *composition*: for two sources, whether the second source's winning positions depend on where the first is
//!   (`coupling`), and that no single source wins;
//! - *reader advantage*: a player who reads the field (`reader_sites`) against a blind one;
//! - *robustness*: the level still works when the obstacle or target is moved, and winning placements are regions,
//!   not lattice points (`thickness`).
use crate::field::{SourceKind, V2};
use crate::level::Level;
use crate::modes::shape::{ShapeSpec, Source, MIN_SEPARATION};
use crate::sim::{RegionKind, Trajectory};

/// Lattice spacing for single-source searches, and for pair searches (a pair search is quadratic).
pub const STEP_SINGLE: f64 = 0.25;
pub const STEP_PAIR: f64 = 0.5;

fn wins(spec: &ShapeSpec, srcs: &[Source]) -> bool {
    spec.reached(&spec.fly(srcs))
}

/// One source of `kind` at every legal lattice site: `(site, wins)`.
pub fn single_map(spec: &ShapeSpec, kind: SourceKind, step: f64) -> Vec<(V2, bool)> {
    spec.sites(step).into_iter().map(|p| (p, wins(spec, &[Source { kind, pos: p }]))).collect()
}

/// For each offered kind: `(kind, legal sites, winning sites)` with one source.
pub fn kind_stats(spec: &ShapeSpec, step: f64) -> Vec<(SourceKind, usize, usize)> {
    spec.kinds
        .iter()
        .map(|&k| {
            let m = single_map(spec, k, step);
            (k, m.len(), m.iter().filter(|(_, w)| *w).count())
        })
        .collect()
}

/// Fraction of all (kind, site) single placements that win.
pub fn blind_single_fraction(spec: &ShapeSpec, step: f64) -> f64 {
    let s = kind_stats(spec, step);
    let (n, w) = s.iter().fold((0usize, 0usize), |a, (_, n, w)| (a.0 + n, a.1 + w));
    w as f64 / n.max(1) as f64
}

/// The flight with no sources: where the probe goes if the player does nothing.
pub fn baseline(spec: &ShapeSpec) -> Trajectory {
    spec.fly(&[])
}

// ------------------------------------------------------------------ the reader model
/// Smallest push a reader counts as "I can see it change the arrows there".
pub const READER_MIN_PUSH: f64 = 0.25;

/// The sites (with a kind) a player who *reads the field* would consider, given the path the probe flies now.
///
/// The model is deliberately simple and kind-agnostic; it is an assumption about reasoning, not a solver. A
/// reader knows (from a free launch) where the probe goes, and knows from the arrows what a source does around
/// itself. They only consider a source that can touch the path (its nearest path point is within the source's
/// influence radius and the push there is visible), and they want that push to carry the probe toward the side of
/// the path the target is on. If the target is almost straight ahead on the path there is no side to want, and the
/// only reason to push sideways is to clear whatever stopped the probe: they then accept a push that is mostly
/// sideways, but only from a source that touches the path near where the probe was stopped (within the source's
/// influence radius of the end of the path, when the path ended on an obstacle). The first kick, not the nearest
/// approach, is what is read, because the first half of a pass is what a swirl or a lane shift is decided by.
pub fn reader_sites(spec: &ShapeSpec, path: &Trajectory, step: f64) -> Vec<(SourceKind, V2)> {
    let reach = spec.influence_radius();
    let target = spec.level.regions[spec.goal].center;
    let pts: Vec<V2> = path.points.iter().step_by(5).copied().collect();
    if pts.len() < 2 {
        return vec![];
    }
    let stopped = matches!(path.outcome, crate::sim::Outcome::Hazard(_)).then(|| path.end());
    let mut out = vec![];
    for p in spec.sites(step) {
        // the reader looks at the push where the probe first meets the source (the first path point within the
        // source's influence radius where the push is visible): that first kick is what the arrows show them
        let Some(bi) = pts.iter().position(|s| s.dist(p) <= reach && spec.kinds.iter().any(|k| k.term(p, spec.k, spec.sigma).eval(*s, 0.0).len() >= READER_MIN_PUSH)) else { continue };
        let s = pts[bi];
        let h = if bi + 1 < pts.len() { pts[bi + 1] - s } else { s - pts[bi - 1] };
        if h.len() < 1e-9 {
            continue;
        }
        let h = h * (1.0 / h.len());
        let n = h.perp();
        let to_t = target - s;
        let c = if to_t.len() > 1e-9 { h.perp().dot(to_t) / to_t.len() } else { 0.0 };
        let side = if c.abs() < 0.15 { 0.0 } else { c.signum() };
        if side == 0.0 && stopped.is_some_and(|e| e.dist(s) > reach) {
            continue;
        }
        for &k in &spec.kinds {
            let u = k.term(p, spec.k, spec.sigma).eval(s, 0.0);
            if u.len() < READER_MIN_PUSH {
                continue;
            }
            let lat = u.dot(n);
            let ok = if side != 0.0 { lat * side >= 0.5 * u.len() } else { lat.abs() >= 0.5 * u.len() };
            if ok {
                out.push((k, p));
            }
        }
    }
    out
}

/// `(reader candidates, how many win)` for a single-source level.
pub fn reader_single(spec: &ShapeSpec, step: f64) -> (usize, usize) {
    let sites = reader_sites(spec, &baseline(spec), step);
    let w = sites.iter().filter(|(k, p)| wins(spec, &[Source { kind: *k, pos: *p }])).count();
    (sites.len(), w)
}

// ------------------------------------------------------------------ two sources
/// Every legal (kind, site) candidate on a lattice.
fn candidates(spec: &ShapeSpec, step: f64) -> Vec<Source> {
    let sites = spec.sites(step);
    spec.kinds.iter().flat_map(|&k| sites.iter().map(move |&p| Source { kind: k, pos: p })).collect()
}

/// A pair of candidates can coexist (the sources are not too close).
fn apart(a: &Source, b: &Source) -> bool {
    a.pos.dist(b.pos) >= MIN_SEPARATION
}

/// Result of an exhaustive search over pairs of sources on a lattice.
#[derive(Clone, Debug, Default)]
pub struct PairStats {
    /// Legal unordered pairs tried, and how many win.
    pub pairs: usize,
    pub wins: usize,
    /// Does any single source (of the same lattice) win? Composition levels need this to be false.
    pub single_wins: usize,
    /// For the winning pairs ordered by x (the first source along the flow is the *upstream* one): the mean
    /// Jaccard distance between the sets of downstream partners of different upstream sources. 0 means the second
    /// source's good positions do not depend on the first (two independent corrections); 1 means they never overlap.
    pub coupling: f64,
    /// Number of different upstream sources that appear in a winning pair.
    pub upstream_sources: usize,
    /// Pairs that win using two sources of the same kind, and using two different kinds.
    pub same_kind_wins: usize,
    pub mixed_kind_wins: usize,
}

pub fn pair_stats(spec: &ShapeSpec, step: f64) -> PairStats {
    let c = candidates(spec, step);
    let mut st = PairStats::default();
    st.single_wins = c.iter().filter(|s| wins(spec, &[**s])).count();
    // downstream partner sets per upstream candidate index
    let mut partners: Vec<Vec<usize>> = vec![vec![]; c.len()];
    for i in 0..c.len() {
        for j in (i + 1)..c.len() {
            if !apart(&c[i], &c[j]) {
                continue;
            }
            st.pairs += 1;
            if wins(spec, &[c[i], c[j]]) {
                st.wins += 1;
                if c[i].kind == c[j].kind {
                    st.same_kind_wins += 1;
                } else {
                    st.mixed_kind_wins += 1;
                }
                let (up, down) = if (c[i].pos.x, c[i].pos.y) <= (c[j].pos.x, c[j].pos.y) { (i, j) } else { (j, i) };
                partners[up].push(down);
            }
        }
    }
    let sets: Vec<&Vec<usize>> = partners.iter().filter(|p| !p.is_empty()).collect();
    st.upstream_sources = sets.len();
    // mean Jaccard distance over (a deterministic sample of) pairs of upstream sources
    let (mut sum, mut n) = (0.0, 0usize);
    let stride = (sets.len() / 60).max(1);
    for a in (0..sets.len()).step_by(stride) {
        for b in ((a + 1)..sets.len()).step_by(stride) {
            let (x, y) = (sets[a], sets[b]);
            let inter = x.iter().filter(|i| y.contains(i)).count();
            let union = x.len() + y.len() - inter;
            sum += 1.0 - inter as f64 / union.max(1) as f64;
            n += 1;
        }
    }
    st.coupling = if n == 0 { 0.0 } else { sum / n as f64 };
    st
}

/// A reader choosing two sources one after the other, looking at the path after the first: `(pairs, wins)` over
/// the first source drawn from the reader's sites for the baseline (a deterministic sample of at most `max_first`)
/// and the second from the reader's sites for the path that results.
pub fn reader_pairs(spec: &ShapeSpec, step: f64, max_first: usize) -> (usize, usize) {
    let first = reader_sites(spec, &baseline(spec), step);
    let stride = (first.len() / max_first.max(1)).max(1);
    let (mut n, mut w) = (0usize, 0usize);
    for (k, p) in first.iter().step_by(stride) {
        let a = Source { kind: *k, pos: *p };
        let path = spec.fly(&[a]);
        for (k2, p2) in reader_sites(spec, &path, step) {
            let b = Source { kind: k2, pos: p2 };
            if !apart(&a, &b) {
                continue;
            }
            n += 1;
            w += wins(spec, &[a, b]) as usize;
        }
    }
    (n, w)
}

// ------------------------------------------------------------------ robustness
/// The level with its target and obstacles moved by `(dx, dy)` each (the same shift for every region).
pub fn shifted(level: &Level, dx: f64, dy: f64) -> Level {
    let mut l = level.clone();
    for r in &mut l.regions {
        if matches!(r.kind, RegionKind::Target | RegionKind::Hazard) {
            r.center = r.center + V2 { x: dx, y: dy };
        }
    }
    l
}

/// For single-source levels: of the winning lattice placements, the fraction whose four neighbours half a step
/// away also win (a *region*, not a lattice point). Sampled deterministically (at most `max` winners).
pub fn thickness_single(spec: &ShapeSpec, step: f64, max: usize) -> f64 {
    let mut win_sites = vec![];
    for &k in &spec.kinds {
        for (p, w) in single_map(spec, k, step) {
            if w {
                win_sites.push((k, p));
            }
        }
    }
    let stride = (win_sites.len() / max.max(1)).max(1);
    let (mut thick, mut n) = (0usize, 0usize);
    for (k, p) in win_sites.iter().step_by(stride) {
        n += 1;
        let h = step * 0.5;
        let all = [V2 { x: h, y: 0.0 }, V2 { x: -h, y: 0.0 }, V2 { x: 0.0, y: h }, V2 { x: 0.0, y: -h }].iter().all(|d| {
            let q = *p + *d;
            spec.site_error(q).is_some() || wins(spec, &[Source { kind: *k, pos: q }])
        });
        thick += all as usize;
    }
    if n == 0 { 0.0 } else { thick as f64 / n as f64 }
}

// ------------------------------------------------------------------ level-level entry points
pub fn solvable(level: &Level) -> bool {
    let spec = ShapeSpec::from_level(level);
    if kind_stats(&spec, STEP_SINGLE).iter().any(|(_, _, w)| *w > 0) {
        return true;
    }
    spec.max_sources >= 2 && pair_stats(&spec, STEP_PAIR).wins > 0
}

pub fn summarize(level: &Level) -> String {
    let spec = ShapeSpec::from_level(level);
    let kinds = kind_stats(&spec, STEP_SINGLE)
        .iter()
        .map(|(k, n, w)| format!("{} {:.1}%", k.short(), 100.0 * *w as f64 / (*n).max(1) as f64))
        .collect::<Vec<_>>()
        .join(", ");
    if spec.max_sources >= 2 {
        let p = pair_stats(&spec, STEP_PAIR);
        format!("single: {kinds}; pairs {:.2}% of {} win, coupling {:.2}", 100.0 * p.wins as f64 / p.pairs.max(1) as f64, p.pairs, p.coupling)
    } else {
        let (rn, rw) = reader_single(&spec, STEP_SINGLE);
        format!("single: {kinds}; reader {:.0}% of {} sites", 100.0 * rw as f64 / rn.max(1) as f64, rn)
    }
}

/// Markdown report for `cargo run --release --bin report -- --shape`.
pub fn report(levels: &[Level]) -> String {
    let mut s = String::from("# Shape validation report\n\nGenerated by `cargo run --release --bin report -- --shape > docs/SHAPE_VALIDATION.md`. See `docs/10-shape.md` for the method.\n\nEvery number comes from the real simulation (`ShapeSpec::fly`). *Single* searches use a 0.25 lattice of legal placements, *pair* searches a 0.5 lattice.\n\n");
    s.push_str("| level | kinds offered | sources | baseline | blind win % (single) | win % by kind (single) | reader: sites / win % | thickness |\n|---|---|---|---|---|---|---|---|\n");
    for l in levels {
        let spec = ShapeSpec::from_level(l);
        let ks = kind_stats(&spec, STEP_SINGLE);
        let (n, w) = ks.iter().fold((0, 0), |a, (_, n, w)| (a.0 + n, a.1 + w));
        let by_kind = ks.iter().map(|(k, n, w)| format!("{} {:.1}", k.short(), 100.0 * *w as f64 / (*n).max(1) as f64)).collect::<Vec<_>>().join(", ");
        let (rn, rw) = reader_single(&spec, STEP_SINGLE);
        let b = baseline(&spec);
        s.push_str(&format!(
            "| {} {} | {} | {} | {} | {:.1} | {} | {} / {:.0} | {:.2} |\n",
            l.id,
            l.title,
            spec.kinds.iter().map(|k| k.short()).collect::<Vec<_>>().join(", "),
            spec.max_sources,
            l.describe(b.outcome),
            100.0 * w as f64 / n.max(1) as f64,
            by_kind,
            rn,
            100.0 * rw as f64 / rn.max(1) as f64,
            thickness_single(&spec, STEP_SINGLE, 200),
        ));
    }
    for l in levels {
        let spec = ShapeSpec::from_level(l);
        if spec.max_sources < 2 {
            continue;
        }
        let p = pair_stats(&spec, STEP_PAIR);
        let (rn, rw) = reader_pairs(&spec, STEP_PAIR, 40);
        s.push_str(&format!(
            "\n## {} {}: two sources\n\n- legal pairs {} ; winning {} ({:.2}%); single sources that win: {}\n- winning pairs by kinds: same kind {}, different kinds {}\n- upstream sources that appear in a winning pair: {}; coupling {:.2}\n- reader choosing the second source after seeing the first: {} pairs, {:.1}% win (blind {:.2}%)\n",
            l.id, l.title, p.pairs, p.wins, 100.0 * p.wins as f64 / p.pairs.max(1) as f64, p.single_wins, p.same_kind_wins, p.mixed_kind_wins, p.upstream_sources, p.coupling, rn, 100.0 * rw as f64 / rn.max(1) as f64, 100.0 * p.wins as f64 / p.pairs.max(1) as f64
        ));
    }
    s
}
