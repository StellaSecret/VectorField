//! Shape: place sources to change the field, then launch the probe. These tests cover the rules (placement,
//! movement, composition), that a launch flies the real combined field, each level's design claim against the real
//! simulation, and that no path is ever computed for the player before a launch.
use vfield::modes::*;
use vfield::sim::Outcome;
use vfield::validate_shape::*;
use vfield::*;

fn lv(id: &str) -> Level {
    levels::shape_levels().into_iter().find(|l| l.id == id).unwrap_or_else(|| panic!("no level {id}"))
}
fn spec(id: &str) -> ShapeSpec {
    ShapeSpec::from_level(&lv(id))
}
fn src(kind: SourceKind, x: f64, y: f64) -> ShapeSource {
    ShapeSource { kind, pos: v2(x, y) }
}
/// A deterministic stream of legal random placements (a different generator from the game's).
fn random_sites(spec: &ShapeSpec, n: usize, seed: u64) -> Vec<V2> {
    let mut g = Lcg(seed);
    let b = spec.zones[0].bbox();
    let mut out = vec![];
    while out.len() < n {
        let p = v2(b.min.x + b.width() * (g.below(10_000) as f64 / 9_999.0), b.min.y + b.height() * (g.below(10_000) as f64 / 9_999.0));
        if spec.site_error(p).is_none() {
            out.push(p);
        }
    }
    out
}

// ---------------------------------------------------------------- the chapter
#[test]
fn shape_is_a_chapter_with_four_validated_levels() {
    assert!(Chapter::ALL.contains(&Chapter::Shape));
    assert_eq!(Chapter::Shape.name(), "Shape");
    let ls = levels::chapter(Chapter::Shape);
    assert_eq!(ls.iter().map(|l| l.id).collect::<Vec<_>>(), ["shp-1", "shp-2", "shp-3", "shp-4"]);
    for l in &ls {
        assert!(matches!(l.rules, Rules::Shape { .. }));
        assert!(!l.hints.is_empty() && l.hints.len() >= 3, "{}", l.id);
        assert!(validate::solvable(l), "{} is not solvable", l.id);
        assert!(validate::oracle_win(l));
        // the player never controls where the probe starts: the same start, whatever is placed
        let sp = ShapeSpec::from_level(l);
        assert_eq!(sp.fly(&[]).points[0], sp.start);
    }
}

#[test]
fn adding_shape_leaves_the_other_chapters_as_they_were() {
    for (c, n) in [(Chapter::Position, 5), (Chapter::Predict, 5), (Chapter::Tempo, 5), (Chapter::Experiment, 5), (Chapter::Steer, 5), (Chapter::Survey, 5)] {
        assert_eq!(levels::chapter(c).len(), n, "{c:?}");
    }
    assert!(levels::chapter(Chapter::Combo).len() >= 6);
    // the ids of every level are still unique across all chapters
    let all = levels::all();
    let mut ids: Vec<_> = all.iter().map(|l| l.id).collect();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), all.len());
}

// ---------------------------------------------------------------- what each kind does to the field
#[test]
fn each_kind_changes_the_field_the_way_its_name_says() {
    let sp = spec("shp-3");
    let c = v2(0.0, 1.2);
    let near = |a: f64, r: f64| c + v2(a.cos(), a.sin()) * r;
    for step in 0..8 {
        let p = near(step as f64 * std::f64::consts::TAU / 8.0, 0.8);
        let out = p - c;
        let base = sp.level.field.eval(p, 0.0);
        let added = |k: SourceKind| sp.field_with(&[ShapeSource { kind: k, pos: c }]).eval(p, 0.0) - base;
        assert!(added(SourceKind::Repeller).dot(out) > 0.0, "a repeller pushes away");
        assert!(added(SourceKind::Attractor).dot(out) < 0.0, "an attractor pulls in");
        let (ccw, cw) = (added(SourceKind::SwirlCcw), added(SourceKind::SwirlCw));
        assert!(ccw.dot(out).abs() < 1e-9 && cw.dot(out).abs() < 1e-9, "a swirl is at right angles to the radius");
        assert!(out.x * ccw.y - out.y * ccw.x > 0.0, "counter-clockwise");
        assert!(out.x * cw.y - out.y * cw.x < 0.0, "clockwise");
    }
    // far from a source it does nothing visible; a source adds exactly its Gauss term to the background
    let far = v2(5.0, -3.0);
    let k = SourceKind::Repeller;
    assert!((sp.field_with(&[ShapeSource { kind: k, pos: c }]).eval(far, 0.0) - sp.level.field.eval(far, 0.0)).len() < 1e-6);
    let p = v2(0.5, 0.7);
    let want = sp.level.field.eval(p, 0.0) + k.term(c, sp.k, sp.sigma).eval(p, 0.0);
    assert_eq!(sp.field_with(&[ShapeSource { kind: k, pos: c }]).eval(p, 0.0), want);
}

#[test]
fn several_sources_add_and_their_order_does_not_matter() {
    let sp = spec("shp-4");
    let (a, b) = (src(SourceKind::Attractor, -1.0, 1.0), src(SourceKind::Repeller, 1.0, 2.0));
    for p in [v2(-0.5, 0.8), v2(0.0, 1.5), v2(1.2, 2.2), v2(3.0, 3.0)] {
        let sum = sp.level.field.eval(p, 0.0) + a.kind.term(a.pos, sp.k, sp.sigma).eval(p, 0.0) + b.kind.term(b.pos, sp.k, sp.sigma).eval(p, 0.0);
        let ab = sp.field_with(&[a, b]).eval(p, 0.0);
        let ba = sp.field_with(&[b, a]).eval(p, 0.0);
        assert!((ab - sum).len() < 1e-12 && (ba - sum).len() < 1e-12);
    }
    // and the path they make is neither source's path alone
    let both = sp.fly(&[a, b]);
    assert!(both.points != sp.fly(&[a]).points && both.points != sp.fly(&[b]).points);
}

// ---------------------------------------------------------------- placement, movement, removal
#[test]
fn placements_are_legal_only_inside_the_area_and_off_the_regions() {
    let mut g = ShapeGame::new(lv("shp-1"));
    // outside the placement area (the start is at x = -5, the area begins at -4)
    assert_eq!(g.place(v2(-5.0, 0.0)), Err(ShapePlaceError::OutsideZone));
    assert_eq!(g.place(v2(4.0, 0.0)), Err(ShapePlaceError::OutsideZone));
    // on the obstacle (-0.5, 0 radius 0.6)
    assert_eq!(g.place(v2(-0.5, 0.3)), Err(ShapePlaceError::OnRegion('X')));
    // a refused placement costs nothing and records nothing
    assert!(g.sources.is_empty() && g.attempts.is_empty() && g.remaining() == 1);
    assert_eq!(g.place(v2(-2.0, -0.8)), Ok(0));
    assert_eq!(g.remaining(), 0);
    // no sources left
    assert_eq!(g.place(v2(-3.0, -1.0)), Err(ShapePlaceError::Full));
    assert_eq!(g.sources.len(), 1);
    // a target cannot hold a source either (shp-2's area ends before the target, so test the rule on the spec)
    let sp = spec("shp-3");
    let t = lv("shp-3").regions[0].center;
    let mut wide = sp.clone();
    wide.zones = vec![Zone::Rect(sim::Rect::new(-6.0, -4.0, 6.0, 4.0))];
    assert_eq!(wide.site_error(t), Some(ShapePlaceError::OnRegion('T')));
    assert_eq!(wide.site_error(t + v2(0.0, 0.6)), None);
}

#[test]
fn sources_keep_their_distance_and_only_the_offered_kinds_can_be_selected() {
    let mut g = ShapeGame::new(lv("shp-4"));
    assert_eq!(g.place(v2(-1.0, 1.0)), Ok(0));
    assert_eq!(g.place(v2(-1.0, 1.0 + MIN_SEP * 0.9)), Err(ShapePlaceError::TooClose));
    assert_eq!(g.place(v2(-1.0, 1.0 + MIN_SEP)), Ok(1));
    // shp-4 offers a repeller and an attractor only
    assert_eq!(g.select(2), Err(ShapePlaceError::KindUnavailable));
    assert!(g.select(1).is_ok() && g.kind() == SourceKind::Attractor);
    g.cycle_kind();
    assert_eq!(g.kind(), SourceKind::Repeller);
    // a source cannot be placed (or moved) onto the obstacle
    let mut g = ShapeGame::new(lv("shp-4"));
    assert!(g.place(v2(-0.5, 0.3)).is_err());
}
const MIN_SEP: f64 = SHAPE_MIN_SEPARATION;

#[test]
fn a_source_can_be_dragged_replaced_and_removed() {
    let mut g = ShapeGame::new(lv("shp-4"));
    g.place(v2(-2.0, 1.0)).unwrap();
    g.select(1).unwrap();
    g.place(v2(1.0, 2.0)).unwrap();
    assert_eq!((g.sources[0].kind, g.sources[1].kind), (SourceKind::Repeller, SourceKind::Attractor));
    // picking by position
    assert_eq!(g.source_at(v2(-2.1, 1.05), 0.35), Some(0));
    assert_eq!(g.source_at(v2(0.0, 0.0), 0.35), None);
    // a legal move keeps the kind and the count; a drag in small steps never collides with the source itself
    for i in 1..=10 {
        assert!(g.move_source(0, v2(-2.0 + 0.05 * i as f64, 1.0)).is_ok());
    }
    assert_eq!(g.sources[0].kind, SourceKind::Repeller);
    // illegal moves leave it where it was
    let before = g.sources[0].pos;
    assert_eq!(g.move_source(0, v2(-9.0, 0.0)), Err(ShapePlaceError::OutsideZone));
    assert_eq!(g.move_source(0, v2(-0.5, 0.4)), Err(ShapePlaceError::OnRegion('X')));
    assert_eq!(g.move_source(0, g.sources[1].pos), Err(ShapePlaceError::TooClose));
    assert_eq!(g.sources[0].pos, before);
    // removal frees a slot
    assert_eq!(g.remaining(), 0);
    assert!(g.remove(0) && g.sources.len() == 1 && g.remaining() == 1);
    assert!(!g.remove(5));
    g.clear();
    assert!(g.sources.is_empty());
}

// ---------------------------------------------------------------- flying the combined field
#[test]
fn a_launch_flies_the_combined_field_and_records_it() {
    let mut g = ShapeGame::new(lv("shp-1"));
    // nothing placed: the background alone carries the probe into the obstacle (the level's premise)
    let a = g.launch().clone();
    assert!(!a.success && matches!(a.traj.outcome, Outcome::Hazard(_)) && a.sources.is_empty());
    assert_eq!(a.traj.points, g.level().run(&g.level().field, g.spec.start, 0.0).points, "no sources = the plain background field");
    // a placed source changes the path, and the attempt remembers the sources it was flown with
    g.place(v2(-2.0, -0.8)).unwrap();
    let b = g.launch().clone();
    assert_eq!(b.sources, g.sources);
    assert_ne!(a.traj.points, b.traj.points);
    assert_eq!(b.traj.points, g.level().run(&g.field(), g.spec.start, 0.0).points, "the launch flies exactly the field the arrows show");
    // it is deterministic: the same sources give bit-identical flights
    let c = g.launch().clone();
    assert_eq!(b.traj.points, c.traj.points);
    assert_eq!(g.attempts.len(), 3);
}

#[test]
fn a_winning_launch_reaches_the_target_and_the_win_is_kept() {
    let sp = spec("shp-1");
    let (site, _) = single_map(&sp, SourceKind::Repeller, 0.25).into_iter().find(|(_, w)| *w).expect("a winning site");
    let mut g = ShapeGame::new(lv("shp-1"));
    g.place(site).unwrap();
    let a = g.launch().clone();
    assert!(a.success && g.won);
    assert_eq!(a.traj.outcome, Outcome::Reached(0));
    // the player can keep experimenting after a win; it is not taken back
    g.clear();
    assert!(!g.launch().success);
    assert!(g.won);
    // obstacles end the run: the path of a failed attempt stops at the obstacle
    let failed = g.attempts.last().unwrap();
    let h = &g.level().regions[1];
    assert!(matches!(failed.traj.outcome, Outcome::Hazard(1)) && failed.traj.end().dist(h.center) <= h.radius + 0.05);
}

// ---------------------------------------------------------------- no preview
#[test]
fn editing_the_field_never_flies_a_probe() {
    let mut g = ShapeGame::new(lv("shp-3"));
    g.place(v2(0.0, -0.8)).unwrap();
    g.select(2).unwrap();
    let _ = g.move_source(0, v2(0.2, -0.9));
    g.remove(0);
    g.clear();
    let _ = g.field();
    assert!(g.attempts.is_empty() && !g.won, "only a launch produces a trajectory");
}

#[test]
fn the_front_end_computes_a_path_only_when_the_launch_button_is_used() {
    // A structural guard for the rule "no trajectory preview": the Shape frame in the UI may call `launch` (once,
    // behind the button) and nothing else that flies a probe.
    let app = include_str!("../src/app.rs");
    let start = app.find("fn shape_frame").expect("shape_frame");
    let end = app[start..].find("#[cfg(test)]").map(|i| start + i).unwrap_or(app.len());
    let body = &app[start..end];
    for forbidden in [".fly(", ".fly_charted(", "simulate(", ".run(&", "level.run", "sim::simulate"] {
        assert!(!body.contains(forbidden), "shape_frame calls {forbidden}: that would preview a path");
    }
    assert_eq!(body.matches(".launch()").count(), 1, "exactly one launch call, behind the button");
}

// ---------------------------------------------------------------- the levels' design claims
#[test]
fn level_one_a_repeller_below_and_upstream_of_the_obstacle_wins() {
    let sp = spec("shp-1");
    assert_eq!(sp.kinds, vec![SourceKind::Repeller]);
    let map = single_map(&sp, SourceKind::Repeller, 0.25);
    let wins: Vec<V2> = map.iter().filter(|(_, w)| *w).map(|(p, _)| *p).collect();
    assert!(!wins.is_empty());
    // every winner is below the stream (it pushes the probe up) and upstream of the obstacle
    assert!(wins.iter().all(|p| p.y < 0.0 && p.x < -0.4), "a winner is on the wrong side or too late");
    // the same distance on the other side of the path does the opposite
    let above = wins.iter().all(|p| !map.iter().any(|(q, w)| *w && q.dist(v2(p.x, -p.y)) < 1e-9));
    assert!(above, "a placement mirrored about the path must not also win");
    // the lane effect: where along the stream the source sits barely matters, how far off it sits does
    let by_x: std::collections::BTreeSet<i64> = wins.iter().map(|p| (p.x * 4.0).round() as i64).collect();
    let by_y: std::collections::BTreeSet<i64> = wins.iter().map(|p| (p.y * 4.0).round() as i64).collect();
    assert!(by_x.len() > by_y.len(), "winners span more of the stream than of the lateral offset");
}

#[test]
fn level_two_needs_the_attractor_and_a_pull_can_carry_the_probe_into_the_obstacle() {
    let sp = spec("shp-2");
    let stats = kind_stats(&sp, 0.125);
    let (_, _, rep) = stats.iter().find(|(k, _, _)| *k == SourceKind::Repeller).copied().unwrap();
    let (_, n, att) = stats.iter().find(|(k, _, _)| *k == SourceKind::Attractor).copied().unwrap();
    assert_eq!(rep, 0, "no repeller in the permitted area can reach the target");
    assert!(att as f64 / n as f64 > 0.08, "enough attractor placements win to be found by reasoning");
    // the drift misses the target without help
    assert_eq!(baseline(&sp).outcome, Outcome::Boundary);
    // the attractor really can pull the probe into the obstacle: some placements end there
    let hit = single_map(&sp, SourceKind::Attractor, 0.25).iter().filter(|(p, _)| matches!(sp.fly(&[ShapeSource { kind: SourceKind::Attractor, pos: *p }]).outcome, Outcome::Hazard(_))).count();
    assert!(hit >= 3, "{hit}");
    // and the placement area does not include the target's surroundings: no winning source sits on or beside it
    let t = sp.level.regions[0].center;
    let wins = single_map(&sp, SourceKind::Attractor, 0.25).into_iter().filter(|(_, w)| *w).map(|(p, _)| p);
    assert!(wins.into_iter().all(|p| p.dist(t) > 1.0));
}

#[test]
fn level_three_needs_the_swirl_and_its_direction_decides_the_side() {
    let sp = spec("shp-3");
    let stats = kind_stats(&sp, 0.125);
    let frac = |k: SourceKind| {
        let (_, n, w) = stats.iter().find(|(kk, _, _)| *kk == k).copied().unwrap();
        w as f64 / n as f64
    };
    assert_eq!((frac(SourceKind::Repeller), frac(SourceKind::Attractor)), (0.0, 0.0), "no radial source wins");
    assert!(frac(SourceKind::SwirlCcw) > 0.08 && frac(SourceKind::SwirlCw) > 0.08);
    // radial sources do clear the obstacle, but leave the probe off the line of the target
    let mut cleared = 0;
    for k in [SourceKind::Repeller, SourceKind::Attractor] {
        for (p, _) in single_map(&sp, k, 0.25) {
            let t = sp.fly(&[ShapeSource { kind: k, pos: p }]);
            if !matches!(t.outcome, Outcome::Hazard(_)) {
                cleared += 1;
                assert!(t.end().y.abs() > 0.4 || t.outcome != Outcome::Boundary || t.end().x > 5.9, "{k:?} at {p:?} ended on the line");
            }
        }
    }
    assert!(cleared > 100, "radial sources do get round the obstacle: {cleared}");
    // the direction a swirl turns decides which way the probe is bumped: counter-clockwise takes it under the
    // obstacle, clockwise over it (checked on the path at the obstacle's x, for every winner)
    let o = sp.level.regions[1].center;
    let passes = |k: SourceKind| -> Vec<f64> {
        single_map(&sp, k, 0.25).into_iter().filter(|(_, w)| *w).map(|(p, _)| sp.fly(&[ShapeSource { kind: k, pos: p }]).points.iter().min_by(|a, b| (a.x - o.x).abs().total_cmp(&(b.x - o.x).abs())).unwrap().y).collect()
    };
    let (ccw, cw) = (passes(SourceKind::SwirlCcw), passes(SourceKind::SwirlCw));
    assert!(!ccw.is_empty() && !cw.is_empty());
    assert!(ccw.iter().all(|y| *y < -0.4), "a counter-clockwise swirl bumps the probe below the obstacle");
    assert!(cw.iter().all(|y| *y > 0.4), "a clockwise swirl bumps it above");
    // and the swirl has to be near the obstacle
    assert!(single_map(&sp, SourceKind::SwirlCcw, 0.25).into_iter().filter(|(_, w)| *w).all(|(p, _)| (p.x - o.x).abs() < 1.3));
}

#[test]
fn level_four_no_single_source_wins_and_the_second_depends_on_the_first() {
    let sp = spec("shp-4");
    assert_eq!(sp.max_sources, 2);
    // nothing wins with one source, on a fine lattice, of either kind
    assert!(kind_stats(&sp, 0.125).iter().all(|(_, _, w)| *w == 0));
    let p = pair_stats(&sp, STEP_PAIR);
    assert_eq!(p.single_wins, 0);
    let frac = p.wins as f64 / p.pairs as f64;
    assert!((0.01..=0.10).contains(&frac), "pair win fraction {frac}");
    // the second source's good positions depend on where the first one is
    assert!(p.coupling >= 0.4, "coupling {}", p.coupling);
    assert!(p.upstream_sources >= 8, "{} different first sources appear in winning pairs", p.upstream_sources);
    // removing either source of a winning pair loses: the effect is the sum, not either part
    let c: Vec<ShapeSource> = {
        let sites = sp.sites(STEP_PAIR);
        let mut out = vec![];
        'o: for (i, a) in sites.iter().enumerate() {
            for b in &sites[i + 1..] {
                for (ka, kb) in [(SourceKind::Attractor, SourceKind::Attractor), (SourceKind::Attractor, SourceKind::Repeller)] {
                    let pair = [ShapeSource { kind: ka, pos: *a }, ShapeSource { kind: kb, pos: *b }];
                    if a.dist(*b) >= MIN_SEP && sp.reached(&sp.fly(&pair)) {
                        out = pair.to_vec();
                        break 'o;
                    }
                }
            }
        }
        out
    };
    assert_eq!(c.len(), 2, "a winning pair exists");
    assert!(!sp.reached(&sp.fly(&c[..1])) && !sp.reached(&sp.fly(&c[1..])));
}

// ---------------------------------------------------------------- blind placement and the reader
#[test]
fn blind_success_is_in_the_range_of_the_existing_levels() {
    // Position levels win 6.5-20% of blind releases (the first 100%), Tempo 9-32% of phases (docs/10-shape.md, sec. 4)
    for id in ["shp-1", "shp-2", "shp-3"] {
        let sp = spec(id);
        let f = blind_single_fraction(&sp, STEP_SINGLE);
        assert!((0.04..=0.35).contains(&f), "{id}: blind fraction {f}");
        // random legal placements (not on a lattice) agree with the lattice figure
        let kinds = sp.kinds.clone();
        let mut g = Lcg(7);
        let n = 1500;
        let w = (0..n).filter(|_| {
            let p = random_sites(&sp, 1, g.next())[0];
            let k = kinds[g.below(kinds.len())];
            sp.reached(&sp.fly(&[ShapeSource { kind: k, pos: p }]))
        }).count();
        let rf = w as f64 / n as f64;
        assert!((rf - f).abs() < 0.04 + 0.4 * f, "{id}: random {rf} vs lattice {f}");
    }
}

#[test]
fn reading_the_field_beats_blind_placement() {
    // The reader model is an assumption (see `reader_sites`), but it uses only what the arrows and one free launch
    // show. Against blind placement it must be clearly better, in every level.
    for id in ["shp-1", "shp-2", "shp-3"] {
        let sp = spec(id);
        let blind = blind_single_fraction(&sp, STEP_SINGLE);
        let (n, w) = reader_single(&sp, STEP_SINGLE);
        let reader = w as f64 / n as f64;
        assert!(n >= 20 && w > 0, "{id}: the reader has something to try");
        assert!(reader >= 2.0 * blind, "{id}: reader {reader:.3} vs blind {blind:.3}");
    }
    // two sources: a reader who looks at the path after the first one
    let sp = spec("shp-4");
    let blind = pair_stats(&sp, STEP_PAIR);
    let (n, w) = reader_pairs(&sp, STEP_PAIR, 40);
    let b = blind.wins as f64 / blind.pairs as f64;
    assert!(n >= 50 && w as f64 / n as f64 >= 5.0 * b, "reader {w}/{n} vs blind {b}");
}

// ---------------------------------------------------------------- robustness
#[test]
fn levels_do_not_depend_on_one_exact_geometry() {
    // moving the obstacle and target together by up to 0.25 in each direction keeps every one-source level solvable
    // with a blind fraction that stays in the same range
    for id in ["shp-1", "shp-2", "shp-3"] {
        let l = lv(id);
        for (dx, dy) in [(0.25, 0.0), (-0.25, 0.0), (0.0, 0.25), (0.0, -0.25), (0.25, 0.25), (-0.25, -0.25)] {
            let m = shifted(&l, dx, dy);
            assert!(validate::solvable(&m), "{id} shifted by ({dx}, {dy}) is not solvable");
            let f = blind_single_fraction(&ShapeSpec::from_level(&m), STEP_SINGLE);
            assert!((0.02..=0.45).contains(&f), "{id} shifted by ({dx}, {dy}): blind fraction {f}");
        }
    }
    // the composition level survives a shift too (a coarser check: a pair search per shift)
    for (dx, dy) in [(0.25, 0.0), (0.0, -0.25)] {
        let m = shifted(&lv("shp-4"), dx, dy);
        assert!(pair_stats(&ShapeSpec::from_level(&m), STEP_PAIR).wins > 0, "shp-4 shifted by ({dx}, {dy})");
    }
}

#[test]
fn winning_placements_are_regions_not_lattice_points() {
    for (id, min) in [("shp-1", 0.6), ("shp-2", 0.35), ("shp-3", 0.5)] {
        let t = thickness_single(&spec(id), STEP_SINGLE, 300);
        assert!(t >= min, "{id}: thickness {t}");
    }
    // a placement half a lattice step off the lattice still wins where its lattice neighbours do (level 1)
    let sp = spec("shp-1");
    let (site, _) = single_map(&sp, SourceKind::Repeller, 0.25).into_iter().find(|(p, w)| *w && (p.y + 0.75).abs() < 1e-9 && (p.x + 2.0).abs() < 1e-9).expect("a central winner");
    for d in [v2(0.1, 0.0), v2(-0.1, 0.07), v2(0.0, -0.1), v2(0.05, 0.1)] {
        assert!(sp.reached(&sp.fly(&[ShapeSource { kind: SourceKind::Repeller, pos: site + d }])));
    }
}

// ---------------------------------------------------------------- Shape is a different kind of decision
#[test]
fn shape_changes_the_field_not_the_start_and_that_changes_what_wins() {
    // In Position the probe's release point is the decision and the field is fixed. Here the release point is fixed
    // and the field is the decision: with the start fixed, a different source gives a different path, and no single
    // number (an x or a y) tells the whole story: the same y wins or loses depending on what stands there.
    let sp = spec("shp-3");
    let (x, y) = (0.0, -0.8);
    let outcomes: std::collections::HashSet<String> =
        sp.kinds.iter().map(|&k| format!("{:?}", sp.fly(&[ShapeSource { kind: k, pos: v2(x, y) }]).outcome)).collect();
    assert!(outcomes.len() >= 2, "at one position different kinds give different results: {outcomes:?}");
    let wins_here: Vec<_> = sp.kinds.iter().filter(|&&k| sp.reached(&sp.fly(&[ShapeSource { kind: k, pos: v2(x, y) }]))).collect();
    assert_eq!(wins_here, [&SourceKind::SwirlCcw]);
}
