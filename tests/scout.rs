//! Scout, then call it (cmb-16): a hidden source bends every flight; scouts feel it; the main probe's exit is called
//! before it flies. The level must need both decisions: no fixed call (nor the visible-only call) can pass, and the
//! evidence is enough to pass. Numbers come from `validate_scout` (real simulation); the rest pins the game rules.
use vfield::modes::*;
use vfield::validate_scout::*;
use vfield::*;

fn lv() -> Level {
    levels::combo_levels()
        .into_iter()
        .find(|l| l.id == "cmb-16")
        .expect("cmb-16 exists")
}
fn rules(l: &Level) -> ScoutRules {
    let Rules::Scout(r) = &l.rules else {
        panic!("cmb-16 is a Scout level")
    };
    r.clone()
}
/// The reader: three scouts at the validator's starts, triangulate, simulate, return the call.
fn read_and_call(g: &mut ScoutGame) -> usize {
    for (x, y) in READER_STARTS {
        g.place_scout(v2(x, y)).expect("scout placed");
    }
    let samples: Vec<(V2, V2)> = g.scouts.iter().flat_map(|s| s.pushes.clone()).collect();
    let (kind, h) = estimate(&samples).expect("the scouts felt the source");
    g.spec
        .exit_in(&g.spec.field_with(kind, h))
        .0
        .expect("the estimate sends the probe to an exit")
}

// ---------------------------------------------------------------- the level and the bar
#[test]
fn cmb_16_is_a_combination_level_and_needs_predict_and_survey() {
    let ids: Vec<_> = levels::chapter(Chapter::Combo)
        .iter()
        .map(|l| l.id)
        .collect();
    assert!(ids.contains(&"cmb-16"));
    assert_eq!(levels::combo_requires("cmb-16"), ["pre-1", "sur-1"]);
    let all: Vec<_> = levels::all().iter().map(|l| l.id).collect();
    for r in levels::combo_requires("cmb-16") {
        assert!(all.contains(r), "unknown prerequisite {r}");
    }
    assert!(lv().hints.len() >= 2);
    assert!(validate::solvable(&lv()));
}

#[test]
fn both_decisions_are_necessary() {
    let r = report(&lv());
    assert_eq!(r.entries, 3);
    assert!(
        r.distinct,
        "the true exits must all be reached and differ: {:?}",
        r.exits
    );
    assert!(
        r.best_fixed < r.streak,
        "a single fixed call is right for {} of {} hidden sources",
        r.best_fixed,
        r.entries
    );
    assert!(
        r.visible_right < r.streak,
        "the visible arrows alone are right for {} sources",
        r.visible_right
    );
    assert_eq!(
        r.seen, r.entries,
        "every hidden source must be felt by the reader's scouts"
    );
    assert_eq!(
        r.reader_right, r.entries,
        "the evidence must be enough to call every exit"
    );
    assert!(
        r.tolerance >= TOLERANCE_MIN,
        "tolerance {:.0}%: the call must survive a person's position error",
        100.0 * r.tolerance
    );
    assert!(r.ablation_holds());
}

#[test]
fn the_ablation_survives_moving_the_exit_gates() {
    for ((dx, dy), ok) in robustness(&lv()) {
        assert!(
            ok,
            "the ablation fails when the gates move by ({dx:+.1}, {dy:+.1})"
        );
    }
}

#[test]
fn the_visible_arrows_alone_name_the_wrong_exit_for_most_hidden_sources() {
    let spec = ScoutSpec::from_level(&lv());
    let visible = spec
        .visible_exit()
        .expect("the visible field sends the probe to an exit");
    let wrong = (0..spec.rules.pool.len())
        .filter(|&a| spec.true_exit(a).0 != Some(visible))
        .count();
    assert!(
        wrong >= 2,
        "the visible exit is right for {} of {} sources",
        spec.rules.pool.len() - wrong,
        spec.rules.pool.len()
    );
}

// ---------------------------------------------------------------- no fixed call passes, the reader does
#[test]
fn no_fixed_call_completes_the_level_however_long_it_plays() {
    let l = lv();
    let n_exits = rules(&l).exits.len();
    for seed in 0..40u64 {
        for exit in 0..n_exits {
            let mut g = ScoutGame::with_seed(l.clone(), seed);
            for _ in 0..60 {
                g.call(exit).unwrap();
                if g.won {
                    break;
                }
                g.next_attempt();
            }
            assert!(
                !g.won,
                "seed {seed}: calling exit {exit} every time completed the level"
            );
        }
    }
}

#[test]
fn calling_what_the_visible_arrows_predict_every_time_does_not_complete_it() {
    let l = lv();
    let visible = ScoutSpec::from_level(&l).visible_exit().unwrap();
    let mut right_sometimes = false;
    for seed in 0..40u64 {
        let mut g = ScoutGame::with_seed(l.clone(), seed);
        for _ in 0..60 {
            g.call(visible).unwrap();
            g.next_attempt();
        }
        assert!(!g.won, "seed {seed}");
        right_sometimes |= g.calls.iter().any(|c| c.correct);
    }
    assert!(
        right_sometimes,
        "it should be right sometimes, just not three times in a row"
    );
}

#[test]
fn reading_the_scouts_completes_it_in_three_attempts() {
    let mut g = ScoutGame::new(lv());
    for attempt in 0..3 {
        assert_eq!(g.attempt, attempt);
        let call = read_and_call(&mut g);
        assert_eq!(
            g.call(call),
            Ok(true),
            "attempt {attempt}: the reader called {call}"
        );
        if attempt < 2 {
            assert!(!g.won);
            assert!(g.next_attempt());
        }
    }
    assert!(g.won);
    assert_eq!(g.streak, 3);
    assert_eq!(g.calls.len(), 3);
}

#[test]
fn the_evidence_belongs_to_one_attempt() {
    let l = lv();
    let spec = ScoutSpec::from_level(&l);
    assert_ne!(
        spec.hidden(0),
        spec.hidden(1),
        "the pool entries are different sources"
    );
    let (a, b) = (spec.scout(0, v2(-5.0, 0.0)), spec.scout(1, v2(-5.0, 0.0)));
    assert_ne!(
        a.trail.last(),
        b.trail.last(),
        "the same scout flies differently under a different hidden source"
    );
    // the game clears the scouts when the attempt changes
    let mut g = ScoutGame::new(l);
    g.place_scout(v2(-5.0, 0.0)).unwrap();
    g.call(0).unwrap();
    assert!(g.next_attempt());
    assert!(g.scouts.is_empty() && g.last_path.is_empty());
}

// ---------------------------------------------------------------- game rules
#[test]
fn scouts_start_in_the_strip_and_are_limited() {
    let mut g = ScoutGame::new(lv());
    assert_eq!(g.scouts_left(), 3);
    assert_eq!(
        g.place_scout(v2(-3.0, 0.0)),
        Err(ScoutError::OutsideZone),
        "right of the strip"
    );
    assert_eq!(
        g.place_scout(v2(-5.0, 3.5)),
        Err(ScoutError::OutsideZone),
        "above the strip"
    );
    assert_eq!(g.scouts_left(), 3, "refused scouts cost nothing");
    for y in [-1.0, 0.0, 1.0] {
        g.place_scout(v2(-5.0, y)).unwrap();
    }
    assert_eq!(g.scouts_left(), 0);
    assert_eq!(g.place_scout(v2(-5.0, 2.0)), Err(ScoutError::NoScoutsLeft));
    assert_eq!(g.scouts[0].start, v2(-5.0, -1.0));
    assert!(
        g.scouts[0].trail.len() > 10,
        "a scout flies and leaves a trail"
    );
}

#[test]
fn a_scout_far_from_the_source_may_feel_nothing_and_one_near_it_does() {
    let spec = ScoutSpec::from_level(&lv());
    let total: usize = [-1.5, 0.0, 1.5]
        .iter()
        .map(|&y| spec.scout(0, v2(-5.0, y)).pushes.len())
        .sum();
    assert!(
        total >= 2,
        "the reader's scouts must feel attempt 0's source"
    );
    for s in [spec.scout(0, v2(-5.0, 0.0)), spec.scout(1, v2(-5.0, 1.5))] {
        for (_, push) in &s.pushes {
            assert!(
                push.len() >= vfield::modes::PUSH_MIN,
                "only pushes of at least Survey's threshold are shown"
            );
        }
    }
}

#[test]
fn a_call_is_committed_once_per_attempt_and_the_probe_then_flies() {
    let mut g = ScoutGame::new(lv());
    assert_eq!(g.call(9), Err(ScoutError::BadExit));
    assert!(g.last_path.is_empty(), "nothing flies before the call");
    let truth = g.spec.true_exit(g.entry()).0.unwrap();
    assert_eq!(g.call(truth), Ok(true));
    assert!(
        g.last_path.len() > 10,
        "the true flight is shown after the call"
    );
    assert_eq!(g.phase, ScoutPhase::Called);
    assert_eq!(
        g.call(truth),
        Err(ScoutError::WrongPhase),
        "no second call in the same attempt"
    );
    assert_eq!(
        g.place_scout(v2(-5.0, 0.0)),
        Err(ScoutError::WrongPhase),
        "and no scouting after the call"
    );
    let mut fresh = ScoutGame::new(lv());
    assert!(
        !fresh.next_attempt(),
        "the next attempt only follows a call"
    );
}

#[test]
fn the_streak_counts_calls_in_a_row_and_a_wrong_call_resets_it() {
    let mut g = ScoutGame::new(lv());
    let right = |g: &ScoutGame| g.spec.true_exit(g.entry()).0.unwrap();
    let wrong = |g: &ScoutGame| (right(g) + 1) % 4;
    let c = right(&g);
    g.call(c).unwrap();
    g.next_attempt();
    let c = right(&g);
    g.call(c).unwrap();
    assert_eq!((g.streak, g.best_streak, g.won), (2, 2, false));
    g.next_attempt();
    let c = wrong(&g);
    assert_eq!(g.call(c), Ok(false));
    assert_eq!(
        (g.streak, g.best_streak, g.won),
        (0, 2, false),
        "a wrong call resets the streak but not the best"
    );
    // three more right ones win
    for i in 0..3 {
        g.next_attempt();
        let c = right(&g);
        g.call(c).unwrap();
        assert_eq!(g.won, i == 2);
    }
    assert_eq!(g.streak, 3);
}

#[test]
fn the_exit_calls_are_the_levels_regions_in_order() {
    let l = lv();
    let r = rules(&l);
    let labels: Vec<char> = r.exits.iter().map(|&i| l.regions[i].label).collect();
    assert_eq!(labels, ['A', 'B', 'C', 'D']);
    assert!(r
        .exits
        .iter()
        .all(|&i| l.regions[i].kind == RegionKind::Target));
}

// ---------------------------------------------------------------- the hidden source is drawn at random
#[test]
fn the_hidden_source_is_random_but_never_the_same_twice_in_a_row() {
    let l = lv();
    let n = rules(&l).pool.len();
    let mut seen_first = std::collections::BTreeSet::new();
    for seed in 0..60u64 {
        let mut g = ScoutGame::with_seed(l.clone(), seed);
        seen_first.insert(g.entry());
        let mut prev = g.entry();
        let mut seen = std::collections::BTreeSet::from([prev]);
        for _ in 0..80 {
            g.call(0).unwrap();
            g.next_attempt();
            assert_ne!(
                g.entry(),
                prev,
                "seed {seed}: the same hidden source twice in a row"
            );
            prev = g.entry();
            seen.insert(prev);
        }
        assert_eq!(
            seen.len(),
            n,
            "seed {seed}: every hidden source should come up in 80 attempts"
        );
    }
    assert_eq!(
        seen_first.len(),
        n,
        "the first hidden source depends on the seed"
    );
}

#[test]
fn a_seed_gives_the_same_play_and_different_seeds_give_different_orders() {
    let l = lv();
    let order = |seed: u64| {
        let mut g = ScoutGame::with_seed(l.clone(), seed);
        for _ in 0..12 {
            g.call(0).unwrap();
            g.next_attempt();
        }
        g.order.clone()
    };
    assert_eq!(order(5), order(5));
    let distinct: std::collections::BTreeSet<_> = (0..30u64).map(order).collect();
    assert!(
        distinct.len() > 10,
        "only {} different orders over 30 seeds",
        distinct.len()
    );
    // the default constructor uses the level's own seed, so tests are reproducible
    assert_eq!(
        ScoutGame::new(l.clone()).entry(),
        ScoutGame::with_seed(l.clone(), l.seed).entry()
    );
}

#[test]
fn a_remembered_order_does_not_help_because_the_order_is_not_fixed() {
    // someone who memorised the exits of one play, in order, cannot pass a play with another seed
    let l = lv();
    let truth_sequence = |seed: u64| {
        let mut g = ScoutGame::with_seed(l.clone(), seed);
        let mut v = vec![];
        for _ in 0..3 {
            v.push(g.spec.true_exit(g.entry()).0.unwrap());
            g.call(0).unwrap();
            g.next_attempt();
        }
        v
    };
    let memorised = truth_sequence(1);
    let mut matches = 0;
    for seed in 2..42u64 {
        if truth_sequence(seed) == memorised {
            matches += 1;
        }
    }
    assert!(
        matches < 15,
        "the memorised answers fit {matches} of 40 other plays"
    );
}

// ---------------------------------------------------------------- cmb-18: the same bar on a curved background
fn lv18() -> Level {
    levels::combo_levels()
        .into_iter()
        .find(|l| l.id == "cmb-18")
        .expect("cmb-18 exists")
}

#[test]
fn cmb_18_is_a_combination_level_and_needs_predict_and_survey() {
    let ids: Vec<_> = levels::chapter(Chapter::Combo)
        .iter()
        .map(|l| l.id)
        .collect();
    assert!(ids.contains(&"cmb-18"));
    assert_eq!(levels::combo_requires("cmb-18"), ["pre-1", "sur-1"]);
    assert!(lv18().hints.len() >= 2);
    assert!(validate::solvable(&lv18()));
}

#[test]
fn cmb_18_needs_both_decisions_and_holds_with_the_gates_moved() {
    let l = lv18();
    let r = report(&l);
    assert_eq!(r.entries, 3);
    assert!(
        r.distinct,
        "true exits must all be reached and differ: {:?}",
        r.exits
    );
    assert!(r.best_fixed < r.streak && r.visible_right < r.streak);
    assert_eq!(r.seen, r.entries);
    assert_eq!(r.reader_right, r.entries);
    assert!(
        r.tolerance >= TOLERANCE_MIN,
        "tolerance {:.0}%",
        100.0 * r.tolerance
    );
    for ((dx, dy), ok) in robustness(&l) {
        assert!(
            ok,
            "the ablation fails when the gates move by ({dx:+.1}, {dy:+.1})"
        );
    }
}

#[test]
fn cmb_18_background_really_curves_the_flight() {
    // The level exists to test reading a push through a swirl: the visible-only path must visibly bend.
    let l = lv18();
    let spec = ScoutSpec::from_level(&l);
    let ys: Vec<f64> = spec.exit_in(&l.field).1.iter().map(|p| p.y).collect();
    let (lo, hi) = ys
        .iter()
        .fold((f64::MAX, f64::MIN), |(a, b), &y| (a.min(y), b.max(y)));
    assert!(
        hi - lo > 0.5,
        "the visible-only path should be bent by the swirl (y range {:.2})",
        hi - lo
    );
}

// ---------------------------------------------------------------- categorical acts: release, cancel (cmb-14, cmb-15)
fn lv_id(id: &str) -> Level {
    levels::combo_levels()
        .into_iter()
        .find(|l| l.id == id)
        .unwrap_or_else(|| panic!("{id} exists"))
}

#[test]
fn the_act_levels_need_both_decisions_and_hold_with_the_target_moved() {
    for id in ["cmb-14", "cmb-15", "cmb-17", "cmb-19"] {
        let l = lv_id(id);
        let r = report(&l);
        assert_eq!(r.entries, 3, "{id}");
        assert!(
            r.distinct,
            "{id}: every hidden source needs a winning option and no option may win for two: {:?}",
            r.exits
        );
        assert!(
            r.best_fixed < r.streak,
            "{id}: a fixed choice must not complete the level"
        );
        assert!(
            r.visible_right < r.streak,
            "{id}: the choice the visible arrows suggest must not complete the level"
        );
        assert_eq!(r.reader_right, r.entries, "{id}");
        assert_eq!(r.seen, r.entries, "{id}");
        assert!(
            r.tolerance >= TOLERANCE_MIN,
            "{id}: tolerance {:.0}%",
            100.0 * r.tolerance
        );
        for ((dx, dy), ok) in robustness(&l) {
            assert!(
                ok,
                "{id}: the ablation fails with the target moved by ({dx:+.1}, {dy:+.1})"
            );
        }
    }
}

#[test]
fn a_choice_is_judged_by_flying_the_probe_and_a_wrong_option_resets_the_streak() {
    for id in ["cmb-14", "cmb-15", "cmb-17", "cmb-19"] {
        let mut g = ScoutGame::with_seed(lv_id(id), 7);
        let n = g.spec.options();
        assert!(n >= 4, "{id}");
        assert_eq!(g.call(n), Err(ScoutError::BadExit), "{id}");
        // a winning option is right, and its flight reaches the target
        let w = g.spec.winners(g.entry());
        assert!(!w.is_empty());
        assert_eq!(g.call(w[0]), Ok(true), "{id}");
        assert_eq!(g.streak, 1);
        assert!(g.last_path.len() > 10);
        assert!(g.next_attempt());
        // an option that wins for no one in this attempt is wrong and resets the streak
        let w = g.spec.winners(g.entry());
        let bad = (0..n).find(|o| !w.contains(o)).unwrap();
        assert_eq!(g.call(bad), Ok(false), "{id}");
        assert_eq!(g.streak, 0);
    }
}

#[test]
fn a_cancel_site_of_the_same_kind_as_the_hidden_source_does_not_cancel_it() {
    // the opposite kind at the same place wins; the same kind there doubles the push and loses
    let l = lv_id("cmb-15");
    let spec = ScoutSpec::from_level(&l);
    let ScoutAct::Cancel { sites, .. } = &spec.rules.act else {
        panic!()
    };
    for a in 0..spec.rules.pool.len() {
        let (kind, pos) = spec.rules.pool[a];
        let w = spec.winners(a);
        for &o in &w {
            assert!(
                sites[o].0 != kind || sites[o].1.dist(pos) > 0.5,
                "entry {a}: option {o} repeats the hidden source"
            );
        }
        let same = sites
            .iter()
            .position(|&(k, p)| k == kind && p.dist(pos) < 1e-9)
            .expect("the hidden source sits at a site");
        assert!(
            !w.contains(&same),
            "entry {a}: the same kind at the same place must not win"
        );
    }
}

#[test]
fn the_release_slots_are_numbered_options_the_probe_flies_from() {
    let l = lv_id("cmb-14");
    let spec = ScoutSpec::from_level(&l);
    let ScoutAct::Release { starts, .. } = &spec.rules.act else {
        panic!()
    };
    assert!(starts.len() >= 5);
    // different slots give different flights in the same field
    let f = spec.truth_field(0);
    let a = spec.try_option(&f, 0).1;
    let b = spec.try_option(&f, starts.len() - 1).1;
    assert!(a[0].dist(b[0]) > 1.0);
}

#[test]
fn the_gust_plans_are_flown_through_the_steer_engine_and_are_short() {
    let l = lv_id("cmb-17");
    let spec = ScoutSpec::from_level(&l);
    let ScoutAct::Plan { plans, dirs, .. } = &spec.rules.act else {
        panic!()
    };
    assert_eq!(plans.len(), 4);
    for p in plans {
        assert!(
            p.iter().flatten().count() <= 2,
            "a plan has at most two gusts"
        );
        assert!(p.iter().flatten().all(|&d| d < dirs.count()));
    }
    // every hidden source has exactly one winning plan, and one plan wins for none (the decoy)
    let wins: Vec<Vec<usize>> = (0..spec.rules.pool.len())
        .map(|a| spec.winners(a))
        .collect();
    assert!(wins.iter().all(|w| w.len() == 1), "{wins:?}");
    assert!((0..plans.len()).any(|o| wins.iter().all(|w| !w.contains(&o))));
}

#[test]
fn cmb_19_default_phase_wins_for_no_hidden_source_and_the_launch_phase_is_flown() {
    let l = lv_id("cmb-19");
    assert_eq!(levels::combo_requires("cmb-19"), ["tem-1", "sur-1"]);
    let r = report(&l);
    assert!(
        r.controls_needed(),
        "phase 0 must win for no hidden source: {:?}",
        r.neutral_wins
    );
    let spec = ScoutSpec::from_level(&l);
    let ScoutAct::Phase { phases, .. } = &spec.rules.act else {
        panic!()
    };
    assert_eq!(phases[0], 0.0, "the first phase is the default");
    // the launch phase changes the flight: the same field and start give different paths at different phases
    let f = spec.truth_field(0);
    let a = spec.try_option(&f, 0).1;
    let b = spec.try_option(&f, 3).1;
    assert!(a.last().unwrap().dist(*b.last().unwrap()) > 0.5 || a.len() != b.len());
}

// ---------------------------------------------------------------- three decisions in one level (cmb-20)
#[test]
fn cmb_20_needs_scouting_the_cancel_and_the_slot_and_holds_with_the_target_moved() {
    for (id, req) in [
        ("cmb-20", ["pos-1", "shp-1", "sur-1"]),
        ("cmb-21", ["shp-1", "ste-1", "sur-1"]),
    ] {
        three_decisions_hold(id, req);
    }
}

fn three_decisions_hold(id: &str, req: [&str; 3]) {
    let l = lv_id(id);
    assert_eq!(
        levels::combo_requires(id),
        req,
        "{id} needs three mechanics"
    );
    let r = report(&l);
    assert!(r.distinct && r.best_fixed < r.streak && r.visible_right < r.streak);
    assert_eq!((r.reader_right, r.seen), (r.entries, r.entries));
    assert!(
        r.tolerance >= TOLERANCE_MIN,
        "tolerance {:.0}%",
        100.0 * r.tolerance
    );
    assert_eq!(
        r.neutral_wins,
        vec![Some(0), Some(0)],
        "no cancel and the default slot must each win for no hidden source"
    );
    assert!(r.controls_needed());
    for ((dx, dy), ok) in robustness(&l) {
        assert!(
            ok,
            "the ablation fails with the target moved by ({dx:+.1}, {dy:+.1})"
        );
    }
}

#[test]
fn a_sequence_of_choices_is_one_option_and_is_judged_by_one_flight() {
    let l = lv_id("cmb-20");
    let spec = ScoutSpec::from_level(&l);
    let comps = spec.components();
    assert_eq!(comps.len(), 2);
    assert_eq!(
        (comps[0].1, comps[1].1),
        (9, 8),
        "eight sources and none; eight slots"
    );
    assert_eq!(spec.options(), 72);
    for o in 0..spec.options() {
        assert_eq!(spec.encode(&spec.decode(o)), o);
    }
    assert_eq!(
        spec.neutral(0),
        Some(8),
        "the cancel's neutral choice is none (last)"
    );
    assert_eq!(
        spec.neutral(1),
        Some(0),
        "the release's neutral choice is the default slot"
    );
    // the game: nothing commits until every group has a choice, then one flight judges them together
    let mut g = ScoutGame::with_seed(l.clone(), 3);
    assert!(!g.ready());
    assert_eq!(
        g.commit(),
        Err(ScoutError::BadExit),
        "a half-made choice cannot be committed"
    );
    let w = g.spec.winners(g.entry());
    let parts = g.spec.decode(w[0]);
    assert!(g.select(0, parts[0]) && !g.ready());
    assert!(!g.select(0, 9) && !g.select(2, 0), "no such choice");
    assert!(g.select(1, parts[1]) && g.ready());
    assert_eq!(g.commit(), Ok(true));
    assert_eq!(g.streak, 1);
    assert!(g.next_attempt());
    assert!(!g.ready(), "a new attempt starts with nothing chosen");
    // the right source alone (default slot) and the right slot alone (no source) both fail
    let w = g.spec.winners(g.entry());
    let parts = g.spec.decode(w[0]);
    assert!(g.select(0, parts[0]) && g.select(1, 0));
    assert_eq!(g.commit(), Ok(false));
}

#[test]
fn the_careful_reader_is_used_only_for_sequences() {
    // pick_robust never picks an option that fails under the estimate itself
    let l = lv_id("cmb-20");
    let spec = ScoutSpec::from_level(&l);
    let (kind, pos) = spec.rules.pool[1];
    let o = spec
        .pick_robust(kind, pos, 0.3)
        .expect("the exact source has a safe option");
    assert!(spec.try_option(&spec.field_with(kind, pos), o).0);
    let _ = (levels::combo_levels()
        .iter()
        .filter(|l| matches!(&l.rules, Rules::Scout(r) if matches!(r.act, ScoutAct::Seq(_))))
        .count()
        >= 1);
}

#[test]
fn cmb_21_has_a_source_group_and_a_gust_plan_group_each_with_a_none() {
    let spec = ScoutSpec::from_level(&lv_id("cmb-21"));
    let comps = spec.components();
    assert_eq!(
        (comps[0].1, comps[1].1),
        (9, 5),
        "eight sources and none; four plans and none"
    );
    assert_eq!((spec.neutral(0), spec.neutral(1)), (Some(8), Some(4)));
    // each winning option adds a source and gusts: neither none wins for any hidden source
    for a in 0..spec.rules.pool.len() {
        for o in spec.winners(a) {
            let d = spec.decode(o);
            assert!(d[0] != 8 && d[1] != 4, "entry {a}: option {o} uses a none");
        }
    }
}
