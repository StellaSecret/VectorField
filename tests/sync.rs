//! Synchronize: several probes, one decision per probe (when to release it), and a success condition that is a
//! relationship between them. These tests cover the rules and the game, that the levels are solvable and not trivial
//! in the real simulation, and the *distinction* claims the design rests on: which levels are arithmetic, which are
//! genuine coordination, measured (`validate_sync`) rather than asserted.
use vfield::modes::*;
use vfield::sim::Outcome;
use vfield::validate_sync::*;
use vfield::*;

fn lv(id: &str) -> Level {
    levels::sync_levels().into_iter().find(|l| l.id == id).unwrap_or_else(|| panic!("no level {id}"))
}
fn spec(id: &str) -> SyncSpec {
    SyncSpec::from_level(&lv(id))
}
fn stats_of(id: &str) -> Stats {
    stats(&spec(id))
}

// ---------------------------------------------------------------- the chapter
#[test]
fn synchronize_is_a_chapter_with_five_validated_levels() {
    assert!(Chapter::ALL.contains(&Chapter::Synchronize));
    assert_eq!(Chapter::Synchronize.name(), "Synchronize");
    let ls = levels::chapter(Chapter::Synchronize);
    assert_eq!(ls.iter().map(|l| l.id).collect::<Vec<_>>(), ["syn-1", "syn-2", "syn-3", "syn-4", "syn-5"]);
    for l in &ls {
        assert!(matches!(l.rules, Rules::Synchronize { .. }));
        assert!(l.hints.len() >= 3, "{}", l.id);
        assert!(validate::solvable(l), "{} is not solvable", l.id);
        assert!(validate::oracle_win(l));
        assert!(!validate::summarize(l).is_empty());
    }
}

#[test]
fn adding_synchronize_leaves_the_other_chapters_as_they_were() {
    for (c, n) in [(Chapter::Position, 5), (Chapter::Predict, 5), (Chapter::Tempo, 5), (Chapter::Experiment, 5), (Chapter::Steer, 5), (Chapter::Survey, 5), (Chapter::Shape, 4)] {
        assert_eq!(levels::chapter(c).len(), n, "{c:?}");
    }
    let all = levels::all();
    let mut ids: Vec<_> = all.iter().map(|l| l.id).collect();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), all.len(), "level ids are unique across chapters");
}

#[test]
fn every_condition_relates_two_different_probes_and_every_probe_is_in_one() {
    // The design rule: a probe that is in no condition could complete an unrelated objective on its own, and a
    // condition between a probe and itself is a Tempo problem. Neither demonstrates Synchronize.
    for l in levels::sync_levels() {
        let sp = SyncSpec::from_level(&l);
        assert!(sp.probes() >= 2, "{}", l.id);
        for c in &sp.conds {
            assert_ne!(c.a, c.b, "{}: a condition between a probe and itself", l.id);
            assert!(c.lo < c.hi, "{}", l.id);
        }
        for i in 0..sp.probes() {
            assert!(sp.conds.iter().any(|c| c.a == i || c.b == i), "{}: probe {i} is in no condition", l.id);
        }
        // and the conditions connect the probes into one group (nobody is a separate sub-puzzle)
        let mut group = vec![0usize];
        loop {
            let before = group.len();
            for c in &sp.conds {
                if group.contains(&c.a) && !group.contains(&c.b) {
                    group.push(c.b);
                }
                if group.contains(&c.b) && !group.contains(&c.a) {
                    group.push(c.a);
                }
            }
            if group.len() == before {
                break;
            }
        }
        assert_eq!(group.len(), sp.probes(), "{}: the probes fall into separate groups", l.id);
    }
}

// ---------------------------------------------------------------- times and conditions
#[test]
fn entry_time_is_the_first_moment_inside_the_region() {
    let sp = spec("syn-4");
    let t = sp.fly_one(0, 3.0);
    // the target is where the run ends: the entry time is when the probe got there
    let at = entry_time(&sp.level, &t, 0).expect("it reaches the target");
    assert!((at - (3.0 + t.duration())).abs() <= t.dt + 1e-9);
    // the gate is entered before the target, and the entry time is the first sample inside it
    let g = entry_time(&sp.level, &t, 1);
    if let Some(g) = g {
        assert!(g < at);
        let k = ((g - t.t0) / t.dt).round() as usize;
        let r = &sp.level.regions[1];
        assert!(t.points[k].dist(r.center) <= r.radius && (k == 0 || t.points[k - 1].dist(r.center) > r.radius));
    }
    // a region the path never touches has no entry time
    let far = spec("syn-1");
    let t = far.fly_one(0, 0.0);
    assert!(entry_time(&far.level, &t, 0).is_some());
    let mut lvl = far.level.clone();
    lvl.regions.push(sim::Region::gate(-5.5, 3.5, 0.2));
    assert_eq!(entry_time(&lvl, &t, 1), None);
}

#[test]
fn a_condition_reports_the_offset_and_how_far_outside_it_is() {
    let cond = SyncCond::meet(0, 1, 0, -0.5, 0.5);
    let mk = |a: Option<f64>, b: Option<f64>| SyncCondResult { cond, at_a: a, at_b: b };
    assert!(mk(Some(5.0), Some(5.4)).ok() && mk(Some(5.0), Some(4.6)).ok());
    assert!(mk(Some(5.0), Some(5.5)).ok(), "the edge of the window is inside");
    let late = mk(Some(5.0), Some(6.2));
    assert!(!late.ok());
    assert!((late.offset().unwrap() - 1.2).abs() < 1e-12 && (late.miss().unwrap() - 0.7).abs() < 1e-12, "late by the part outside the window");
    let early = mk(Some(5.0), Some(3.9));
    assert!((early.miss().unwrap() + 0.6).abs() < 1e-12, "early is negative");
    assert_eq!(mk(Some(5.0), Some(5.2)).miss(), Some(0.0));
    // a probe that never arrived cannot satisfy a condition
    assert!(!mk(None, Some(5.0)).ok() && mk(Some(5.0), None).miss().is_none());
    // an ordered condition: b must come 1 to 2.2 s after a
    let order = SyncCondResult { cond: SyncCond::meet(0, 1, 0, 1.0, 2.2), at_a: Some(4.0), at_b: Some(4.5) };
    assert!(!order.ok() && (order.miss().unwrap() + 0.5).abs() < 1e-12);
}

#[test]
fn a_delay_is_the_field_clock_at_release_and_only_a_periodic_field_makes_it_matter_for_the_path() {
    // Static field (level 1): the same path whenever it leaves; only the arrival time moves.
    let s1 = spec("syn-1");
    let (a, b) = (s1.fly_one(0, 0.0), s1.fly_one(0, 7.3));
    assert_eq!(a.points, b.points);
    assert!((b.t0 - 7.3).abs() < 1e-12 && (b.duration() - a.duration()).abs() < 1e-12);
    // Periodic field (level 2): the same probe takes a different path (and time) at a different phase.
    let s2 = spec("syn-2");
    assert_ne!(s2.fly_one(0, 0.0).points, s2.fly_one(0, 6.0).points);
    // and after exactly one period the field is the same, so the path is too
    let (p0, p12) = (s2.fly_one(0, 0.0), s2.fly_one(0, s2.period));
    assert_eq!(p0.points.len(), p12.points.len());
    assert!(p0.points.iter().zip(&p12.points).all(|(x, y)| x.dist(*y) < 1e-6));
}

// ---------------------------------------------------------------- the game
#[test]
fn delays_snap_to_the_step_and_stay_in_range_and_choosing_them_flies_nothing() {
    let mut g = SyncGame::new(lv("syn-2"));
    assert_eq!(g.delays, vec![0.0, 0.0]);
    g.set_delay(0, 3.14159);
    assert!((g.delays[0] - 3.1).abs() < 1e-9);
    g.set_delay(1, -4.0);
    assert_eq!(g.delays[1], 0.0);
    g.set_delay(1, 99.0);
    assert_eq!(g.delays[1], g.spec.max_delay);
    g.nudge(1, -0.1);
    assert!((g.delays[1] - 11.9).abs() < 1e-9);
    assert!(g.flights.is_empty() && !g.won, "only a launch flies the probes");
}

#[test]
fn a_launch_flies_every_probe_at_its_delay_and_records_it() {
    let mut g = SyncGame::new(lv("syn-1"));
    g.set_delay(0, 2.0);
    g.set_delay(1, 5.0);
    let f = g.launch().clone();
    assert_eq!(f.delays, vec![2.0, 5.0]);
    assert_eq!(f.trajs.len(), 2);
    for i in 0..2 {
        assert_eq!(f.trajs[i].points, g.spec.fly_one(i, f.delays[i]).points, "the launch is the real single-probe run");
        assert!((f.trajs[i].t0 - f.delays[i]).abs() < 1e-12);
    }
    // deterministic
    let again = g.launch().clone();
    assert_eq!(f.trajs[0].points, again.trajs[0].points);
    assert_eq!(g.flights.len(), 2);
}

#[test]
fn a_winning_launch_is_kept_and_a_failing_one_says_who_was_early_or_late() {
    let sp = spec("syn-1");
    let w = Table::new(&sp).winners().into_iter().next().expect("a winner");
    let mut g = SyncGame::new(lv("syn-1"));
    for (i, k) in w.iter().enumerate() {
        g.set_delay(i, *k as f64 * SYNC_DELAY_STEP);
    }
    assert!(g.launch().success && g.won);
    // the same pair, with B released 1.5 s later than it needs to be, misses and B is the late one (the winner's own
    // offset can be anywhere in the +-0.4 window, so the miss is between 0.7 and 1.5 s)
    g.set_delay(1, g.delays[1] + 1.5);
    let f = g.launch().clone();
    assert!(!f.success && g.won, "a win is not taken back");
    let c = &f.conds[0];
    assert!(c.miss().unwrap() > 0.6, "B is late: {:?}", c.miss());
    // A released late instead: B is early (negative)
    g.set_delay(1, w[1] as f64 * SYNC_DELAY_STEP);
    g.set_delay(0, g.delays[0] + 1.5);
    assert!(g.launch().conds[0].miss().unwrap() < -0.6);
}

#[test]
fn a_probe_that_misses_its_goal_fails_the_level_whatever_the_timing() {
    let sp = spec("syn-2");
    // find a delay at which probe 0 does not reach the target (outside its window)
    let k = (0..=120).find(|k| sp.fly_one(0, *k as f64 * 0.1).outcome != Outcome::Reached(0)).expect("a bad delay");
    let f = sp.fly(&[k as f64 * 0.1, 0.0]);
    assert!(!f.reached[0] && !f.success);
    assert!(f.conds[0].at_a.is_none() || f.conds[0].at_b.is_none() || !f.conds[0].ok() || !f.reached[0]);
}

// ---------------------------------------------------------------- solvable, not trivial, not a needle
#[test]
fn the_search_table_agrees_with_flying_the_probes_together() {
    // validation evaluates on a cache of single-probe flights; make sure that is the same as the real joint flight
    for id in ["syn-1", "syn-3", "syn-4", "syn-5"] {
        let sp = spec(id);
        let t = Table::new(&sp);
        let n = sp.probes();
        let mut seed = 12345u64;
        let mut next = move |m: usize| {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            ((seed >> 33) as usize) % m
        };
        let mut won = 0;
        for _ in 0..400 {
            let ks: Vec<usize> = (0..n).map(|_| next(t.n_delays)).collect();
            let delays: Vec<f64> = ks.iter().map(|k| *k as f64 * SYNC_DELAY_STEP).collect();
            assert_eq!(t.wins(&ks), sp.fly(&delays).success, "{id} {ks:?}");
        }
        for w in t.winners().iter().step_by(37).take(40) {
            let delays: Vec<f64> = w.iter().map(|k| *k as f64 * SYNC_DELAY_STEP).collect();
            assert!(sp.fly(&delays).success, "{id}: a recorded winner does not win when flown");
            won += 1;
        }
        assert!(won > 0, "{id}");
    }
}

#[test]
fn blind_timing_is_neither_trivial_nor_a_needle() {
    // (lowest, highest) share of random delay combinations that win. Level 1 has one narrow band on a 12 s range
    // (6%); the others join two or three such constraints, so far fewer random combinations win; none is zero or a
    // single point (docs/11-synchronize.md, section 4).
    for (id, lo, hi) in [("syn-1", 0.03, 0.12), ("syn-2", 0.004, 0.05), ("syn-3", 0.004, 0.05), ("syn-4", 0.003, 0.03), ("syn-5", 0.0005, 0.01)] {
        let s = stats_of(id);
        assert!((lo..=hi).contains(&s.blind()), "{id}: blind {:.4}", s.blind());
        assert!(s.winners >= 20, "{id}: only {} winning combinations", s.winners);
    }
}

#[test]
fn the_synchronization_window_is_a_real_constraint() {
    // Timing sensitivity: with the others fixed, a probe's release can move only a little before a winning
    // combination stops winning (the width is within a couple of window widths, not the whole range).
    for (id, max_width) in [("syn-1", 1.2), ("syn-2", 1.6), ("syn-3", 1.8), ("syn-4", 2.0), ("syn-5", 2.5)] {
        let s = stats_of(id);
        assert!(s.width > 0.0 && s.width <= max_width, "{id}: width {} s", s.width);
    }
    // and moving one probe of a winning pair by more than the allowed gap, in steps, flips the result both ways
    let sp = spec("syn-1");
    let w = Table::new(&sp).winners().into_iter().next().unwrap();
    let d = |k: usize| k as f64 * SYNC_DELAY_STEP;
    assert!(sp.fly(&[d(w[0]), d(w[1])]).success);
    assert!(!sp.fly(&[d(w[0]), d(w[1]) + 1.0]).success && !sp.fly(&[d(w[0]), (d(w[1]) - 1.0).max(0.0)]).success || w[1] < 10);
}

// ---------------------------------------------------------------- distinct from Tempo?
#[test]
fn level_one_is_arithmetic_and_says_so() {
    // The honest result for the plain rendezvous: in a static field success depends only on the difference of the two
    // delays (shifting both changes nothing) and the first probe can leave at any time. It is an on-ramp, not the
    // mechanic: it does not by itself separate Synchronize from Tempo.
    let s = stats_of("syn-1");
    assert_eq!(s.shift_break_1s, 0.0);
    assert_eq!(s.shift_break_2s, 0.0);
    // every release of the first probe has a partner, except one so late that the second probe's delay would pass the
    // 12 s limit (B has to leave 2.2 s after A): 9.8 / 12 = 82%
    assert!(s.anchor > 0.8, "almost every release of the first probe has a partner: {}", s.anchor);
    assert!(s.alone.iter().all(|a| *a > 0.99), "each probe reaches the target at any time");
    assert!((s.tempo_only - s.blind()).abs() < 1e-9, "per-probe timing tells a Tempo player nothing here");
    // the relationship is a single number: the winners are exactly the pairs with the right difference
    let sp = spec("syn-1");
    let (ta, tb) = (sp.fly_one(0, 0.0).duration(), sp.fly_one(1, 0.0).duration());
    for w in Table::new(&sp).winners().iter().step_by(11) {
        let diff = (w[1] as f64 - w[0] as f64) * SYNC_DELAY_STEP + tb - ta;
        assert!(diff.abs() <= 0.4 + 0.11, "{diff}");
    }
}

#[test]
fn levels_two_to_five_depend_on_when_each_probe_leaves_not_only_on_the_difference() {
    // In a periodic field the same relative delay wins at one time and loses at another: success depends on the
    // absolute phase of every probe, so the problem is not a subtraction.
    for (id, min_break) in [("syn-2", 0.3), ("syn-3", 0.4), ("syn-4", 0.7), ("syn-5", 0.9)] {
        let s = stats_of(id);
        assert!(s.shift_break_1s >= min_break, "{id}: only {:.2} of the winners stop winning when everything leaves 1 s later", s.shift_break_1s);
        assert!(s.shift_break_2s >= s.shift_break_1s * 0.9, "{id}");
    }
}

#[test]
fn levels_two_and_three_cannot_be_solved_by_solving_each_probe_on_its_own() {
    // The Tempo-only test. A player who finds, for each probe separately, a release at which it reaches the target
    // (what Tempo teaches), and ignores the other probe, gets the relationship right far less often than a player who
    // looks at both; and the probes are limited to windows by their own pulsing push, so most releases miss entirely.
    for id in ["syn-2", "syn-3"] {
        let s = stats_of(id);
        assert!(s.alone.iter().all(|a| *a < 0.4), "{id}: each probe reaches the target at only some times: {:?}", s.alone);
        assert!(s.tempo_only < 0.35, "{id}: independent good releases win {:.2}", s.tempo_only);
        assert!(s.tempo_only > 4.0 * s.blind(), "{id}: but they are much better than blind guessing");
        // and a first probe is not free: some of its good releases have no partner at all (16% in level 2, 49% in level 3)
        assert!(s.anchor < 0.9, "{id}: anchor {}", s.anchor);
    }
}

#[test]
fn levels_four_and_five_need_the_same_delays_to_satisfy_conditions_that_pull_against_each_other() {
    // Each condition must matter: with any one of them removed the level is much easier. (An earlier version of level 4
    // asked the probes to meet at the gate and again at the target; the second condition then removed almost nothing,
    // 151 of 14,641 combinations against 134, because two probes at one place at one time share a future. See
    // docs/11-synchronize.md.)
    for id in ["syn-4", "syn-5"] {
        let sp = spec(id);
        assert_eq!(sp.conds.len(), 2);
        let s = stats(&sp);
        assert!(s.anchor < 0.5, "{id}: most releases of the first probe have no partner ({})", s.anchor);
        for keep in 0..2 {
            let mut one = sp.clone();
            one.conds = vec![sp.conds[keep]];
            let o = stats(&one);
            assert!(o.blind() > 1.5 * s.blind(), "{id}: condition {keep} alone {:.4}, both {:.4}: the other condition adds too little", o.blind(), s.blind());
        }
    }
}

#[test]
fn coordination_beats_independent_choices_and_both_beat_guessing_in_every_periodic_level() {
    for id in ["syn-2", "syn-3"] {
        let s = stats_of(id);
        assert!(s.blind() < s.tempo_only && s.tempo_only < s.anchor, "{id}: {:.3} < {:.3} < {:.3}", s.blind(), s.tempo_only, s.anchor);
    }
    // the player who looks at the whole timeline (knows each probe's arrival times) has a partner for most releases
    for id in ["syn-2", "syn-3"] {
        assert!(stats_of(id).anchor >= 0.4, "{id}: coordination is possible without a needle search");
    }
}

#[test]
fn levels_differ_in_kind_not_only_in_tolerance() {
    // static / windows (same place, same time) / windows (one after the other) / two places / three probes
    let kinds: Vec<(usize, usize, bool)> = levels::sync_levels()
        .iter()
        .map(|l| {
            let sp = SyncSpec::from_level(l);
            let ordered = sp.conds.iter().any(|c| c.lo > 0.0);
            let regions: std::collections::HashSet<usize> = sp.conds.iter().flat_map(|c| [c.ra, c.rb]).collect();
            (sp.probes(), regions.len(), ordered)
        })
        .collect();
    assert_eq!(kinds[0].0, 2);
    assert!(kinds[2].2 && !kinds[1].2, "level 3 asks for an order, level 2 for simultaneity");
    assert!(kinds[3].1 == 2 && kinds[1].1 == 1, "level 4 uses two places");
    assert!(kinds[4].0 == 3, "level 5 has three probes");
    // the static level is the only one in which a travel time does not depend on the release
    for l in levels::sync_levels() {
        let sp = SyncSpec::from_level(&l);
        let durs: Vec<f64> = (0..12).map(|k| sp.fly_one(0, k as f64).duration()).collect();
        let constant = durs.iter().all(|d| (d - durs[0]).abs() < 1e-9);
        assert_eq!(constant, l.id == "syn-1", "{}: durations {durs:?}", l.id);
    }
}

// ---------------------------------------------------------------- robustness
fn perturbed(l: &Level, start_shift: V2, widen: f64) -> Level {
    let mut m = l.clone();
    if let Rules::Synchronize { starts, conds, .. } = &mut m.rules {
        for s in starts.iter_mut() {
            *s = *s + start_shift;
        }
        for c in conds.iter_mut() {
            c.lo -= widen;
            c.hi += widen;
        }
    }
    m
}

#[test]
fn levels_survive_moved_starts_and_changed_windows() {
    for l in levels::sync_levels() {
        // windows 0.2 s narrower and wider
        for widen in [-0.2, 0.2] {
            let m = perturbed(&l, V2::ZERO, widen);
            let sp = SyncSpec::from_level(&m);
            if sp.conds.iter().any(|c| c.lo >= c.hi) {
                continue;
            }
            let s = stats(&sp);
            assert!(s.winners > 0, "{} with windows changed by {widen}: no solution", l.id);
            assert!(s.blind() < 0.2, "{} with windows changed by {widen}: blind {:.3}", l.id, s.blind());
        }
        // starts moved by 0.25 along the stream or across it
        for d in [v2(0.25, 0.0), v2(0.0, 0.25), v2(-0.25, 0.0), v2(0.0, -0.25)] {
            let s = stats(&SyncSpec::from_level(&perturbed(&l, d, 0.0)));
            assert!(s.winners > 0, "{} with starts moved by {d:?}: no solution", l.id);
        }
    }
}

// ---------------------------------------------------------------- the front end
#[test]
fn the_front_end_flies_the_probes_only_when_the_launch_button_is_used() {
    // The timeline shows arrival times only of flights that already happened. Nothing in the Synchronize frame may fly a
    // probe to show where it would arrive.
    let app = include_str!("../src/app.rs");
    let start = app.find("fn sync_frame").expect("sync_frame");
    let end = app[start..].find("#[cfg(test)]").map(|i| start + i).unwrap_or(app.len());
    let body = &app[start..end];
    for forbidden in [".fly(", ".fly_one(", ".fly_charted(", "simulate(", ".run(&", "level.run"] {
        assert!(!body.contains(forbidden), "sync_frame calls {forbidden}: that would show an arrival before launch");
    }
    assert_eq!(body.matches(".launch()").count(), 1);
}

#[test]
fn the_panel_text_states_early_and_late_but_not_the_answer() {
    // the verdict names a direction and an amount for the condition that failed, never a delay to choose
    let app = include_str!("../src/app.rs");
    assert!(app.contains("s too late") && app.contains("s too early"));
    let start = app.find("fn sync_cond_text").unwrap();
    let end = app[start..].find("fn sync_frame").map(|i| start + i).unwrap();
    assert!(!app[start..end].contains("leave at") && !app[start..end].contains("release at"));
}

#[test]
fn no_level_is_won_by_launching_with_the_default_delays() {
    // Found by looking at the running game: level 1 used to be won by pressing Launch immediately, because the two
    // travel times (4.0 s and 3.7 s) differed by less than the window. Every level must need a decision.
    for l in levels::sync_levels() {
        let sp = SyncSpec::from_level(&l);
        assert!(!sp.fly(&vec![0.0; sp.probes()]).success, "{}: the untouched defaults win", l.id);
        // nor is there any solution within 1.5 s of the defaults (level 1's nearest is 1.8 s: B must leave 2.2 s after A)
        let t = Table::new(&sp);
        let nearest = t.winners().iter().map(|k| *k.iter().max().unwrap() as f64 * SYNC_DELAY_STEP).fold(f64::MAX, f64::min);
        assert!(nearest >= 1.5, "{}: a solution has every delay within {nearest} s of the defaults", l.id);
        // and not by luck of a one-step nudge of a single probe either
        for i in 0..sp.probes() {
            let mut d = vec![0.0; sp.probes()];
            d[i] = SYNC_DELAY_STEP;
            assert!(!sp.fly(&d).success, "{}: nudging probe {i} by one step wins", l.id);
        }
    }
}
