# 07. Combinations: composing the mechanics

Source: `src/levels.rs::combo_levels`. The combination levels add **no new rules**: each one reuses an existing mechanic's code and changes only *data* (a region, a reveal setting, an extra control). The chapter unlocks after the first level of Position, Predict, Tempo and Experiment is complete (`U` unlocks everything), and its levels are played in order. A level that uses Shape or Steer also needs the first level of that mechanic (`levels::combo_requires`: `cmb-7` and `cmb-8` need `shp-1`; `cmb-9` needs `shp-1` and `syn-1`; `cmb-11` and `cmb-12` need `ste-1`; `cmb-13` needs both).

`cmb-1` to `cmb-4` each add one constraint to one mechanic. `cmb-5` and `cmb-6` combine two decisions. `cmb-7`, `cmb-8`, `cmb-9`, `cmb-10`, `cmb-11`, `cmb-12`, `cmb-13`, `cmb-22`, `cmb-23` and `cmb-24` are held to an explicit bar for that (see "The bar for a two-decision level" below). `cmb-16` is the first *scout, then act* level and is held to a variant of it (see "The scout-then-call level" below). `cmb-18` is a second scout-then-call level (see "The second scout-then-call level" below). `cmb-14`, `cmb-15` and `cmb-17` are scout-then-act levels with a categorical act (see "Scout, then act: release, cancel, steer" below). `cmb-10` Route and Rendezvous has a categorical start choice (see "Route and Rendezvous: start slots" below). Every id of the plan is now built.

The architecture separates three kinds of things, and a combination is a choice from each:

| Kind | What it changes | Examples in the engine |
|---|---|---|
| **Fundamental mechanic** | what the player decides, and when | Position, Predict, Tempo, Experiment, Steer |
| **Constraint** | what counts as success | target, exits, hazard, gate, several probes, partial reveal |
| **Field property** | the mathematics of the flow | uniform, vortex, radial, saddle, periodic, orbiting |

Steer is combined in `cmb-11`, `cmb-12` and `cmb-13`; Synchronize is combined in `cmb-9`; Survey and Predict are combined in `cmb-16`, the first *scout, then act* level, and again in `cmb-18`.

## The first six levels

| Level | Mechanic | Constraint or extra control added | What it does to the base level | Measured |
|---|---|---|---|---|
| `cmb-1` Through the Gate | Position | a **gate** (r = 0.4 at (0, 2)); success needs `reached_goal`, i.e. the gate must be visited before the target | removes most starts that won `pos-2` | win area 7.1% (19.9% without the gate) |
| `cmb-2` Early Clue | Predict | `Reveal::Partial(2.5)`: the first 2.5 s of the true path are drawn before the commit | adds evidence to extrapolate from; the field is `pre-4`'s | pool 89, answers B C D |
| `cmb-3` Dodge | Tempo | a **hazard** (r = 0.7 at (0.5, 0.8)) on the path of the early launches | removes the early part of the good window | success phases 18.8% (31.7% without) |
| `cmb-4` Three Exits | Experiment | a Threshold challenge between two **specific** exits among four (C to D) | the switch must be found among several exits | truth 2.7352, tolerance 0.15 |
| `cmb-5` Right Place, Right Time | Position | the **launch phase** is also a control (period 12 s, `Rules::Position{period: Some(12)}`) over a time-dependent orbiting-vortex field | place and time interact: the winning place moves with the phase | 4 of 12 phases admit any win (33%); mean win area 13.5% |
| `cmb-6` Shared Dial | Position | **two probes with their own zones and targets** sharing one **intensity** (`Rules::Position{intensity: Some((0, 3))}`) | both probes must win under the same dial setting | 5 of 13 dial settings admit a win for both (38%); mean per-slot win area 13.2% |

## The bar for a two-decision level

A level combines two mechanics only if all of this holds, and `validate_combo` measures each point on the real simulation (`src/validate_combo.rs`, `tests/combo.rs`, [COMBO_VALIDATION.md](COMBO_VALIDATION.md)):

1. **Both decisions are necessary (ablation).** With the plan left empty (no source, no gusts) no control setting wins. At the control's default (phase 0, the middle of the dial, the default start, or no source at all) no plan wins, and none wins anywhere within the *dead band* of the default: 1.5 s of phase, 15% of the dial range, 1.0 of distance for a start point, or every probe released within 1.5 s of its default delay (Synchronize). A placed source has no distance from \"no source\", so its dead band is just the empty default. Removing either decision makes the level unwinnable.
2. **The decisions do not substitute for each other.** If one control can do the other's job, it is not a combination. This is measured by the ablation above and by *coupling*: the plans that win at one control setting differ from those at another (mean Jaccard distance between the win sets; 0 would mean the two decisions are independent).
3. **Reading the field beats guessing** (measured for every level except `cmb-9`: the reader model needs a Shape spec, and the Synchronize report says "not measured" rather than claim it). A player who scans the control with free empty launches and takes the setting that gets closest to the target finds a win; for the Shape levels, a reader choosing the source (the same reader model as Shape) wins far more often than a blind draw.
4. **Robust.** The ablation still holds when the target and obstacle are moved by 0.3 in each direction.
5. **One new idea** per level.

## The two-decision levels

| Level | Mechanics | The two decisions | Measured |
|---|---|---|---|
| `cmb-7` Source on the Tide | Shape + Tempo | the **launch phase** over a sideways pulse (period 12 s), and **where to put one source**. Sources are steady; only the background pulses | 9 of 24 phases admit a win; 0.60% of (phase, site, kind) triples win; coupling 0.83; 4/4 moved geometries hold |
| `cmb-8` Compensate | Shape + dial | the **background intensity** (0 to 3) of a strong vortex, and **where to put one source** downstream of a large obstacle | 6 of 25 dial settings admit a win (0.38 to 1.00 on a 0.125 grid; default 1.5); 4.39% of pairs win; coupling 0.61; 4/4 hold |
| `cmb-9` Make Them Meet | Shape + Synchronize | **one source** (repeller or attractor, in the upper part of the field) and **the two release delays**. Each probe reaches the target only in certain release windows, and with no source their arrival times never come within 0.6 s of each other | 113 of 481 candidate sources admit a win (23.5%); up to 159 delay pairs win at one source; 0.18% of (source, delay pair) triples win; coupling 0.85; nearest solution 4.6 s from the default delays; 4/4 hold |
| `cmb-11` Catch the Gust | Steer + Tempo | the **start phase** over a sideways pulse, and **two gusts** among 8 beats | 10 of 24 phases admit a win; 4.09% of runs win; coupling 0.71; 4/4 hold |
| `cmb-12` Fewer Gusts | Steer + Position | the **start point** on a vertical line (25 points, default at height 0), and **two gusts** among 8 beats; an obstacle above the lane punishes a start that is too high | 5 of 25 start points admit a win (heights 1.25 to 2.25); every winning plan uses both gusts; 0.10% of runs win; coupling 0.94; 4/4 hold |
| `cmb-13` Pave the Way | Steer + Shape | **one source** (repeller or attractor, placed before the first beat in a 6.5 x 6 area) and **a single gust** among 8 beats; an obstacle sits in the lower lane | 120 of 905 candidate sources (kind and site) admit a win, 13%; 1.43% of (source, gust plan) pairs win; coupling 0.79; 4/4 hold |

In all six the empty plan wins at no setting, and nothing wins at the default or within its dead band (the nearest winning setting is 4.0 s of phase away in `cmb-7`, 3.5 s in `cmb-11`, 0.5 of the dial in `cmb-8` and 1.25 in height from the default start in `cmb-12`; in `cmb-13` no source wins without a gust and no gust wins without a source; in `cmb-9` no source wins at the default delays and no solution has both delays within 4.6 s of them).

## The scout-then-call level (`cmb-16`)

`cmb-16` Scout, Then Call It (Survey + Predict) is a two-stage level, so it is held to a variant of the bar. A hidden source, drawn at random from a pool of three each attempt (never the one just used), bends every flight. In stage one the player launches up to three scouts from a strip on the left and reads the push each felt (measured as Survey measures it: observed velocity minus the visible field's). In stage two the player calls which of four exits (A to D) the main probe will leave through; the call is committed before the probe flies and its true flight is then shown. Three correct calls in a row complete the level.

What `validate_scout` measures, on the real simulation (`tests/scout.rs`, [COMBO_VALIDATION.md](COMBO_VALIDATION.md)):

| Property | `cmb-16` |
|---|---|
| The three hidden sources send the probe to three different exits (B, A, C), so no fixed call can be right twice in a row | yes; the best fixed call is right for 1 of 3 |
| The visible arrows alone (what a player who never scouts would simulate) name the right exit | for 1 of 3 sources only, so that call cannot pass either |
| A reader who launches three scouts at fixed starts, triangulates the source from the push, and simulates the probe with that estimate | calls all 3 correctly; the scouts felt the source in all 3 |
| Tolerance of that call to an error in the estimated position | 16 of 16 directions at an error of 0.3 for every source (the bar is 80%); at 0.5, 12 to 16 of 16; at 0.8, 5 to 16 of 16 |
| The same, with the exit gates moved by 0.3 in all four directions | holds in 4 of 4 |

Why it works where `cmb-14` and `cmb-15` did not: the act is a *categorical* choice (one of four wide gates), not a placement or a start height that must be exact. Wide gates give the tolerance a person needs, and giving each hidden source its own exit removes the shared "universal answer" that defeated the continuous acts. In one run of 30,000 random designs, 288 passed the base requirements and 23 (about 1 in 1,300) also held with the gates moved in all four directions; the parameters were sampled at two decimals, so what passed is exactly what ships (a first attempt with unrounded numbers changed result when rounded). The first draft of the order used the pool in a fixed cycle, which a player could memorise on a replay; the hidden source is now drawn at random with no immediate repeat (seeded from the clock in the game), and a test shows a memorised order fits few other plays and that no fixed call completes the level over 40 seeds of 60 attempts.

Limits, stated plainly. The reader model assumes the source's strength and profile are known and triangulates exactly from the measured push; a person reading arrows by eye is less precise, which is what the tolerance rows are for, but no one has played it yet. The pool has three sources and four exits, so after a few attempts a player may recognise which of the three it is from the push pattern and not triangulate; that still needs the scouts every attempt (the source is not known until scouted), and it is the reason the level asks for three correct calls in a row.

## The second scout-then-call level (`cmb-18`)

`cmb-18` Scout Through the Swirl (Survey + Predict) uses the same rules, game code and validator as `cmb-16` and changes only data. The visible field is a stream plus a clockwise swirl (`vortex(-0.81, -0.8, -2.54)`), so every flight, scouts and main probe alike, is bent by the swirl as well as by the hidden source. The push arrows already subtract the visible field, so the evidence is the same kind as in `cmb-16`; what is new is that the final prediction must be made through a curved flow, and the visible-only path visibly bends (a test pins a y range above 0.5). The pool again has three hidden sources (an attractor and two repellers) that send the probe to exits D, C and B; the visible arrows alone name C.

How it was found: a random search (not part of this repository) samples a background (stream and swirl), three hidden sources and their kinds at two decimals, and keeps a design only if the `cmb-16` bar holds (distinct exits, no fixed or visible-only call can pass, the reader calls all three, tolerance at least 90%, the ablation holds with the gates moved in all four directions). 118 of 40,000 samples passed (about 1 in 340; `cmb-16`'s plain stream needed about 1 in 1,300), and the one shipped was chosen for a strong swirl, mixed kinds and 100% tolerance. A search with four hidden sources, one per exit, found none in 160,000 samples, so the pool stays at three.

Limits are those of `cmb-16`: the reader model triangulates exactly and no person has played it. The swirl makes a by-eye prediction harder than in `cmb-16`, which is intended, but the tolerance row is the only evidence that it stays achievable.

## Scout, then act: release, cancel, steer (`cmb-14`, `cmb-15`, `cmb-17`)

These three were first tried with a *continuous* act and failed (see "Not built yet" for why). They are built here with the lesson of `cmb-16`: the act that ends an attempt is a choice among a few labelled, wide options. `Rules::Scout` carries a `ScoutAct`, and one engine (`ScoutSpec::try_option`) flies the real simulation for every act:

| level | pair | the choice | success means |
|---|---|---|---|
| `cmb-14` Scout, Then Release | Survey + Position | one of 7 release slots (numbered diamonds at x = -5) | the main probe, released there, reaches the target |
| `cmb-15` Cancel It | Survey + Shape | one of 8 sources to add: a repeller or an attractor at each of 4 places | the probe reaches the target with that source added |
| `cmb-17` Scout, Then Steer | Survey + Steer | one of 4 fixed gust plans (at most two gusts, six one-second beats) | the probe steered by that plan reaches the target |

The hidden source is drawn at random from a pool of three each attempt (never the one just used), as in `cmb-16`, and three correct choices in a row complete the level.

**The bar.** The `cmb-16` bar, restated for options (`validate_scout::report`): every hidden source has at least one winning option and no option wins for two of them, so no fixed choice can complete the level (`best fixed` is 1 of 3, the streak is 3); the choice that the visible arrows alone suggest (the first option that works with nothing hidden) is right for at most one source; a reader that triangulates the source from three scouts and picks the first option that works in its estimated field is right for all three; and that still holds when the estimate is off by 0.3 in eight directions (tolerance at least 90%) and with the target moved by 0.3 in all four directions.

| level | tolerance | robust to moved target | visible-only right | best fixed right |
|---|---|---|---|---|
| `cmb-14` | 92% | 4/4 | 1/3 | 1/3 |
| `cmb-15` | 92% | 4/4 | 1/3 | 1/3 |
| `cmb-17` | 100% | 4/4 | 1/3 | 1/3 |

**How they were found** (a random search that is not part of this repository; parameters sampled at two decimals so that what passed is what ships):
- `cmb-14`: a stream and an optional swirl, a target, 5 to 8 slots, three hidden sources. With 5 or 6 slots, 63 and 135 designs per 40,000 had disjoint winning slots but none survived the moved target, because each source's window was a single slot. With 7 to 8 narrower slots (0.7 to 0.8 apart), 1 design in 100,000 passed everything.
- `cmb-15`: four places along the lane, a hidden source at three of them, the cancel options at all four. About 1 in 60,000 passed. The opposite kind at the same place cancels exactly; the same kind doubles the push (a test checks that).
- `cmb-17`: for each hidden source the search enumerates every plan of at most two gusts (1,009 plans), keeps the plan that wins for that source alone and survives the source being moved by 0.3, and adds a decoy that wins for none. 1 design in about 2,000 passed. The source strength for this level is 3.0, not the default 2.01.

**Limits.** `cmb-17` is a choice of prepared plans, not closed-loop steering: a live steer would show the probe's own flight and make the scouting unnecessary, so the Steer decision here is only which gust plan suits the source. A plan that ends before the target is reached fails, and the probe gets six beats. The reader model triangulates exactly and picks the first working option; nobody has played these levels, and the game screens for the new acts have been compiled but not run by a person.

## The front page with 27 combination levels

One progress pip per level stops fitting a card above 8 levels, so the Combinations card on the menu shows a bar with one tick per level and the count (the other cards keep their pips). The select screen lists the combination levels under five headings, in two columns so that every row stays readable: *First combinations* (`cmb-1` to `cmb-6`) and *Two controls before the run* (`cmb-7` to `cmb-10`, `cmb-22`, `cmb-23`, `cmb-24`, `cmb-28`) on the left, *Set up, then steer* (`cmb-11` to `cmb-13`), *Learn, then act* (`cmb-14` to `cmb-19` and `cmb-29`) and *Three decisions* (`cmb-20`, `cmb-25`) on the right (`levels::combo_family`). Titles and objectives are cut to the row with "..." when they are too long, so the full objective is shown once the level opens. Each column's rows shrink to fit above the Back button (44 px at most, 36 px at least, headings 26 px; below 40 px the objective line is dropped), so the left column holds up to 14 rows and the right one up to 13: the left one has 14 now and the right one 13, so both columns are full and the next level needs a third column or scrolling. (`cmb-22` was moved to the left column's family for that reason: it has a dial set before the run, like `cmb-23` and `cmb-24`.) A test checks that the families are contiguous and that both columns fit at 36 px. The layout has been compiled but not looked at on a real screen.

## Survey + Tempo: choose the launch phase (`cmb-19`)

`cmb-19` Scout, Then Time It adds two acts to the scout engine: `Phase` (a launch phase chosen from a list, as in Tempo) and `Dial` (an intensity setting, as in Experiment; no level uses it, see below). The background is a stream plus a sideways push that pulses with a twelve-second period, and the attempt ends with a choice among six launch phases (0, 2 ... 10 s; phase 0 is the default). The three hidden sources are attractors, and their winning phases are 6 s, 2 s and 4 s. Phase 0 wins for none of them (`ScoutReport::neutral_wins`, new: for each control of the final choice, the number of hidden sources that some winning option beats with that control left neutral; a level needs all of its controls when every entry is 0).

| measure | `cmb-19` |
|---|---|
| winning phase per hidden source | 6 s, 2 s, 4 s |
| best fixed phase right for | 1 of 3 |
| visible-only choice right for | 1 of 3 |
| reader right / tolerance | 3 of 3 / 100% |
| robust to the target moved by 0.3 | 4/4 |

Found by a random search for launch phases (not part of this repository): 1 design in 40,000 passed everything with seed 7, none with seed 13, so the rate is about 1 in 80,000. Weakness: all three sources are attractors, so reading the source's kind never changes the answer.

**The dial act is not used.** Survey + Experiment (choose a dial setting) found 2 designs with disjoint winners in 40,000 and none that passed. The dial only scales the tunable background, so it cannot bend a probe either way to compensate sources on opposite sides, and the best dial is nearly the same for every source. The act exists in the engine (`ScoutAct::Dial`) but no level ships with it.

## Three decisions: `cmb-20` Cancel and Launch (Survey + Shape + Position)

`ScoutAct::Seq` makes one flight judge several choices together; the options are every combination (the first choice is the most significant), and a `Cancel` or `Plan` inside a sequence gains a last option "none". The screen shows one group of buttons per choice and a "Commit all" button (Enter). `cmb-20` is the first level with three mechanics: Survey (scout the hidden source), Shape (add a cancelling source, or none) and Position (one of eight release slots, slot 1 being the default far off the lane). Its prerequisites are `pos-1`, `shp-1` and `sur-1`.

The bar is the scout bar plus one more measure, `ScoutReport::neutral_wins`: for each choice that has a neutral option (no source; the default slot, phase or dial), the number of hidden sources that some winning option beats with that choice left neutral. Every entry is 0 for `cmb-20`, so each control is needed: no cancel never wins, the default slot never wins, and the visible arrows alone suggest a choice that is right for 1 of 3 hidden sources at best. The scouting is needed because no fixed combination wins for more than 1 of 3 (best fixed 1/3), and the reader finds all three (3/3), with tolerance 92% and 4/4 robustness to the target moved by 0.3.

**How it is built, and what that costs.** Random search over joint choices failed (see below), so `cmb-20` was constructed so that the choices do not fight over one narrow window: the hidden source is always on one of four places and the opposite kind at the same place cancels it exactly (the tolerant `cmb-15` structure), after which the flight is the visible one, and the release slot is chosen among slots that reach the target through the visible swirl. The right slot is therefore the same for every hidden source; only the cancel depends on the scouting. The three decisions are all necessary and none substitutes for another, but the slot is not something the scouts tell you: it is a property of the visible field that is learned once. A level where the hidden source also moves the right slot (a true interaction) would be harder to make tolerant, and none was found.

Found by a random search for a cancel and a slot (not part of this repository): 2 designs in 2,000 samples passed everything (the shipped one has the wider window of three slots; the other had tolerance 96% and two slots). That is a much higher rate than random search over joint windows.

## Three decisions with the dial: `cmb-25` Cancel and Dial (Survey + Shape + Experiment)

Built like `cmb-20` and `cmb-21`: the hidden source is on one of four places and the opposite kind there cancels it exactly, after which the probe flies the visible field, and the second choice is one of six dial settings (1.0, 0.5, 0.75, 1.25, 1.5, 2.0; the first is the default) that scale the field's tunable drift; the hidden source and the one the player adds ignore the dial. Requires `exp-1`, `shp-1` and `sur-1`. Bar: reader 3/3, tolerance 96%, best fixed 1/3, and the choice the visible arrows alone suggest is right for 0 of 3 hidden sources; robust 4/4; `neutral_wins` 0 for both choices (no cancel and the default dial each win for no hidden source). The winning dial settings are 0.5 and 0.75 for every hidden source (for the first one, 1.25 also wins, with a different cancelling source), as the right slot and gust plan were in `cmb-20` and `cmb-21`: after an exact cancel the flight is the visible one, so the right dial is learned once from the visible field, and only the cancel depends on the scouting. A search for it found 2 designs that passed everything before it was stopped (its total sample count was not recorded; the other one had only attractors and one winning option per hidden source, so the shipped one is the one with mixed kinds and wider windows). One tunable layer is a vortex of strength 0.13, which hardly bends the flow, so in practice the dial scales a uniform drift. The dial is not scouted: nothing the scouts feel says which setting works.

**What did not work.** Random search over two stages whose winning windows interact found no design:
- Survey → Position → Tempo (8 slots × 6 phases, default slot far off the lane): 0 in about 120,000 samples; 153 to 162 per 40,000 had disjoint winners and 11 to 13 needed all three controls and were read correctly.
- Survey → Shape → Steer (all 48 single-gust plans enumerated, four plans chosen for exclusive winners): 0 in 40,000.

Those candidates fail on tolerance, not on the moved target: with the reader's estimate of the source off by 0.3, the winning (slot, phase) pair changes, because the winning combinations are narrow in the two-dimensional grid of options (tolerances 0.46 to 0.67 where 0.8 is required). A careful reader (`ScoutSpec::pick_robust`: the option that wins under the most estimates within 0.3 of its own) is used for sequences and did not rescue them. It is a different reader model from the one used for single choices, which pick the first option that works for the estimate.

## Three decisions again: `cmb-21` Cancel and Steer (Survey + Shape + Steer)

`cmb-21` is built the same way as `cmb-20`, with a gust plan where `cmb-20` has a release slot. The hidden source is on one of four places (a repeller or an attractor); the opposite kind there cancels it exactly, after which the probe flies the visible field (a stream plus a swirl), which leaves it off the target; one of four fixed single-gust plans (or none) then brings it onto the target. Requires `shp-1`, `ste-1` and `sur-1`. Bar: reader 3/3, tolerance 96%, best fixed 1/3, visible-only 1/3, robust 4/4, and `neutral_wins` is 0 for both choices (no source added and no gust each win for no hidden source). Found by a random search for a cancel and a gust plan (not part of this repository): the plans are chosen from the single-gust plans that fix the visible-only flight (two that do, two that do not), and 1 design in 11,600 samples passed everything (3,000 samples found none at 0.3, so the rate is low even by construction). The same limit as `cmb-20` applies: the right gust plan is the same for every hidden source, because after an exact cancel the flight is the visible one.

## Dial levels: Experiment with Steer, Synchronize and Tempo (`cmb-22`, `cmb-23`, `cmb-24`)

The dial (the intensity of the tunable layers of the field, as in Experiment) becomes a pre-run control in three more mechanics, with the categorical shape of the start slots of `cmb-10`. `cmb-22` and `cmb-23` offer thirteen settings (`dial13`: the default 1.0 first, then 0.0 to 2.4 in steps of 0.2 without 1.0); `cmb-24` still has five (1.0, 0, 0.5, 1.5, 2.0). Hidden or placed sources and the level's fixed layers ignore the dial. `Rules::Steer`, `Rules::Synchronize` and `Rules::Tempo` gained `dial: Option<Vec<f64>>`; each spec has `with_dial`, each game has `set_dial` (Steer only before the first beat), the screens have numbered dial buttons in rows, and the combination validator has the axis `Axis::Slots(n, "dial setting")` (setting 0 is the default) with the measures of the start slots: none_wins (the empty plan: no gust, phase 0 or no delay), default_wins (a win at the default setting) and dead_wins (a win within 1.5 s of phase 0 or with near-simultaneous release at any setting) must all be 0, so the dial and the plan are both needed.

| level | pair | the plan after the dial | settings that admit a win | most winning plans at one setting | coupling | informed / blind | robust to moved target | meets the protocol |
|---|---|---|---|---|---|---|---|---|
| `cmb-22` Set the Dial, Then Steer | Steer + Experiment | one gust (eight beats) | 3 of 13 | 15 | 0.84 | 4.3 | 4/4 | yes |
| `cmb-23` Dial and Rendezvous | Synchronize + Experiment | both release delays | 3 of 13 | 214 | 0.78 | not measured | 4/4 | yes (measured part) |
| `cmb-24` Dial and Time It | Tempo + Experiment | the launch phase (240 scanned) | 2 of 5 | 71 | 0.51 | not measured | 4/4 | **no** |

The first versions of `cmb-22` and `cmb-23` had five settings and failed the protocol (below); they were redesigned with thirteen, searching with the protocol itself as the filter (a design had to pass `ProtocolReport::failures()`, the ablation and the four moved-target checks): 25 designs in 8,000 samples for Steer (with budget 1), and 3 in about 2,300 for Synchronize.

**Tempo + dial is not redesigned and does not meet the protocol.** Two searches with the protocol as the filter found nothing: 6,000 samples with thirteen settings from 0 to 2.4 (91 passed the ablation, none the protocol), and 8,000 with thirteen settings from 0.4 to 1.6 in steps of 0.1 (130 passed the ablation, none the protocol). The measures that failed were a winning band touching an end of the dial, a fill ratio above 0.5 and a coupling below 0.5. A reason that fits, not tested separately: when the dial scales a pulsing push, the winning launch phases at neighbouring settings are nearly the same, so the winning pairs look like a product of the two choices. `cmb-24` ships as it was built, with the weaker bar (ablation, moved target, coupling 0.51), and is pinned by a test as failing the protocol. Dropping it is a decision for the owner of the plan.

**Limits.** In the Experiment chapter the player probes the dial with free runs and then *reports a rule*. Here the dial is only a control, and the player learns what it does by watching the arrows change (and in Tempo by scrubbing the clock); there are no separate test runs and nothing is reported. That is the precedent of `cmb-6` and `cmb-8`, but it means these levels share Experiment's dial and not its decision. Nobody has played them.

## Steer + Synchronize: the drone (`cmb-28`)

`cmb-28` Meet the Drone puts a drone in a Steer level. `Rules::Steer` gained `companion: Option<Companion>` (the drone's start, its release delays and a band); the drone flies the unsteered field by itself, released at the delay the player chooses before the first beat, and the steered probe succeeds only if it reaches the goal within the band of the drone's arrival (otherwise it ends with the new failure `MissedRendezvous`). The Steer screen draws the drone's path with its arrival time and has a row of release buttons; the validator's axis is "drone release" (the setting is the release delay, the plan is the steered probe's gust plan), so the protocol measures apply unchanged. The drone does not interact with the steered probe.

The level: a stream with a pulsing sideways push; the drone and the steered probe start 0.03 apart at the bottom of the field; 25 releases (0 to 12 s in half seconds); one gust of strength 2.0; band 0.6 s. Result (from the shipped level): 3 of 25 releases admit a meeting, the adjacent releases 0.5, 1.0 and 1.5 s, with 2, 1 and 10 winning gust plans; fill ratio 0.39, window 3, interior, coupling 0.80, informed over blind 8.3, no gust-free run, no default release and no near-simultaneous release wins, and all four moved-target checks hold.

**The informed reader for the drone is new.** The Steer validator's informed scan takes the setting at which a gust-free run ends nearest the goal. With a drone that is not what a person does, and it found a winning release for none of the first candidates. The reader for a drone level (the anchor reader of the protocol) compares the drone's arrival time at each release with the time at which a gust-free run of the steered probe is nearest the goal, and takes the release at which the two are closest. It is a measure of what a player who reads the two times can do, and it is a different check from the one used for the other Steer levels; whether it is a fair one is a decision for the owner of the plan.

**How it was found.** A random search over the field, the target, the start heights, the gust strength and the release grid, with the protocol as the filter. With 13 releases 1 s apart no design came near (the winning windows were narrower than three settings and the winning pairs looked like a product, in about 5,500 samples). With 25 releases 0.5 s apart and a stronger gust, 15,000 samples under the old informed scan found none, and 3 candidates per 5,000 passed the protocol measures; under the anchor reader all 3 qualified and one passed everything.

**Limits.** Two of the three winning releases have only one or two winning gust plans, which is close to a knife edge for those releases. The drone and the steered probe start next to each other, so the level is mostly a timing puzzle: the release shifts when the drone arrives, and the gust trims when the steered probe does. The drone reaches the target only at some releases, so part of the choice is also which releases give an arrival at all. Nobody has played it.

## The protocol, as far as it is implemented

`validate_combo::protocol(level)` computes the measures of the validation protocol that follow from a level's win sets (`ComboReport::win_sets`, one set of winning plans per setting, now filled in for Steer, Synchronize and Tempo levels): the fill ratio |W| / (|settings with a win| x |plans that win anywhere|), the longest run of adjacent winning settings, whether the winning band is interior (no win at the lowest or highest setting), the Spearman correlation of the winning (dial, phase) pairs for Tempo levels, the share of settings that admit a win, the informed-over-blind ratio where the informed scan is measured (Steer only), and the coupling. `ProtocolReport::failures()` names each measure that misses its threshold: fill ratio at most 0.5, a window of at least 3 adjacent settings, an interior band, |Spearman| at most 0.8, a random setting admitting a win at most half the time, informed at least 4 times blind, coupling at least 0.5.

| level | fill | window | interior | blind | informed / blind | coupling | measures that fail |
|---|---|---|---|---|---|---|---|
| `cmb-10` | 0.61 | 2 | yes | 0.40 | not measured | 0.78 | fill ratio, window |
| `cmb-11` | 0.41 | 10 | yes | 0.42 | 2.4 | 0.71 | informed / blind |
| `cmb-12` | 0.26 | 5 | yes | 0.20 | 5.0 | 0.94 | none |
| `cmb-22` | 0.44 | 3 | yes | 0.23 | 4.3 | 0.84 | none |
| `cmb-23` | 0.50 | 3 | yes | 0.23 | not measured | 0.78 | none |
| `cmb-24` | 0.75 | 2 | no | 0.40 | not measured | 0.51 | fill ratio, window, interior |
| `cmb-28` | 0.39 | 3 | yes | 0.12 | 8.3 | 0.80 | none |

**Decisions taken on the thresholds.** The thresholds are kept, because they are consistent with each other once the control has enough settings: a winning window of three adjacent settings together with an informed scan at least four times better than a random setting needs at least thirteen settings (three winning of thirteen is 23% blind and informed over blind 4.3), which is why the dial levels have thirteen. Levels built before the protocol are not changed silently: `cmb-10` (fill ratio 0.61 and a window of two slots) and `cmb-11` (informed over blind 2.4, because 42% of the launch phases admit a win) are pinned by a test as failing exactly those measures, so that redesigning them is a decision. The strict bar applies to new and redesigned levels.

Not implemented: the prune measure (win set with both conditions against the smaller single-condition set, which has no definition yet for these levels), the wrong-reader control, the anchor reader for Synchronize, the ten-launch blind sweep, the tolerance to a dial or phase estimate that is off, and the eight moved-geometry cases (these levels have no obstacle, so the four target shifts are all there is). The thresholds were also not calibrated against the known failures (the first `cmb-8`, early `cmb-12`, the start line of `cmb-10`, the continuous `cmb-14` and `cmb-15`), because those prototypes are not in the repository.

## Why `cmb-8` is built the way it is

The first design for `cmb-8` scaled a sideways drift with the dial. It failed the bar: a source could compensate almost any dial setting, so the dial was redundant (winning sources existed at 19 to 25 of the 25 dial settings, depending on the layout). The dial must do something a local source physically cannot. In the shipped level the vortex carries the probe round an obstacle larger than a source can deflect it around, and the placement area starts downstream of that obstacle. Too weak a dial and the probe meets the obstacle; too strong and it is thrown past the target by more than one source can undo. The source then finishes the approach. This is the substitution case the bar is meant to catch.

A lesson from `cmb-12`: with a steady background, a start offset and a gust are partly *substitutes* (a higher start saves gusts), so the winning starts always ended at the edge of the start line, which makes the start a dull "as far as it goes" decision. About 670 random backgrounds produced none with an interior window. Adding a second obstacle on the far side of the lane made too high a start fail as well, so the winning band sits inside the line. The coupling is the highest of the four (0.94) because the gusts that work change a lot with the start.

A lesson from `cmb-13`: with sources the ablation is easy to pass. A source bends the flow over the whole run and its influence is wide, so in at least a tenth of the random backgrounds tried (at least 30 of the first 300; the printout was cut off there) no gust plan wins alone, no source wins without a gust, and at least 6 sources admit a win. The hard part is making the level read well, not making it necessary. The shipped design has 13% of candidate sources winning, with an obstacle in the lane so the source has a visible job (push the probe round it), and the gust budget is one, so the gust can only finish the approach. As in `cmb-12`, the source and the gust partly substitute (the source helps the whole run, the gust adds one push); the bar is met by limiting each of them, not by making them independent.

A lesson from `cmb-9`, and a limit. The first designs (a pulsing repeller beside each lane, a source area over the whole field) passed the ablation in the base geometry, but every one of the nine tried failed one of the four moved geometries: with about 260 source positions and 3,700 pairs of delays, moving the target by 0.3 produced an accidental solution near the default delays for some source. A larger margin in the base geometry did not predict this (the nearest solution was 5.2 s away and 1.2 s away after one move). Two changes fixed it: sources only in the upper half, where probe B flies, so a source can reshape one route and not both, and the search itself had to require the ablation in all four moved geometries (7 of 1,100 random designs did). The limit: the informed-versus-blind measure is not implemented for Synchronize, so `cmb-9` has no figure for it, and the placement is the only thing a reader can use; the report prints \"not measured\".

A second lesson, from `cmb-7` and `cmb-11`: the first versions had sources winning at the default launch phase, and a target moved 0.3 down broke the ablation. A phase offset in the pulse (`phase: w * 4.25` and `w * 5.0`) puts the default launch in the dead part of the cycle, and the targets were raised until the ablation held in all four moved geometries.

## How each combination is expressed

- **`cmb-7`, `cmb-8`**: `Rules::Shape` has optional `period` and `intensity`. `ShapeGame` keeps a `phase` and an `intensity`; `launch` flies from that clock at that dial and records both in the attempt. A source is added as a fixed layer, so neither the clock nor the dial changes it.
- **`cmb-16`**: `Rules::Scout(ScoutRules)` is a new rule kind: the main probe's start, the exit regions, the number of scouts and the strip they start in, the pool of hidden sources (one strength and width) and the streak. `ScoutSpec` flies scouts and the main probe in the true field (the level's visible field plus the hidden source) and measures the push; `ScoutGame` runs scouting, one committed call per attempt, the streak and the seeded draw of the next hidden source. `validate_scout` holds the reader model, the tolerance check and the moved-gates check. The screen (`scout_frame`) shows only the visible field, the scouts' trails and push arrows, and the call buttons.
- **`cmb-9`**: `Rules::Synchronize` has an optional `sources` (the same `SteerSources` type; the name predates its second use). `SyncSpec::with_sources` builds the field every probe flies; `SyncGame::place_source`, `remove_source` and `clear_sources` work between launches (unlike Steer, a Synchronize attempt can be repeated, so nothing locks), and the screen hides the last flight when the source changes. In the validator a source is a setting (0 is none) and the plan is a pair of release delays on a 0.2 s grid; each probe is flown once per delay and every pair is judged on the recorded arrival times, which is exact because probes do not interact. Two probes only.
- **`cmb-13`**: `Rules::Steer` has an optional `sources` (`SteerSources`: placement zones, kinds, a limit, strength and width, the same placement rules as Shape). `SteerSpec::with_sources` builds the field from the level's own plus one steady term per placed source (it never stacks); `SteerGame::place_source`, `remove_source` and `clear_sources` work only before the first beat. A retry keeps the placed sources. A level has at most one of `period`, `start_zone` and `sources`. In the validator a source is a setting: index 0 is no source, then every kind at every legal site on a 0.3 lattice.
- **`cmb-12`**: `Rules::Steer` has an optional `start_zone` (a `Rect`; a zero-width one is a line). `SteerSpec::with_start` and `SteerGame::set_start` choose the start point inside it, only before the first beat and only at a legal point (inside the zone and outside every region). A retry keeps the chosen start. A level has either a `period` or a `start_zone`, never both.
- **`cmb-11`**: `Rules::Steer` has an optional `period`. `SteerSpec::with_clock` sets the field clock at beat 0; `SteerGame::set_clock` works only before the first beat and is locked once a beat is played. A retry unlocks it again.

- **`cmb-1`**: `Region::gate(...)` in the region list; `Level::reached_goal` then requires every gate in the trajectory's visited list. No code in `PositionGame` is gate-specific.
- **`cmb-2`**: `Rules::Predict{..., reveal: Reveal::Partial(2.5)}`; `PredictGame::reveal_mode` exposes it and the interface draws `trajectory.upto(2.5)` before the commit.
- **`cmb-3`**: `Region::hazard(...)`; a hazard contact ends the run as a miss (`Outcome::Hazard`), so `reached_goal` is false.
- **`cmb-4`**: a normal Experiment level; only the challenge parameters differ from `exp-5`.
- **`cmb-5`**: `PositionGame` with a period: `set_phase` and the timeline are enabled, and `launch` passes the phase as the clock.
- **`cmb-6`**: several `Slot`s (one per zone). `place_at` routes a click to the zone that contains it; `launch` simulates each probe independently at the shared intensity, and the attempt succeeds only if every slot succeeds.

## Why they exist

## Route and Rendezvous: start slots (`cmb-10`)

`cmb-10` (Synchronize + Position) is the Synchronize level with a categorical start choice. Probe B starts from one of five numbered slots (a click on a marker, or a number in the panel); probe A's start is fixed; both release delays are free (0 to 12 s, the whole period). `Rules::Synchronize` gained `start_slots: Option<(probe, slots)>` (slot 0 is the probe's default start), `SyncSpec::with_slot` and `SyncGame::set_slot`; the combination validator gained the axis `Axis::Slots(n)`, whose settings are the slots and whose default is slot 0, and `sync_report` measures the same quantities as for `cmb-9`, with a slot where `cmb-9` has a source.

The bar (`validate_combo::sync_report`, all from the real simulation, delays on a 0.2 s grid):

| measure | `cmb-10` | what it means |
|---|---|---|
| wins at the default slot | 0 | from the default start no pair of delays brings the two arrivals within the band, so the start has to be chosen |
| wins with both probes released within 1.5 s of each other's zero | 0 | at no slot does a near-simultaneous release win, so the delays have to be used |
| slots that admit a win | 2 of 5 | a random slot usually loses |
| most winning delay pairs at one slot | 240 | the winning delays are a wide set, not a knife edge |
| coupling (mean Jaccard distance between slots' win sets) | 0.78 | the two decisions depend on each other |
| robust to the target moved by 0.3 in four directions | 4/4 | the ablations still hold |

**Why discrete slots work where a line failed.** The line attempt (above) failed because moving the target changed the geometry both probes arrive in, so the winning (start, delays) set shrank or emptied. With five slots the win sets are wide in the delays (240 pairs), so a 0.3 shift of the target moves them without emptying them, and with only a few starts to choose from it is easy to keep the default and the winning slots apart.

**How it was found** (a random search that is not part of this repository): a stream plus two pulsing Gauss sources (`cmb-9`'s recipe) at random places and phases, probe A's start height, five random slots for probe B, and the target height, all at two decimals. The bar above (the base ablation, one to three winning slots, at least 15 winning delay pairs, and the four moved-target checks) was passed by 6 of 300 samples, and the one shipped has the fewest winning slots (2 of 5).

**Limits.** The informed-versus-blind comparison is not measured for Synchronize levels (as for `cmb-9`). The search found this kind of level easily (6 of 300 samples), so it is worth playing: the bar establishes that both decisions are necessary, not that the level is enjoyable, and nobody has played it. The hint says that two of the five slots work, which is true of this design.

## Survey + Synchronize: tried, not built

A scout-then-act level with an act that chooses the release delays of two probes was prototyped (probes A and B started at different heights, eight numbered delay pairs: no delay, and B later or A later by 1 to 4 steps of 1.0 or 1.5 s, meeting band 0.6 s, judged by `SyncSpec::fly`) and searched with the scout bar. It was removed again, because no design came close:

- Version 1 (a stream and a swirl, three hidden sources anywhere, strength 2.01 or 3.0): 40,000 samples; in 3,000 of them only 24 had a winning option for all three hidden sources, and 3 in 40,000 had winners that were disjoint. None passed the control and reader checks.
- Version 2 (hidden sources placed near one probe's lane, strength 3.0 or 5.0, option spacing 1.0 or 1.5 s): 1 in 40,000 had disjoint winners, none passed.

The reason seems to be scale: a hidden source changes a probe's travel time by less than the width of a winning delay window (the band is 1.2 s wide), so one option wins for several hidden sources, and where the effect is larger a probe often misses the target altogether. A different shape of act (for example choosing among few widely spaced delay differences with a wider band) might work but was not tried. Survey + Experiment (choose the dial) was also tried and found nothing, because the dial only scales the background (see the `cmb-19` section).

## The 28 pairs, and the pairs not built (judgment, not tested)

Every pair of the eight mechanics and where it stands (Position P, Predict Pr, Tempo T, Experiment E, Shape Sh, Steer St, Survey Su, Synchronize Sy):

| status | pairs |
|---|---|
| built (18) | P+T `cmb-5`, P+E `cmb-6`, P+St `cmb-12`, P+Su `cmb-14`, P+Sy `cmb-10`, Pr+Su `cmb-16`, `cmb-18`, T+E `cmb-24`, T+Sh `cmb-7`, T+St `cmb-11`, T+Su `cmb-19`, `cmb-29`, E+Sh `cmb-8`, E+St `cmb-22`, E+Sy `cmb-23`, Sh+St `cmb-13`, Sh+Su `cmb-15`, Sh+Sy `cmb-9`, St+Su `cmb-17`, St+Sy `cmb-28` |
| skipped on purpose in the plan (4) | P+Pr, P+Sh, Pr+Sh, T+Sy |
| searched for, not found (2) | E+Su (the dial as the act), Su+Sy |
| argued, not tried (4) | Pr+E, Pr+T, Pr+St, Pr+Sy |

(Pr+Su has two levels, `cmb-16` and `cmb-18`, and T+Su two, `cmb-19` and `cmb-29`, so the 18 built pairs have 20 levels; 18 + 4 + 2 + 4 = 28.) An earlier version of this section said the combination levels covered 18 pairs and that four were untried; that count was wrong, because before `cmb-28` the built pairs were 17 and five were untried (the four above and Steer + Synchronize). The reasons for the four argued pairs are arguments from how the mechanics are defined, not results from a search or a playtest:

| Pair | Why no level was built |
|---|---|
| Predict + Experiment | `exp-4` (the Extrapolate challenge, `docs/05-experiment.md`) already makes the player infer an outcome beyond what the rig can run, so the pair's natural level exists. |
| Predict + Tempo, Predict + Steer, Predict + Synchronize | A forecast adds a statement but changes nothing the player does, which is the reason Position + Predict and Predict + Shape were skipped. |

An earlier version of this table also listed Experiment with Steer and with Tempo as pairs without a natural level, on the argument that Experiment's decision is a report of a rule and not an action toward a goal. That was an argument, not a result: `cmb-22` to `cmb-24` (below) show that the pairs work when the dial is a pre-run control, as in `cmb-6` and `cmb-8`. The same caution applies to the rows above: each needs a reason why its two decisions depend on each other, and none of them has one yet, but that is not proof that none exists.

The random searches named above are not part of this tree, so the rates and counts quoted for them cannot be reproduced from the repository alone. What is checked is the bar each level passes: `tests/scout.rs` and `tests/combo.rs` assert it, so a level that stops meeting it after an engine change fails a test.

## Survey + Shape + Tempo: tried, not built

A third three-decision level was searched for in the way of `cmb-20` and `cmb-21`: the hidden source is on one of four places and the opposite kind there cancels it exactly, and the attempt also ends with a choice of launch phase in a field that pulses with a twelve-second period (a stream, a swirl and a sideways push), the default phase 0 being the neutral choice. No design met the bar:

- Six phases 2 s apart, target radius 0.8 to 1.0: 30,000 samples; 722 had disjoint winners, 25 needed both controls and were read correctly, none survived the target moved by 0.3 (one design survived a shift of 0.15, with tolerance 100%).
- Twelve phases 1 s apart, target radius 0.9 to 1.0: 30,000 samples; 267 had disjoint winners, 14 passed the controls and reader checks, none survived 0.3.

A reason that fits the numbers, but was not tested separately: in a pulsing field the winning phases move when the target moves, so a shift of 0.3 empties or overlaps the winning phase windows even when they are wide enough for the reader. The four-decision version (cancel, release slot and launch phase) was started but its search did not finish, so there is no result for it.

## Survey + Tempo II (`cmb-29`)

A second Survey + Tempo level was first searched for by random sampling, with the protocol as the filter: a pool with both attractors and repellers, a stream and a pulsing sideways push, a winning window of at least three adjacent phases for every hidden source, winning phases disjoint across sources, phase 0 winning for none, a wrong-kind reader (it guesses the opposite kind of the same source at the right place) right for at most one source in three, tolerance at least 0.9 and the four moved-target checks. No design passed in about 380,000 completed samples:

- Twelve phases 1 s apart, the first pulse range: 100,000 samples; 732 had disjoint winners and 142 passed the controls and reader checks.
- The same, with a weaker pulse and a target of radius 1.0: 100,000; 399 and 136.
- Twenty-four phases 0.5 s apart: 60,000; 241 and 84.
- Twenty-four phases, with the window-centre reader below: 60,000 and 60,000; 273 and 253 had disjoint winners, 104 and 89 passed the controls and reader checks.

Of about 40 candidates that passed the controls and reader checks (24 phases, first-working reader), only 3 had every window at least three phases wide, and those failed tolerance (0.33, 0.75 and 0.29): the winning windows are narrow, and a 0.3 error in the estimate of the source moves them. That is below the stop rule of about 1 in 100,000, so the search stopped. The construction below the line was not what worked, and is kept as a record of the reasoning: the launch phase shifts the probe sideways by a sinusoid of the phase, a source's kind flips the sign of what has to be compensated (so the right phases for the two kinds at one place should be half a cycle apart) and its distance sets how much, and windows are widest where the sinusoid is flat, near its peak and trough; sources whose compensation sits there for two of them and mid-way for the third would give wide, separated windows. This is reasoning, not a result.

**What worked: selection instead of sampling.** For each sampled field (a stream and a pulsing sideways push, target radius 1.0), the winning phases of every candidate source were computed once (two kinds at 30 grid places, 60 candidates, 24 phases 0.5 s apart), the candidates kept were those whose winning phases include no phase 0, form a run of at least four adjacent phases and number at most twelve, and then triples of kept candidates with disjoint winning phases and both kinds were run through the full check (reader 3/3, controls needed, visible-only right for at most one source, wrong-kind reader right for at most one, tolerance at least 0.9, four moved-target checks). In 300 fields, 264 had three wide candidates, 205 triples were tried, 148 passed the report and 2 passed everything. The shipped one has the sources repeller (-2, 1), repeller (2, -2) and attractor (-3, 0) in a stream (1.37, 0.2) with a push (0, 1.14) of amplitude 0.66 (phase 3.09), a target at (4.5, -0.66) of radius 1.0 and source strength 3.0. Winning phases: 0.5 to 2.0 s, 9.5 to 11.5 s and 5.5 to 7.0 s (windows of 4, 5 and 4 phases); phase 0 wins for none; reader 3/3, tolerance 100%, visible-only right for 0 of 3, best fixed phase for 1 of 3, 4/4 robust. This is the method of the `cmb-17` gust plans (enumerate what wins, then choose) applied to the sources, and it is why a level that 380,000 random samples did not produce came out of 300 fields.

**The window-centre reader.** The reader of a single-choice scout level picks the first option that works for its estimate of the source. For an ordered control such as the launch phase that is the edge of a window, which a small error in the estimate loses. `ScoutSpec::pick_center` instead picks the middle of the longest run of adjacent working options, and `validate_scout::report` uses it for `Phase` acts (as it uses `pick_robust` for sequences). `cmb-19` still passes with it (3/3, tolerance 100%, robust 4/4). It is a change of the bar for launch-phase levels and a more realistic reader, not a weaker one: it is the choice a person who can see the window would make.

## Planned levels, renumbered

The ids `cmb-22` to `cmb-25` are taken by the levels above, so the levels that were planned and are not built take the next free ids. None has a design yet.

| id | title | pair | the decisions |
|---|---|---|---|
| `cmb-26` | Test, Then Forecast | Experiment + Predict | which intensities to test, then call the exit at untested intensities |
| `cmb-27` | Edge of the Window | Experiment + Tempo | which launch phases to test, then conclude the interval of phases that reach the target |
| `cmb-28` | Meet the Drone | Steer + Synchronize | **built** (see its section) |
| `cmb-29` | Scout, Then Time It II | Survey + Tempo | mixed attractors and repellers, so the right phase depends on the kind and place of the source **built** (see its section) |
| `cmb-30` | Pick the Lane | Survey + Shape + Position | built so that the right slot changes with the hidden source |
| `cmb-31` | Tide, Source, Gust | Steer + Shape + Tempo | phase, one source and one gust |

## Attempts that failed first (kept as a record)

From the plan: `cmb-10` Route and Rendezvous (Synchronize + Position) was first attempted with a start *line* and **not shipped** then, because no design met the bar; it is built now with discrete start slots (see "Route and Rendezvous: start slots"). The attempt with a line: probe B's start point is chosen on a line (the "Position" decision), the delay range is limited to 2 to 4 s (so delays alone cannot fix the timing), and the field is the pulsing-lanes recipe of `cmb-9`. More than 15,000 random designs were scored with the real simulation. A vertical start line almost never helped (1 design in 600 had even three winning starts); a horizontal line along the direction of travel did (about 1% of designs passed the base ablation). But none survived the target being moved by 0.3 in all four directions, except one design with a larger target (radius 1.2), in which 9 of the 15 start points win, so the start point hardly matters, which fails the \"a random setting usually loses\" requirement. The failure is the one found in `cmb-9`: moving the target in y changes the geometry both probes arrive in, so the set of winning (start, delays) combinations changes a lot, and sometimes empties. What would help, untested: a structure where the start point acts on only one route (as the upper-half source area did in `cmb-9`), or a fixed set of discrete starts instead of a line. The exploratory engine work (a `start_zone` for one Synchronize probe and a start-point control in `sync_report`) is kept in `cmb-10-wip.patch`, not in the code. `cmb-14` and `cmb-15` (learn, then act) were first prototyped with a continuous act and **not shipped** then, because no design met the bar; `cmb-16` was redesigned around a categorical act, and `cmb-14`, `cmb-15` and `cmb-17` were then built the same way (see above). The notes below describe the continuous attempts, which remain a record of why the act is categorical. What was tried:

- **`cmb-15` Cancel It (Survey + Shape).** Idea: a hidden source from a pool of four (repeller or attractor) bends the probe; scouts reveal its push; a placed source of the opposite kind at the same spot cancels it exactly. About 500 random designs were scored. With a target radius of 0.5 to 0.6, nearly all designs either had more than 15% of placements winning or had a single placed source that won for 3 or 4 of the 4 hidden positions (an attractor near the end of the lane pulls any deflected probe onto the target). With the placement area cut back (sources only at x of 1.5 or less) and target radii of 0.15 to 0.3, 5 of 120 designs passed, including an estimator that triangulates the source from three scouts. For the two of those I measured (both radius 0.15) the tolerance (the radius around the hidden source within which an opposite-kind source still wins) was only 0.1 to 0.15, which cannot be read off scout trails. Growing the target to 0.3 and 0.45 on the same two pools raised the tolerance to 0.2 to 0.45, but the funnel attractor returned (it won for 3 or 4 of the 4 hidden positions, from the edge of the placement area).
- **`cmb-14` Scout, Then Release (Survey + Position).** Idea: three scouts are test flights from a release line, a hidden source from a pool bends them, then one release is committed. About 32,000 random designs were scored, with an adaptive reader (secant steps on the scouts' misses). The required properties: the default start fails, the winning windows of the four hidden positions are disjoint (so one start cannot pass the level without scouting), and each window is wide enough to hit. With windows of any width, 9 of 8,000 designs passed, but their windows were 0 to 0.3 units wide (some a single 0.1 lattice point). Requiring windows of at least 0.2 wide, none of 24,000 passed: the windows of different hidden positions overlapped (127 and 360 designs in the two runs) or the default start won. In the passing designs the winning starts sit at similar heights (about 0.4 to 1.2 above or below the lane) whatever the hidden position, which suggests, without proving it, that wide windows get shared.
- **What the two failures and `cmb-16` have in common.** A continuous act (a placement, a start height) needs a precision a person cannot read off scout trails, and widening the tolerance lets one answer serve several hidden positions. A categorical act with wide gates and one exit per hidden source avoids both. `cmb-17` was built with a categorical act (a fixed gust plan).

The prototypes are in `staged-prototypes.zip` (scratch test files for `tests/`). Pairings skipped on purpose: Position + Shape (a sideways source substitutes for a sideways start), Tempo + Synchronize (the delays are launch phases), Position + Predict and Predict + Shape (Predict adds no real choice there).

## Why the first six exist

Each of the first six has a measured reason to exist: the added element removes a large share of the previously winning options (cmb-1, cmb-3, cmb-5, cmb-6), or adds evidence that changes the reasoning (cmb-2), or moves the discovery problem (cmb-4). `combination_levels_each_add_exactly_one_idea` in `tests/mechanics.rs` asserts the first group numerically.

Parameters: [LEVEL_CATALOG.md](LEVEL_CATALOG.md).
