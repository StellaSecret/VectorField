# 10. Shape: "How should I change the field?"

Shape is the converse of [Survey](09-survey.md). Survey infers a field from its effects; Shape creates the effects by changing the field. The probe starts where it always starts. The player adds up to one or two local sources (repeller, attractor, swirl) to a fixed background, launches the probe, and watches where the combined field carries it. Code: `src/modes/shape.rs`, validation `src/validate_shape.rs`, front end `src/app.rs::shape_frame`, levels `src/levels.rs::shape_levels`, tests `tests/shape.rs`.

## 1. At a glance

| | |
|---|---|
| Question | How should I change the field to make the path I want possible? |
| Decision | which kind of source and where, for up to `sources` sources (1 in levels 1-3, 2 in level 4); sources can be moved and removed freely |
| Timing | before each launch; nothing can be done during the run |
| Feedback | the arrows redraw live as a source is placed or dragged; after a launch, the real trajectory through the combined field |
| Retry | free and unlimited; the sources stay where they are; the last three launches stay as faint trails |
| Win | the probe reaches the target; an obstacle (a hazard region) ends the run |
| Fail | hazard, leaving the field, timeout, coming to rest: a miss, never a lost level |
| Fixed | the background field, the probe's start `(-5, 0)`, the obstacle(s), the target, the source strength `k` = 1.65 and width `sigma` = 0.8 (the Survey source), the placement area |
| Hidden | nothing is hidden, but the result of a configuration is known only by flying it: **there is no trajectory preview** |

## 2. Information

Always visible: the arrows of the *combined* field (background plus the placed sources: `ShapeGame::field`; `G` changes their density, `K` makes them high-contrast), the probe's start, the target and obstacles, the placement area (a dashed frame, or shading when the area is small), every placed source with a dashed ring at its influence radius (where its push falls to `PUSH_MIN` = 0.2, 1.87 for the shipped source; `ShapeSpec::influence_radius`) and an arrow glyph for its kind (out, in, or around, drawn the way the world turns), and the faint trails of the last three launches.

Never shown before a launch: any path. The front end calls `ShapeGame::launch` in exactly one place (the Launch button) and nothing else that flies a probe; this is enforced by a test that reads the source of `shape_frame` (`the_front_end_computes_a_path_only_when_the_launch_button_is_used`). A launch with nothing placed is allowed and free: it shows what the background does by itself, which is the first hint of every level.

## 3. Controls

Mouse: click inside the area to place a source of the selected kind; drag a source to move it (an illegal position leaves it where it was); right-click, `Del`, `Backspace` or `X` removes the selected or clicked source; "Clear all" removes all. `1`-`4` or `Tab` (or the buttons) select the kind (only the kinds the level offers). Keyboard-only: the arrow keys move a cursor (Shift for fine steps), `P` places at it (when no sources are left, it moves the selected source there). `Space` launches, `R` clears the result (sources stay), `V` replays the last launch, `H` hints.

## 4. State and loop

`ShapeGame { spec, sources, sel, attempts, won }`. A loop: edit sources (any number of times) -> `launch` (records an `Attempt { sources, traj, success }`) -> read the result -> edit. `won` latches on the first success and is never taken back; the player may keep experimenting. Placement, movement and removal never simulate (tested by `editing_the_field_never_flies_a_probe`).

## 5. Exact rules

- **A source** is `SourceKind::term(pos, k, sigma)`: a `Term::Gauss` (repeller `k`, attractor `-k`) or `Term::GaussVortex` (counter-clockwise swirl `k`, clockwise `-k`). The combined field is the background with one `Field::fixed` layer per source (`ShapeSpec::field_with`); fields add, and order does not matter (tested). No new engine concept was needed.
- **Legal placement** (`ShapeGame::can_place`, `ShapeSpec::site_error`): inside a placement zone; not on a target or hazard (centre within its radius); at least `MIN_SEPARATION` = 0.5 from every other source; fewer than `sources` placed. A move is legal on the same terms, except that it does not count the source itself. A refused action changes nothing.
- **A launch** is `Level::run` through the combined field from `start` at time 0, limit 25 s. Success is `Level::reached_goal` (no gates in these levels).
- **The probe has no inertia**: its velocity is the field's value. Two consequences shape every level and are worth knowing (they were measured in the real simulation before any level was written):
  1. In a steady stream a repeller or attractor shifts the path *sideways, for good*, by an amount that depends on how far off the path it sits and **not on where along the stream it sits** (the same offset for a source at x = -3 and at x = 1). The offset is largest for a source about 0.5 off the path (1.27 for a repeller) and falls away beyond about 1.9.
  2. An attractor near the path draws the probe toward *its own height* (a source at height 1.0 leaves the probe at 0.99), so it works as a lane selector. A swirl pushes the probe sideways on the way in and back on the way out: **a bump with no net shift** (the residual at x = 5 is under 0.05).

## 6. Success and failure

Success: the probe's path touches the target region. Failure is only a missed launch (the obstacle, the edge of the field, the 25 s limit, or coming to rest, which an attractor placed right on the path can cause). The message names what happened ("The probe hit the obstacle. Change the field and launch again.").

## 7. Invariants and edge cases

- The start is the same in every attempt; the player cannot move it (so Shape is not Position).
- A source dragged onto its own old position, or in small steps, is never "too close" to itself.
- A win is kept even if the player then clears the field and launches again.
- Two sources on the same lattice point are refused (`TooClose`); a pair of sources of one kind is allowed (and common among level 4's solutions).
- The placement area deliberately stops short of the target in level 2, so the target's own neighbourhood is not a shortcut; in all levels a source cannot sit on the target.
- None of these backgrounds has a point where the flow is zero, so a launch always moves; the exception is an attractor right on the path, which can hold the probe at its equilibrium (outcome `Stalled`; 138 of the 255 attractor placements in level 3 do this).

## 8. Feedback

Messages (`shape_frame`): "Success! The probe reached the target."; "The probe {hit the obstacle | left the field | ran out of time | came to rest | hit a numerical fault}. Change the field and launch again."; "With no sources the probe ... ."; placement refusals: "Sources can only go inside the shaded placement area.", "Not on the target.", "Not on an obstacle.", "Too close to another source.", "No sources left: move or remove one." A red cross under the cursor and a note in the panel say why a click would be refused before it is made. The completion banner reports the launches used.

Not verified: the front end has not been looked at on a screen in this work (the code compiles and its logic is tested, but there was no display), so spacing, the glyphs and the panel layout need an eyeball pass.

## 9. Levels

All four use the Survey source (k = 1.65, sigma = 0.8), the start `(-5, 0)`, a 25 s limit and the same hint structure: (1) launch first and see what the field does by itself, (2) what the source does, (3) a question about *where* and *when* it has to act, never the answer. Numbers below are from `docs/SHAPE_VALIDATION.md` (the real simulation, a 0.25 lattice of legal placements for one source, 0.5 for pairs).

| Level | Offered | Blind win | Win by kind | Reader (sites, win) | Thickness |
|---|---|---|---|---|---|
| `shp-1` Deflect | repeller | 9.4% | repeller 9.4% | 48, 31% | 0.80 |
| `shp-2` Recover | repeller, attractor | 8.6% | repeller 0%, attractor 17.2% | 51, 43% | 0.43 |
| `shp-3` Turn | repeller, attractor, swirl CCW, swirl CW | 7.0% | radial 0%, swirl CCW 14.1%, CW 13.7% | 264, 26% | 0.66 |
| `shp-4` Compose | repeller, attractor (2 sources) | pairs 3.08% | single sources 0% | sequential reader 46% of 213 pairs | n/a |

**Level 1, Deflect.** A uniform stream (1, 0), a hazard of radius 0.6 on the path at (-0.5, 0), the target above the path at (4.5, 1.3) radius 0.7. Only a repeller. It wins from below the path and upstream of the obstacle (every winner has y < 0 and x < -0.4) and from nowhere on the other side: the same distance above the path pushes the probe down into the stream's other half. Because of the lane effect the winners spread over x (about 3.5 units) but only over a narrow range of y (0.25 to 1.25 below the path): the decision is "how far off the path, on which side", and "before the obstacle". Baseline: hits the hazard.

**Level 2, Recover.** A stream with a slight downward drift (1, -0.08), so the probe passes about 2 below the target at (3.8, 1.2) radius 0.7; a hazard of radius 0.5 in the target's lane at (0.2, 1.2); the placement area is above the path and ends at x = 2.8, short of the target. A repeller in the area can only push the probe further down (0 wins at a 0.125 lattice, tested); an attractor wins when it sits at about the target's height and is *downstream of the obstacle* (about 17% of attractor sites). An attractor placed upstream of the obstacle pulls the probe into it (8 of the sites on the 0.25 lattice end on the hazard; at least 3 are tested for). This is the "an attractor can also pull the probe somewhere undesirable" lesson, made by the real simulation rather than by a hint.

**Level 3, Turn.** The stream is slower (0.7, so a source has time to act), the hazard is *on the stream* at (0, 0) radius 0.55 and the target straight behind it at (4.5, 0) radius 0.5. Every offered radial source (repeller or attractor) that clears the obstacle leaves the probe shifted off the line of the target (0 wins, tested at a 0.125 lattice, and 100+ of them do clear it); a swirl bumps the probe round the obstacle and lets it come back. Which way it turns decides which side the bump goes (counter-clockwise takes the probe under, clockwise over; tested on every winner). The swirl has to sit within about 1.3 of the obstacle's x. The arrows show it: on the approach side a counter-clockwise swirl's arrows point down, on the departure side they point up.

**Level 4, Compose.** A uniform stream, a hazard on the path at (-0.5, 0) radius 0.55, the target at (4.5, 2.8) radius 0.7, which is higher than one source can lift the probe (a source is felt out to 1.87): no single source of either kind wins (0 wins at a 0.125 lattice, tested). With two, 3.08% of legal pairs win (359 of 11,652); 217 use two sources of the same kind and 142 two different kinds. The first source lifts the probe into the reach of the second, so where the second can go depends on where the first put the probe: of the winning pairs ordered by x, the second source's set of winning partners has a mean Jaccard distance of 0.56 between different first sources (`coupling`; 0 would be two independent corrections, 1 would be no overlap). Removing either source of a winning pair loses (tested).

## 10. Distinctness and known weaknesses

What differs from the other mechanics, in the terms of [08-comparison.md](08-comparison.md): what is controlled (the field, not the probe, a time, a parameter, or a sequence of pushes) and what is observed between decisions (a finished path, as in Position).

Where the claim is weaker than it sounds:

1. **The interaction loop is Position's.** Place, launch, look, adjust, with unlimited free retries. A player *can* sweep placements until one wins; in level 1 a blind placement wins 9.4% of the time, so about 10 launches. What is meant to differ is the reasoning that makes a placement worth trying, and the numbers say it is worth something: a reader of the field (below) wins 3 to 5 times as often per try in levels 1-3, and about 15 times in level 4. That is an assumption about reasoning, not a measurement of players.
2. **The reader model is simple and was chosen after seeing numbers.** `validate_shape::reader_sites`: it considers a site only if the push there, at the first point where the probe meets the source, is visible (0.25) and mostly toward the side of the path the target is on; if the target is straight ahead it accepts a mostly-sideways push from a source that touches the path within one influence radius of where the probe was stopped. The "within one influence radius" was settled after trying 2, 1.5 and 1: the level 3 advantage is 2.2x, 2.6x and 3.7x. The tests ask only for 2x (5x for level 4), not for the headline number.
3. **Level 2's solution is "put the attractor at the target's height, after the obstacle".** It is a lane, and that is guessable. The winning region is thin in y (thickness 0.43: a winner's neighbours half a step away often lose). It is the thinnest level.
4. **Level 3's orientation has a mirror.** Both swirl directions win (from opposite sides), so the decision is which direction goes with which position, not one unique swirl.
5. **Level 4 coupling is measured on a lattice and ordered by x**, and its first source has only 15 distinct useful positions. It shows the second source depends on the first; it does not show that a player will find the relay by reasoning.
6. **The placement area is a hint.** In level 2 the area is above the path, which tells the player which side matters.
7. **Not playtested, and the front end not looked at on a screen.**

## 11. Tests

`tests/shape.rs` (20 tests): `shape_is_a_chapter_with_four_validated_levels`, `adding_shape_leaves_the_other_chapters_as_they_were`, `each_kind_changes_the_field_the_way_its_name_says`, `several_sources_add_and_their_order_does_not_matter`, `placements_are_legal_only_inside_the_area_and_off_the_regions`, `sources_keep_their_distance_and_only_the_offered_kinds_can_be_selected`, `a_source_can_be_dragged_replaced_and_removed`, `a_launch_flies_the_combined_field_and_records_it`, `a_winning_launch_reaches_the_target_and_the_win_is_kept`, `editing_the_field_never_flies_a_probe`, `the_front_end_computes_a_path_only_when_the_launch_button_is_used`, `level_one_a_repeller_below_and_upstream_of_the_obstacle_wins`, `level_two_needs_the_attractor_and_a_pull_can_carry_the_probe_into_the_obstacle`, `level_three_needs_the_swirl_and_its_direction_decides_the_side`, `level_four_no_single_source_wins_and_the_second_depends_on_the_first`, `blind_success_is_in_the_range_of_the_existing_levels`, `reading_the_field_beats_blind_placement`, `levels_do_not_depend_on_one_exact_geometry`, `winning_placements_are_regions_not_lattice_points`, `shape_changes_the_field_not_the_start_and_that_changes_what_wins`. The generic suites cover Shape too (`every_level_is_solvable`, `simulation_is_deterministic_for_every_level`, the catalog test), and the other chapters' suites are unchanged.

The ranges asserted come from the existing levels, not from a free choice: Position levels win 6.5-20% of blind releases (the first 100%), Tempo 9-32% of launch phases, Survey's free scout alone 18-33% of random sources. Shape levels 1-3 are asked to land in 4-35%; level 4 in 1-10% of pairs.
