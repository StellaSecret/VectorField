//! Two-decision combination levels (cmb-7, cmb-8, cmb-11): each level must need *both* of its decisions. The
//! ablation numbers come from `validate_combo` (real simulation); the rest pin the rules of the added controls.
use vfield::modes::*;
use vfield::validate_combo::*;
use vfield::*;

const TWO: [&str; 7] = [
    "cmb-7", "cmb-8", "cmb-9", "cmb-10", "cmb-11", "cmb-12", "cmb-13",
];

/// Each report is a few seconds of simulation and many tests read the same ones, so compute them once per run.
fn rep(id: &str) -> ComboReport {
    static CACHE: std::sync::OnceLock<
        std::sync::Mutex<std::collections::HashMap<String, ComboReport>>,
    > = std::sync::OnceLock::new();
    let m = CACHE.get_or_init(Default::default);
    if let Some(r) = m.lock().unwrap().get(id) {
        return r.clone();
    }
    let r = report_for(&lv(id), SITE_STEP);
    m.lock().unwrap().insert(id.to_string(), r.clone());
    r
}
fn rob(id: &str) -> Vec<((f64, f64), bool)> {
    static CACHE: std::sync::OnceLock<
        std::sync::Mutex<std::collections::HashMap<String, Vec<((f64, f64), bool)>>>,
    > = std::sync::OnceLock::new();
    let m = CACHE.get_or_init(Default::default);
    if let Some(r) = m.lock().unwrap().get(id) {
        return r.clone();
    }
    let r = robustness(&lv(id), SITE_STEP);
    m.lock().unwrap().insert(id.to_string(), r.clone());
    r
}

fn lv(id: &str) -> Level {
    levels::combo_levels()
        .into_iter()
        .find(|l| l.id == id)
        .unwrap_or_else(|| panic!("no level {id}"))
}
fn src(kind: SourceKind, x: f64, y: f64) -> ShapeSource {
    ShapeSource {
        kind,
        pos: v2(x, y),
    }
}

// ---------------------------------------------------------------- the levels exist and are wired in
#[test]
fn the_new_levels_are_in_the_combination_chapter_after_the_old_six() {
    let ids: Vec<_> = levels::chapter(Chapter::Combo)
        .iter()
        .map(|l| l.id)
        .collect();
    assert_eq!(
        ids,
        [
            "cmb-1", "cmb-2", "cmb-3", "cmb-4", "cmb-5", "cmb-6", "cmb-7", "cmb-8", "cmb-9",
            "cmb-10", "cmb-11", "cmb-12", "cmb-13", "cmb-14", "cmb-15", "cmb-16", "cmb-17",
            "cmb-18", "cmb-19", "cmb-20", "cmb-21"
        ]
    );
    for id in TWO {
        let l = lv(id);
        assert!(axis_of(&l).is_some(), "{id} has exactly one added control");
        assert!(l.hints.len() >= 2, "{id}");
        assert!(vfield::validate::solvable(&l), "{id} is not solvable");
    }
}

#[test]
fn prerequisites_name_a_finished_shape_or_steer_level() {
    for id in ["cmb-7", "cmb-8"] {
        assert_eq!(levels::combo_requires(id), ["shp-1"]);
    }
    assert_eq!(
        levels::combo_requires("cmb-9"),
        ["shp-1", "syn-1"],
        "it needs both mechanics"
    );
    assert_eq!(
        levels::combo_requires("cmb-10"),
        ["pos-1", "syn-1"],
        "it needs both mechanics"
    );
    assert_eq!(levels::combo_requires("cmb-11"), ["ste-1"]);
    assert_eq!(levels::combo_requires("cmb-12"), ["ste-1"]);
    assert_eq!(
        levels::combo_requires("cmb-13"),
        ["shp-1", "ste-1"],
        "it needs both mechanics"
    );
    for id in ["cmb-1", "cmb-5", "cmb-6"] {
        assert!(levels::combo_requires(id).is_empty());
    }
    let all: Vec<_> = levels::all().iter().map(|l| l.id).collect();
    for id in TWO {
        for r in levels::combo_requires(id) {
            assert!(all.contains(r), "{id} requires unknown level {r}");
        }
    }
}

// ---------------------------------------------------------------- the bar: both decisions are necessary
#[test]
fn removing_either_decision_makes_every_level_unwinnable() {
    for id in TWO {
        let r = rep(id);
        assert_eq!(
            r.none_wins, 0,
            "{id}: some control setting wins with an empty plan, so the plan is not needed"
        );
        assert_eq!(
            r.default_wins, 0,
            "{id}: the default control setting wins, so the control is not needed"
        );
        assert_eq!(
            r.dead_wins, 0,
            "{id}: a win within the dead band of the default, so a small nudge is enough"
        );
        assert!(r.settings_with_wins > 0, "{id}: nothing wins at all");
        assert!(r.ablation_holds(), "{id}");
    }
}

#[test]
fn the_dead_band_is_wide_enough_to_mean_something() {
    for id in TWO {
        let r = rep(id);
        if let Some(Axis::Slots(n)) = axis_of(&lv(id)) {
            // a numbered slot has no distance from the default slot: the dead band is the default itself
            assert_eq!(r.dead_settings, 1, "{id}");
            assert_eq!(r.settings, n, "{id}");
        } else if axis_of(&lv(id)) == Some(Axis::Source) {
            // a source has no distance from "no source": the dead band is the default itself
            assert_eq!(r.dead_settings, 1, "{id}");
            assert!(r.settings > 100, "{id}: only {} sources tried", r.settings);
        } else {
            assert_eq!(r.settings, axis_of(&lv(id)).unwrap().settings().len());
            // a dead band of at least 1.5 s of phase (3 settings each way, plus the default) or 15% of the dial range
            assert!(
                r.dead_settings >= 7,
                "{id}: dead band only {} settings",
                r.dead_settings
            );
        }
    }
}

#[test]
fn the_control_must_be_found_a_random_setting_usually_loses() {
    for id in TWO {
        let r = rep(id);
        assert!(
            r.blind_setting_fraction < 0.5,
            "{id}: {:.0}% of settings admit a win, so the control hardly matters",
            100.0 * r.blind_setting_fraction
        );
        assert!(
            r.blind_setting_fraction > 0.1,
            "{id}: too few settings admit a win to be found by a player"
        );
    }
}

#[test]
fn the_two_decisions_depend_on_each_other() {
    // The plans that win at one setting are not the plans that win at another: neither decision can be made alone.
    for id in TWO {
        let r = rep(id);
        assert!(
            r.coupling > 0.4,
            "{id}: coupling {:.2} - the winning plans barely change with the control",
            r.coupling
        );
    }
}

#[test]
fn reading_the_field_beats_choosing_blind() {
    for id in TWO {
        let r = rep(id);
        if !r.informed_measured {
            continue; // not measured for Synchronize levels (the report says so)
        }
        assert!(
            r.informed_admits_win,
            "{id}: scanning with empty launches does not find a setting that admits a win"
        );
    }
    // For the Shape levels the reader then picks a source: it must beat a blind (setting, site) draw.
    for id in ["cmb-7", "cmb-8"] {
        let r = rep(id);
        assert!(
            r.informed_plan_fraction > 2.0 * r.win_fraction,
            "{id}: informed {:.1}% against blind {:.1}%",
            100.0 * r.informed_plan_fraction,
            100.0 * r.win_fraction
        );
    }
}

#[test]
fn the_ablation_survives_moving_the_target_and_obstacle() {
    for id in TWO {
        for ((dx, dy), ok) in rob(id) {
            assert!(ok, "{id}: the ablation fails when the target and obstacle move by ({dx:+.1}, {dy:+.1})");
        }
    }
}

// ---------------------------------------------------------------- Shape: the added controls
#[test]
fn shape_games_start_at_the_default_controls_and_clamp_them() {
    let g7 = ShapeGame::new(lv("cmb-7"));
    assert_eq!((g7.phase, g7.period()), (0.0, Some(12.0)));
    assert!(g7.intensity_range().is_none());
    let mut g7 = g7;
    g7.set_phase(13.5);
    assert!(
        (g7.phase - 1.5).abs() < 1e-12,
        "the phase wraps into the period"
    );
    g7.set_phase(-1.0);
    assert!((g7.phase - 11.0).abs() < 1e-12);
    g7.set_intensity(9.0);
    assert_eq!(
        g7.intensity,
        g7.spec.default_intensity(),
        "a level with no dial ignores it"
    );

    let mut g8 = ShapeGame::new(lv("cmb-8"));
    assert_eq!(
        g8.intensity, 1.5,
        "the dial starts in the middle of its range"
    );
    g8.set_intensity(9.0);
    assert_eq!(g8.intensity, 3.0);
    g8.set_intensity(-2.0);
    assert_eq!(g8.intensity, 0.0);
    g8.set_phase(5.0);
    assert_eq!(g8.phase, 0.0, "a level with no period ignores the phase");
}

#[test]
fn the_ordinary_shape_levels_are_unchanged_by_the_new_controls() {
    for l in levels::shape_levels() {
        let sp = ShapeSpec::from_level(&l);
        assert!(sp.period.is_none() && sp.intensity.is_none(), "{}", l.id);
        let s = [src(SourceKind::Repeller, 0.0, -1.0)];
        assert_eq!(
            sp.fly(&s).points.len(),
            sp.fly_at(&s, 0.0, l.field.intensity).points.len(),
            "{}",
            l.id
        );
        assert_eq!(
            sp.fly(&s).end(),
            sp.fly_at(&s, 0.0, l.field.intensity).end(),
            "{}",
            l.id
        );
        assert_eq!(ShapeGame::new(l.clone()).intensity, l.field.intensity);
    }
}

#[test]
fn a_source_is_steady_it_ignores_the_clock_and_the_dial() {
    // the difference a source makes to the field does not change with the phase (cmb-7) or the dial (cmb-8)
    let probe = v2(0.3, 0.4);
    let s = [src(SourceKind::Repeller, 0.5, 0.2)];
    let sp7 = ShapeSpec::from_level(&lv("cmb-7"));
    let d = |t: f64| sp7.field_at(&s, 1.0).eval(probe, t) - sp7.field_at(&[], 1.0).eval(probe, t);
    assert!(d(0.0).dist(d(4.7)) < 1e-12);
    assert!(d(0.0).len() > 0.1, "the source does push here");
    let sp8 = ShapeSpec::from_level(&lv("cmb-8"));
    let d8 = |i: f64| sp8.field_at(&s, i).eval(probe, 0.0) - sp8.field_at(&[], i).eval(probe, 0.0);
    assert!(d8(0.2).dist(d8(2.9)) < 1e-12);
    // while the background does move with them
    assert!(
        sp7.field_at(&[], 1.0)
            .eval(probe, 0.0)
            .dist(sp7.field_at(&[], 1.0).eval(probe, 4.7))
            > 0.1
    );
    assert!(
        sp8.field_at(&[], 0.2)
            .eval(probe, 0.0)
            .dist(sp8.field_at(&[], 2.9).eval(probe, 0.0))
            > 0.1
    );
}

#[test]
fn a_launch_records_and_uses_the_phase_and_the_dial() {
    let mut g = ShapeGame::new(lv("cmb-7"));
    g.set_phase(7.0);
    let a = g.launch().clone();
    assert_eq!(a.phase, 7.0);
    let direct = g.spec.fly_at(&[], 7.0, g.spec.default_intensity());
    assert_eq!(a.traj.end(), direct.end());
    let mut g = ShapeGame::new(lv("cmb-8"));
    g.set_intensity(0.5);
    let a = g.launch().clone();
    assert_eq!(a.intensity, 0.5);
    assert_eq!(a.traj.end(), g.spec.fly_at(&[], 0.0, 0.5).end());
    // the arrows the player sees follow the dial
    assert!((g.field().intensity - 0.5).abs() < 1e-12);
}

/// Find a winning (setting, plan) through the public game API, so the test plays the game the way a player does.
fn winning_play(id: &str) -> (f64, ShapeSource) {
    let l = lv(id);
    let axis = axis_of(&l).unwrap();
    let sp = ShapeSpec::from_level(&l);
    for s in axis.settings() {
        for (ki, &k) in sp.kinds.iter().enumerate() {
            for p in sp.sites(0.3) {
                let mut g = ShapeGame::new(l.clone());
                match axis {
                    Axis::Phase(_) => g.set_phase(s),
                    Axis::Dial(..) => g.set_intensity(s),
                    Axis::Start(..) | Axis::Source | Axis::Slots(_) => {
                        unreachable!("Shape levels have no start-point, slot or source control")
                    }
                }
                g.select(ki).unwrap();
                if g.place(p).is_ok() && g.launch().success {
                    return (s, ShapeSource { kind: k, pos: p });
                }
            }
        }
    }
    panic!("{id}: no winning play found");
}

#[test]
fn each_shape_level_is_won_by_the_pair_and_lost_by_either_half() {
    for id in ["cmb-7", "cmb-8"] {
        let l = lv(id);
        let axis = axis_of(&l).unwrap();
        let (setting, plan) = winning_play(id);
        let play = |setting: f64, sources: &[ShapeSource]| {
            let mut g = ShapeGame::new(l.clone());
            match axis {
                Axis::Phase(_) => g.set_phase(setting),
                Axis::Dial(..) => g.set_intensity(setting),
                Axis::Start(..) | Axis::Source | Axis::Slots(_) => {
                    unreachable!("Shape levels have no start-point, slot or source control")
                }
            }
            for s in sources {
                let i = g.spec.kinds.iter().position(|k| *k == s.kind).unwrap();
                g.select(i).unwrap();
                g.place(s.pos).unwrap();
            }
            g.launch().success
        };
        assert!(
            play(setting, &[plan]),
            "{id}: the winning pair does not win through the game"
        );
        assert!(!play(setting, &[]), "{id}: the control alone wins");
        assert!(
            !play(axis.default_setting(), &[plan]),
            "{id}: the source alone, at the default control, wins"
        );
    }
}

// ---------------------------------------------------------------- Steer: the added control
#[test]
fn steer_phase_is_set_before_the_first_beat_and_then_locked() {
    let mut g = SteerGame::new(lv("cmb-11"));
    assert_eq!(g.clock(), 0.0);
    assert!(g.set_clock(14.0));
    assert!((g.clock() - 2.0).abs() < 1e-12, "wraps into the period");
    g.choose(Action::Wait).unwrap();
    assert!(!g.set_clock(5.0), "no change once a beat has started");
    assert!((g.clock() - 2.0).abs() < 1e-12);
    g.finish_beat();
    assert!(!g.set_clock(5.0), "nor between beats");
    // a finished run can be retried, and the phase can be chosen again
    while !matches!(g.phase, Phase::Completed(_)) {
        g.step(Action::Wait).unwrap();
    }
    g.retry().unwrap();
    assert!(g.set_clock(9.0));
    assert!((g.clock() - 9.0).abs() < 1e-12);
}

#[test]
fn a_steer_level_without_a_period_ignores_the_phase() {
    let mut g = SteerGame::new(levels::steer_levels().remove(0));
    assert!(!g.set_clock(3.0));
    assert_eq!(g.clock(), 0.0);
}

#[test]
fn the_phase_moves_the_field_the_player_sees_each_beat() {
    let mut a = SteerGame::new(lv("cmb-11"));
    let mut b = SteerGame::new(lv("cmb-11"));
    b.set_clock(3.0);
    assert!(
        a.field_velocity().dist(b.field_velocity()) > 0.05,
        "the arrows at the start differ with the phase"
    );
    // and the natural flight really starts from that clock
    let n = |g: &SteerGame| {
        vfield::validate_steer::natural_run(&g.spec)
            .path()
            .last()
            .copied()
            .unwrap()
    };
    assert!(n(&a).dist(n(&b)) > 0.3);
    a.set_clock(3.0);
    assert!(n(&a).dist(n(&b)) < 1e-12, "same phase, same flight");
}

#[test]
fn each_steer_phase_has_its_own_solutions_and_the_default_has_none() {
    let l = lv("cmb-11");
    let base = SteerSpec::from_level(&l);
    let at = |p: f64| vfield::validate_steer::enumerate(&base.with_clock(p));
    assert_eq!(
        at(0.0).successes,
        0,
        "the default phase wins: the phase is not needed"
    );
    let best = at(6.0);
    assert!(
        best.successes > 0 && !best.no_gust_success,
        "a good phase needs the gusts too"
    );
    assert!(
        best.min_gusts.is_some_and(|g| g >= 1),
        "at least one gust is always needed"
    );
    // replay a real solution through the game: it wins at its phase and fails at the default phase
    let sol = best.solutions[0].clone();
    let play = |phase: f64| {
        let mut g = SteerGame::new(l.clone());
        g.set_clock(phase);
        let mut end = None;
        for a in &sol {
            end = g.step(*a).unwrap();
        }
        end
    };
    assert_eq!(play(6.0), Some(End::Success));
    assert_ne!(play(0.0), Some(End::Success));
}

#[test]
fn replay_in_the_original_steer_levels_is_unchanged() {
    // clock0 is 0 for a level with no period, so every old beat time is as before
    for l in levels::steer_levels() {
        let sp = SteerSpec::from_level(&l);
        assert_eq!(sp.clock0, 0.0);
        assert_eq!(
            sp.with_clock(5.0).clock0,
            0.0,
            "{}: a level with no period must ignore a phase",
            l.id
        );
        assert!(
            sp.start_zone.is_none() && sp.source_rules.is_none() && sp.sources.is_empty(),
            "{}",
            l.id
        );
        assert_eq!(
            sp.with_start(v2(0.0, 0.0)).start,
            sp.start,
            "{}: a level with no start zone must ignore a start",
            l.id
        );
    }
}

// ---------------------------------------------------------------- Steer: the start-point control (cmb-12)
#[test]
fn steer_start_is_chosen_on_the_line_before_the_first_beat_and_then_locked() {
    let mut g = SteerGame::new(lv("cmb-12"));
    assert_eq!(g.start(), v2(-5.0, 0.0), "the default start");
    assert!(g.set_start(v2(-5.0, 1.75)));
    assert_eq!(g.start(), v2(-5.0, 1.75));
    assert_eq!(g.pos(), v2(-5.0, 1.75), "the probe is where the start is");
    assert!(!g.set_start(v2(-4.0, 1.75)), "off the line");
    assert!(!g.set_start(v2(-5.0, 3.5)), "beyond the end of the line");
    assert_eq!(g.start(), v2(-5.0, 1.75), "a refused start changes nothing");
    g.choose(Action::Wait).unwrap();
    assert!(
        !g.set_start(v2(-5.0, 0.0)),
        "no change once a beat has started"
    );
    g.finish_beat();
    assert!(!g.set_start(v2(-5.0, 0.0)), "nor between beats");
    while !matches!(g.phase, Phase::Completed(_)) {
        g.step(Action::Wait).unwrap();
    }
    g.retry().unwrap();
    assert_eq!(g.start(), v2(-5.0, 1.75), "a retry keeps the chosen start");
    assert!(g.set_start(v2(-5.0, 2.0)), "and it can be chosen again");
}

#[test]
fn a_steer_level_without_a_start_zone_ignores_the_start() {
    let mut g = SteerGame::new(levels::steer_levels().remove(0));
    let s = g.start();
    assert!(!g.set_start(v2(s.x, s.y + 1.0)));
    assert_eq!(g.start(), s);
    // and a phase level has no start zone either
    assert!(!SteerGame::new(lv("cmb-11")).set_start(v2(-5.0, 1.0)));
    assert!(
        !SteerGame::new(lv("cmb-12")).set_clock(3.0),
        "a start-point level has no launch phase"
    );
}

#[test]
fn a_solution_wins_from_its_start_and_fails_from_the_default_start() {
    let l = lv("cmb-12");
    let Axis::Start(zone, dflt) = axis_of(&l).unwrap() else {
        panic!("cmb-12 has a start-point control")
    };
    let base = SteerSpec::from_level(&l);
    let mut found = None;
    for p in start_points(zone, dflt) {
        if !base.start_legal(p) {
            continue;
        }
        let e = vfield::validate_steer::enumerate(&base.with_start(p));
        assert!(!e.no_gust_success, "{p:?}: no gust needed");
        if let Some(sol) = e.solutions.first() {
            found = Some((p, sol.clone()));
            break;
        }
    }
    let (start, sol) = found.expect("some start has a solution");
    assert!(
        start.dist(dflt) > START_DEAD,
        "the first solution found is within the dead band of the default"
    );
    let play = |p: V2| {
        let mut g = SteerGame::new(l.clone());
        g.set_start(p);
        let mut end = None;
        for a in &sol {
            end = g.step(*a).unwrap();
        }
        end
    };
    assert_eq!(play(start), Some(End::Success));
    assert_ne!(
        play(dflt),
        Some(End::Success),
        "the same gusts from the default start also win"
    );
}

#[test]
fn the_start_points_tried_begin_with_the_default_and_cover_the_line() {
    let Axis::Start(zone, dflt) = axis_of(&lv("cmb-12")).unwrap() else {
        panic!()
    };
    let pts = start_points(zone, dflt);
    assert_eq!(pts[0], dflt);
    assert_eq!(
        pts.len(),
        25,
        "a line of 25 lattice points (the default is one of them)"
    );
    assert!(pts.iter().all(|p| p.x == -5.0 && p.y >= -3.0 && p.y <= 3.0));
}

// ---------------------------------------------------------------- Steer: sources placed first (cmb-13)
#[test]
fn steer_sources_follow_the_placement_rules_and_are_locked_after_the_first_beat() {
    let mut g = SteerGame::new(lv("cmb-13"));
    assert!(g.sources().is_empty());
    assert!(g.place_source(SourceKind::Repeller, v2(-3.5, 0.0)).is_ok());
    assert_eq!(g.sources().len(), 1);
    assert_eq!(
        g.place_source(SourceKind::Repeller, v2(-3.0, 1.0)),
        Err(ShapePlaceError::Full),
        "the limit is one source"
    );
    assert!(g.remove_source(0));
    assert!(g.sources().is_empty());
    assert_eq!(
        g.place_source(SourceKind::Repeller, v2(5.0, 0.0)),
        Err(ShapePlaceError::OutsideZone)
    );
    assert!(
        matches!(
            g.place_source(SourceKind::Repeller, v2(-0.94, -0.9)),
            Err(ShapePlaceError::OnRegion(_))
        ),
        "not on the obstacle"
    );
    assert!(
        matches!(
            g.place_source(SourceKind::Repeller, v2(3.76, -2.05)),
            Err(ShapePlaceError::OutsideZone | ShapePlaceError::OnRegion(_))
        ),
        "not on the target"
    );
    assert_eq!(
        g.place_source(SourceKind::SwirlCcw, v2(-3.5, 0.0)),
        Err(ShapePlaceError::KindUnavailable),
        "only repellers and attractors"
    );
    assert!(g.sources().is_empty(), "refused placements change nothing");
    g.place_source(SourceKind::Attractor, v2(-3.5, 0.0))
        .unwrap();
    g.choose(Action::Wait).unwrap();
    assert!(
        g.place_source(SourceKind::Repeller, v2(-3.0, 1.0)).is_err(),
        "no placing once a beat has started"
    );
    assert!(!g.remove_source(0) && !g.clear_sources(), "nor removing");
    assert_eq!(g.sources().len(), 1);
    g.finish_beat();
    while !matches!(g.phase, Phase::Completed(_)) {
        g.step(Action::Wait).unwrap();
    }
    g.retry().unwrap();
    assert_eq!(g.sources().len(), 1, "a retry keeps the placed source");
    assert!(g.clear_sources(), "and it can be changed again");
}

#[test]
fn a_steer_level_without_sources_offers_none() {
    let mut g = SteerGame::new(levels::steer_levels().remove(0));
    assert_eq!(
        g.place_source(SourceKind::Repeller, v2(0.0, 0.0)),
        Err(ShapePlaceError::KindUnavailable)
    );
    assert!(!g.remove_source(0) && !g.clear_sources());
    assert!(
        SteerSpec::from_level(&lv("cmb-12")).source_rules.is_none()
            && SteerSpec::from_level(&lv("cmb-11")).source_rules.is_none()
    );
}

#[test]
fn a_placed_source_changes_the_field_steadily_and_does_not_stack() {
    let base = SteerSpec::from_level(&lv("cmb-13"));
    let a = ShapeSource {
        kind: SourceKind::Repeller,
        pos: v2(-3.5, 0.0),
    };
    let with = base.with_sources(&[a]);
    let p = v2(-3.0, 0.3);
    let diff = |t: f64| with.field.eval(p, t) - base.field.eval(p, t);
    assert!(diff(0.0).len() > 0.1, "the source pushes here");
    assert!(
        diff(0.0).dist(diff(5.3)) < 1e-12,
        "and does not depend on the clock"
    );
    assert!(
        with.with_sources(&[a])
            .field
            .eval(p, 0.0)
            .dist(with.field.eval(p, 0.0))
            < 1e-12,
        "placing again does not add a second term"
    );
    assert!(
        with.with_sources(&[])
            .field
            .eval(p, 0.0)
            .dist(base.field.eval(p, 0.0))
            < 1e-12,
        "and removing it restores the level's own field"
    );
    assert_eq!(with.sources.len(), 1);
}

#[test]
fn a_solution_wins_with_its_source_and_fails_without_the_source_or_without_the_gust() {
    let l = lv("cmb-13");
    let base = SteerSpec::from_level(&l);
    let mut found = None;
    'o: for &k in &[SourceKind::Repeller, SourceKind::Attractor] {
        for p in base.sites(0.5) {
            let e = vfield::validate_steer::enumerate(
                &base.with_sources(&[ShapeSource { kind: k, pos: p }]),
            );
            assert!(!e.no_gust_success, "{k:?} at {p:?}: the source alone wins");
            if let Some(sol) = e.solutions.first() {
                found = Some((k, p, sol.clone()));
                break 'o;
            }
        }
    }
    let (kind, pos, sol) = found.expect("some source has a solution");
    let play = |with_source: bool, gusts: bool| {
        let mut g = SteerGame::new(l.clone());
        if with_source {
            g.place_source(kind, pos).unwrap();
        }
        let mut end = None;
        for a in &sol {
            match g.step(if gusts { *a } else { Action::Wait }) {
                Ok(e) => end = e,
                Err(_) => break, // the run has already ended (for example on the obstacle)
            }
        }
        end
    };
    assert_eq!(play(true, true), Some(End::Success));
    assert_ne!(
        play(false, true),
        Some(End::Success),
        "the same gusts with no source also win"
    );
    assert_ne!(
        play(true, false),
        Some(End::Success),
        "the source with no gusts also wins"
    );
}

// ---------------------------------------------------------------- Synchronize: sources placed first (cmb-9)
#[test]
fn sync_sources_follow_the_placement_rules() {
    let mut g = SyncGame::new(lv("cmb-9"));
    assert!(g.sources().is_empty());
    assert!(g.place_source(SourceKind::Attractor, v2(2.5, 1.0)).is_ok());
    assert_eq!(
        g.place_source(SourceKind::Repeller, v2(0.0, 2.0)),
        Err(ShapePlaceError::Full),
        "the limit is one source"
    );
    assert!(g.remove_source(0) && g.sources().is_empty());
    assert_eq!(
        g.place_source(SourceKind::Repeller, v2(-5.0, 2.0)),
        Err(ShapePlaceError::OutsideZone),
        "left of the area"
    );
    assert_eq!(
        g.place_source(SourceKind::Repeller, v2(0.0, -1.0)),
        Err(ShapePlaceError::OutsideZone),
        "below the area (it is the upper half)"
    );
    assert_eq!(
        g.place_source(SourceKind::SwirlCw, v2(0.0, 2.0)),
        Err(ShapePlaceError::KindUnavailable),
        "only repellers and attractors"
    );
    assert!(g.sources().is_empty(), "refused placements change nothing");
    // unlike a Steer run, a Synchronize attempt can change its source between launches
    g.place_source(SourceKind::Repeller, v2(0.0, 2.0)).unwrap();
    g.launch();
    assert!(g.clear_sources(), "a source can be changed after a launch");
}

#[test]
fn a_synchronize_level_without_sources_offers_none_and_the_old_levels_are_unchanged() {
    let mut g = SyncGame::new(levels::sync_levels().remove(0));
    assert_eq!(
        g.place_source(SourceKind::Repeller, v2(0.0, 0.0)),
        Err(ShapePlaceError::KindUnavailable)
    );
    assert!(!g.remove_source(0) && !g.clear_sources());
    for l in levels::sync_levels() {
        let sp = SyncSpec::from_level(&l);
        assert!(
            sp.source_rules.is_none() && sp.sources.is_empty(),
            "{}",
            l.id
        );
        assert_eq!(
            sp.with_sources(&[ShapeSource {
                kind: SourceKind::Repeller,
                pos: v2(0.0, 0.0)
            }])
            .sources
            .len(),
            0,
            "{}: a level with no sources ignores them",
            l.id
        );
    }
}

#[test]
fn a_placed_source_changes_the_field_every_probe_flies_steadily_and_does_not_stack() {
    let base = SyncSpec::from_level(&lv("cmb-9"));
    let a = ShapeSource {
        kind: SourceKind::Attractor,
        pos: v2(2.5, 1.0),
    };
    let with = base.with_sources(&[a]);
    let p = v2(2.0, 0.8);
    let diff = |t: f64| with.field.eval(p, t) - base.field.eval(p, t);
    assert!(diff(0.0).len() > 0.1, "the source pulls here");
    assert!(
        diff(0.0).dist(diff(5.3)) < 1e-12,
        "and does not pulse with the clock"
    );
    assert!(
        with.with_sources(&[a])
            .field
            .eval(p, 1.0)
            .dist(with.field.eval(p, 1.0))
            < 1e-12,
        "placing again does not add a second term"
    );
    assert!(
        with.with_sources(&[])
            .field
            .eval(p, 1.0)
            .dist(base.field.eval(p, 1.0))
            < 1e-12
    );
    // and both probes really fly in it
    let moved = (0..2)
        .filter(|&i| with.fly_one(i, 4.0).end().dist(base.fly_one(i, 4.0).end()) > 0.05)
        .count();
    assert!(moved >= 1, "at least one probe's flight changes");
}

#[test]
fn a_solution_wins_with_its_source_and_fails_without_it_or_at_the_default_delays() {
    let l = lv("cmb-9");
    let base = SyncSpec::from_level(&l);
    let mut found = None;
    'o: for &k in &[SourceKind::Repeller, SourceKind::Attractor] {
        for p in base.sites(0.5) {
            let spec = base.with_sources(&[ShapeSource { kind: k, pos: p }]);
            assert!(
                !spec.fly(&[0.0, 0.0]).success,
                "{k:?} at {p:?}: the source wins at the default delays"
            );
            for a in 0..61 {
                for b in 0..61 {
                    let d = [a as f64 * COMBO_DELAY_STEP, b as f64 * COMBO_DELAY_STEP];
                    if spec.fly(&d).success {
                        found = Some((k, p, d));
                        break 'o;
                    }
                }
            }
        }
    }
    let (kind, pos, d) = found.expect("some source has a winning pair of delays");
    assert!(
        d[0].max(d[1]) > DELAY_DEAD_S,
        "the first solution found is within the dead band of the default delays"
    );
    let play = |with_source: bool, delays: [f64; 2]| {
        let mut g = SyncGame::new(l.clone());
        if with_source {
            g.place_source(kind, pos).unwrap();
        }
        g.set_delay(0, delays[0]);
        g.set_delay(1, delays[1]);
        g.launch().success
    };
    assert!(
        play(true, d),
        "the winning pair does not win through the game"
    );
    assert!(!play(false, d), "the same delays win with no source");
    assert!(
        !play(true, [0.0, 0.0]),
        "the source wins at the default delays"
    );
}

#[test]
fn no_solution_comes_near_releasing_everything_at_once() {
    let r = rep("cmb-9");
    assert!(
        r.nearest_win_s.is_some_and(|m| m >= 3.0),
        "nearest solution only {:?} s from the default delays",
        r.nearest_win_s
    );
    assert!(
        rep("cmb-7").nearest_win_s.is_none(),
        "the measure is for Synchronize levels only"
    );
}

// ---------------------------------------------------------------- Synchronize: a start slot chosen first (cmb-10)
#[test]
fn start_slots_are_a_numbered_choice_and_the_probe_starts_from_the_chosen_one() {
    let mut g = SyncGame::new(lv("cmb-10"));
    let (probe, slots) = g.spec.slots.clone().expect("cmb-10 offers start slots");
    assert_eq!((probe, slots.len(), g.spec.slot), (1, 5, 0));
    assert_eq!(
        g.spec.starts[probe], slots[0],
        "slot 1 is the default start"
    );
    assert!(
        !g.set_slot(0),
        "choosing the slot already chosen changes nothing"
    );
    assert!(!g.set_slot(slots.len()), "there is no sixth slot");
    assert!(g.set_slot(3));
    assert_eq!(g.spec.starts[probe], slots[3]);
    assert!(
        g.spec.fly_one(probe, 2.0).points[0].dist(slots[3]) < 1e-9,
        "the flight begins at the chosen slot"
    );
    assert!(
        g.spec.fly_one(0, 2.0).points[0].dist(g.spec.starts[0]) < 1e-9,
        "the other probe is not affected"
    );
    // the other levels have no slots
    for l in levels::sync_levels() {
        let mut g = SyncGame::new(l);
        assert!(g.spec.slots.is_none() && !g.set_slot(1));
    }
}

#[test]
fn cmb_10_needs_the_slot_and_the_delays_and_a_win_exists() {
    let l = lv("cmb-10");
    let r = rep("cmb-10");
    assert_eq!(r.axis, "start slot");
    assert_eq!(r.default_wins, 0, "no delays win from the default slot");
    assert_eq!(r.none_wins, 0, "releasing both at once wins from no slot");
    assert_eq!(
        r.dead_wins, 0,
        "no pair of delays within 1.5 s of a joint release wins from any slot"
    );
    assert!(
        r.settings_with_wins >= 1 && r.settings_with_wins <= 3,
        "{} of {} slots win",
        r.settings_with_wins,
        r.settings
    );
    assert!(
        r.peak_wins >= 15,
        "the winning delays are a wide set, not a knife edge ({})",
        r.peak_wins
    );
    // play it: scan the slots and the delays through the real game
    let mut won_from = vec![];
    for slot in 0..5 {
        let mut g = SyncGame::new(l.clone());
        g.set_slot(slot);
        'scan: for a in 0..=30 {
            for b in 0..=30 {
                g.set_delay(0, a as f64 * 0.4);
                g.set_delay(1, b as f64 * 0.4);
                if g.launch().success {
                    won_from.push(slot);
                    break 'scan;
                }
            }
        }
    }
    assert!(
        !won_from.is_empty() && !won_from.contains(&0),
        "won from slots {won_from:?}"
    );
    assert!(won_from.len() <= 3);
}

#[test]
fn the_catalog_lists_the_start_slots() {
    let text = vfield::catalog::render();
    assert!(text.contains("numbered start slots (slot 1 is its default)"));
    assert_eq!(
        text.matches("numbered start slots").count(),
        1,
        "only cmb-10 has them"
    );
}

// ---------------------------------------------------------------- the select screen groups the levels in families
#[test]
fn combination_levels_are_grouped_in_contiguous_families_that_fit_the_select_screen() {
    let ids: Vec<_> = levels::chapter(Chapter::Combo)
        .iter()
        .map(|l| l.id)
        .collect();
    let fams: Vec<usize> = ids.iter().map(|id| levels::combo_family(id)).collect();
    assert!(
        fams.windows(2).all(|w| w[0] <= w[1]),
        "families must be contiguous and in order: {fams:?}"
    );
    for f in 0..levels::COMBO_FAMILIES.len() {
        assert!(fams.contains(&f), "family {f} is empty");
    }
    assert_eq!(levels::combo_family("cmb-10"), 1, "the two-control levels");
    assert_eq!(levels::combo_family("cmb-13"), 2, "set up, then steer");
    assert_eq!(levels::combo_family("cmb-17"), 3, "learn, then act");
    assert_eq!(levels::combo_family("cmb-20"), 4, "three decisions");
    // the select screen puts families 0 and 1 in the left column and the others in the right one. Each column's
    // rows shrink to fit (44 px at most, 36 px at least; headings 26 px, from y = 160, ending above the Back button
    // at y = 740), so what must hold is that the columns fit with the smallest rows.
    for col in 0..2 {
        let in_col = |f: usize| (if f < 2 { 0 } else { 1 }) == col;
        let rows = fams.iter().filter(|&&f| in_col(f)).count();
        let heads = (0..levels::COMBO_FAMILIES.len())
            .filter(|&f| in_col(f))
            .count();
        assert!(
            160 + rows * 36 + heads * 26 <= 735,
            "column {col} has {rows} rows: too many even at 36 px"
        );
        let pitch = ((575.0 - 26.0 * heads as f32) / rows as f32).clamp(36.0, 44.0);
        assert!(pitch >= 36.0);
    }
}
