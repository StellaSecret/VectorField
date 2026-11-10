# 11. Synchronize: "How must these probes be coordinated?"

Several probes, each with a fixed start, fly the same fixed field. The player chooses only **when each one is released**. Success is a *relationship* between the probes: every probe reaches its goal and every timing condition holds (for example "B enters the target within 0.6 s of A"). Code: `src/modes/synchronize.rs`, validation `src/validate_sync.rs`, front end `src/app.rs::sync_frame`, levels `src/levels.rs::sync_levels`, tests `tests/sync.rs`.

**Status: a prototype with a qualified verdict.** The plain rendezvous the brief proposes first turned out to be arithmetic and is *not* distinct from Tempo (section 10, measured). Synchronize becomes a different kind of reasoning only when each probe can reach its goal at certain times only, or when conditions pull against each other. Levels 2-5 do that; level 1 is an on-ramp and says so.

## 1. At a glance

| | |
|---|---|
| Question | How must these probes be coordinated so the timing condition holds? |
| Decision | one release delay per probe, 0 to 12 s in steps of 0.1 s (12 s is one field period; the delay is the field clock at release, as in Tempo) |
| Timing | before the launch; nothing can be done during the flight |
| Feedback | after a launch: a playback on the shared clock, then, on a timeline, each probe's flight bar, its arrival, and the allowed band for the others; in words, which condition failed and by how many seconds (too early / too late) |
| Retry | free and unlimited; the delays stay as they are |
| Win | every probe reaches its goal and every condition holds |
| Fixed | the field (periodic with period 12 s except in level 1), the starts, the regions, the conditions |
| Hidden | nothing, but an arrival time is known only by flying: **nothing is flown before a launch** |

## 2. Architecture audit (what was reused, what was needed)

- **Launch delays already exist.** `simulate(field, start, t0, ...)` takes the field time at release; `Level::run` passes it through. Tempo uses it as the launch phase. In a time-invariant field a delay changes only *when* a probe arrives, never where it goes.
- **Several probes need no engine change.** A probe is one independent `Level::run`; the front end already animates several trajectories at once.
- **One missing piece:** the engine records the final outcome and which gates were passed, but not *when* a region was entered. `modes::synchronize::entry_time` finds the first sample inside a region (`Trajectory::t0 + k * dt`).
- **One engine-level subtlety:** `Level::reached_goal` requires *every* gate to be passed. Synchronize uses `outcome == Reached(goal)` instead, because a checkpoint can belong to one probe's route; a condition that mentions a gate needs its entry time, which is the same requirement.
- **Rules:** `Rules::Synchronize { starts, goals, period, max_delay, conds }` and `SyncCond { a, ra, b, rb, lo, hi }`: probe `b` enters region `rb` between `lo` and `hi` seconds after probe `a` enters `ra`. One type covers a rendezvous (`ra == rb`, `lo = -w`, `hi = w`), an order (`0 < lo < hi`) and a checkpoint on each route (`ra != rb`).
- **Validation:** `solvable`, `oracle_win` (the generic one, which flies each probe at its best delay) and `summarize` are wired in `validate.rs`; the analysis is `validate_sync`.

## 3. The finding that shaped everything: independent probes make timing arithmetic

Probes in a fixed field do not interact, so the problem has a closed form:

- **Static field.** A probe's travel time `T_i` does not depend on when it leaves. Success depends only on `delay_B - delay_A` (a subtraction of two travel times). Measured on level 1: shifting both delays by 1 s or 2 s never changes the result (**shift-break 0%**), the first probe can leave at *any* time and the second can still be matched (**anchor 100%**), and a player's per-probe skill is worthless (Tempo-only win % = blind win % = 6.4%). This is arithmetic, and it is *less* than Tempo (no phase to read).
- **Periodic field, every probe always reaches its goal.** Travel time now depends on the release time, and success depends on the absolute phase (shift-break 46% in the first periodic prototype). But arrival `a_i(d) = d + T_i(d)` is increasing, so for any common arrival time `a*` each probe has a release that makes it arrive then: choose `a*`, then solve each probe alone. That is two Tempo problems and a shared number (anchor 100% again).
- **What breaks the decomposition:** each probe can arrive only during certain times (it reaches the target only if it passes its own pulsing push at the right moment), or conditions that pull against each other. Then the *sets* of possible arrival times have to be matched, and the probes cannot be solved one at a time.

A second physical fact limits the conditions: **two probes at one place at one time share a future**, so "meet at the gate, then meet again at the target" is redundant (151 winning combinations for the gate alone and 134 for both in the first version of level 4). Level 4 therefore asks the opposite: meet at the gate, then arrive apart.

## 4. Levels

All use the 12 s period, the 0.1 s delay step and a 20 s flight limit. Numbers are from `docs/SYNC_VALIDATION.md` (every probe flown once per delay in the real simulation; the joint condition evaluated on the recorded arrival times, which is exact because the probes do not interact).

| Level | Probes | Condition | Each probe alone reaches the target | Blind win | Tempo-only win | Anchor | Shift-break 1 s / 2 s | Width |
|---|---|---|---|---|---|---|---|---|
| `syn-1` Rendezvous | 2 | meet at the target within 0.4 s | 100%, 100% | 6.42% | 6.4% | 100% | 0% / 0% | 0.8 s |
| `syn-2` Windows | 2 | meet at the target within 0.6 s | 37%, 37% | 2.04% | 14.7% | 84% | 47% / 86% | 1.1 s |
| `syn-3` Order | 2 | B arrives 1.0 to 2.2 s after A | 37%, 37% | 1.26% | 9.1% | 51% | 64% / 99% | 1.1 s |
| `syn-4` Meet, Then Part | 2 | meet at the gate within 1.0 s, then B arrives 0.4 to 1.2 s after A | 100%, 100% | 0.59% | 0.6% | 13% | 100% / 100% | 0.7 s |
| `syn-5` Chain | 3 | A and B meet at the gate within 0.8 s; B and C meet at the target within 0.8 s | 100%, 100%, 73% | 0.12% | 0.2% | 21% | 100% / 100% | 1.3 s |

*Blind*: random delay combinations that win. *Tempo-only*: combinations that win among those where every probe can reach its goal alone (what a player gets by solving each probe separately, as in Tempo). *Anchor*: of the delays at which probe 0 can reach its goal, the share that have a winning partner. *Shift-break*: of the winners, the share that stop winning when every probe leaves 1 s (2 s) later; 0% means success depends only on the relative delays. *Width*: the median number of seconds one probe's release can move, the others fixed, with every step still winning.

**Level 1, Rendezvous.** A static stream with a pull toward the target; the probes start at different distances (4.0 s and 3.7 s to the target). Only the difference of the delays matters: the on-ramp. It teaches "a probe arrives *when it leaves plus how long it takes*" and the timeline, and nothing else.

**Level 2, Windows.** A uniform stream (2.0) with a pulsing repeller beside each probe's lane (strength 1.65, width 0.8, 0.7 off the lane, one beside each, in phase). A probe is lifted into the target only if it passes its source while it pushes the right way: each probe reaches the target for 37% of releases (a window of about 4.4 s of the 12 s cycle). The probes leave at different places, so their arrival windows are offset; they must be matched. The first probe is not free: 16% of its good releases have no partner.

**Level 3, Order.** The same windows with the second source 0.5 rad out of phase, and the condition is "B 1.0 to 2.2 s after A". A relationship other than "together": an order with a gap. About half of the first probe's good releases have a partner (anchor 51%).

**Level 4, Meet, Then Part.** A speed-pulsing stream (x-speed 1.0 + 1.2 * 0.8 sin(wt), a cross-stream pulse, a pull toward the target). The probes must meet at a gate within 1.0 s and then reach the target 0.4 to 1.2 s apart. Each condition prunes the other: the gate alone admits 201 combinations, the order alone 1,551, both 86. Shift-break 100%: success depends on absolute phase everywhere.

**Level 5, Chain.** Three probes in the same field; the middle probe belongs to both pairs. 2,136 of 1,771,561 combinations win (0.12%); 21% of the first probe's releases have a partner.

## 5. Interface

The timeline strip under the play area is the shared clock (0 to 22 s; long ticks mark each cycle of the field). Each probe has a lane with a **release handle** the player drags (or selects with `1`-`3` and moves with `Left` / `Right`, Shift for 0.5 s; the panel has `-` / `+` buttons for 0.1 s). `Space` launches; the field arrows animate on the same clock; a launch plays back at 3x speed with each probe leaving at its own delay. After the playback the strip shows each probe's flight bar and its arrival time, and, on the lane of the probe that must be compared, a **band** showing where its arrival would have been allowed *given the other probe's actual arrival* (green if it is inside, red if not): early or late is visible, but the band moves with the flight that happened and is not a solution. The panel says it in words: "Probe B reached the target 0.8 s after probe A; allowed within 0.6 s: 0.2 s too late." Nothing is flown before a launch (a test reads the source of `sync_frame` and refuses any call that flies a probe except the Launch button).

Not verified: the front end has not been looked at on a screen in this work (the code compiles and its logic is tested; there was no display), so spacing, the timeline strip and the nine-card menu need an eyeball pass.

## 6. Validation

All from the real simulation. Checks and where they are tested (`tests/sync.rs`):

- **Solvable:** every level has winning combinations (`synchronize_is_a_chapter_with_five_validated_levels`); the cached search agrees with flying the probes together on random and winning combinations (`the_search_table_agrees_with_flying_the_probes_together`).
- **Meaningful challenge:** blind win share is bounded per level and every level has at least 20 winning combinations (`blind_timing_is_neither_trivial_nor_a_needle`). The ranges are not copied from Position (a pair or triple has a smaller random win share by construction): level 1 is a single 0.8 s band on a 12 s range (6%), the others join two or three constraints.
- **Coordination necessity:** every condition relates two different probes, every probe is in a condition, and the conditions connect the probes into one group (`every_condition_relates_two_different_probes_and_every_probe_is_in_one`); in levels 2 and 3 solving each probe separately is far worse than looking at both (`levels_two_and_three_cannot_be_solved_by_solving_each_probe_on_its_own`); in levels 4 and 5 each condition alone is at least 1.5x easier than both together (`..._conditions_that_pull_against_each_other`).
- **Timing sensitivity:** the width of a winning run is small (`the_synchronization_window_is_a_real_constraint`); moving a probe of a winning pair by 1 s flips the result.
- **Robustness:** every level keeps winning combinations when the windows are 0.2 s narrower or wider and when the starts move 0.25 in any direction (`levels_survive_moved_starts_and_changed_windows`). *This test found a real fault:* the first versions of levels 2 and 3 (target radius 0.7, sources 0.5 off the lane) had no solution when both starts moved 0.25 across the stream; the geometry was rebuilt with margin (target 0.9, sources 0.7 off).
- **Blind versus deliberate:** `coordination_beats_independent_choices_and_both_beat_guessing_in_every_periodic_level`: in levels 2 and 3, blind < Tempo-only < anchor (2.0% < 14.7% < 84%; 1.3% < 9.1% < 51%).
- **Which levels are arithmetic:** `level_one_is_arithmetic_and_says_so` asserts shift-break 0%, anchor 100% and Tempo-only = blind for level 1, and `levels_two_to_five_depend_on_when_each_probe_leaves_not_only_on_the_difference` asserts the opposite for the others, so the distinction claims in this document cannot silently go stale.

## 7. The critical design tests (honest answers)

1. **Can a player explain the relationship they are trying to establish?** Yes, in every level: "B must leave 0.3 s after A" (1); "find an arrival time both can reach" (2); "B 1 to 2 s behind A" (3); "together at the gate, apart at the target" (4); "A with B, B with C" (5).
2. **Fundamentally coordination rather than independent timing?** Level 1: no (a subtraction). Levels 2-3: partly (a shared arrival time must lie in both probes' possible arrival windows). Levels 4-5: yes (conditions on the same delays pull against each other). Even then, each probe by itself is a Tempo problem; what is new is the shared constraint.
3. **Must the player consider several trajectories together?** In levels 2-5 yes: the anchor measure (13-84% of the first probe's good releases have a partner) means choosing one probe's release without looking at the other often wastes it.
4. **A decision Tempo cannot express?** The relationship (matching, ordering and separating arrival times of different probes) is not expressible with one probe. Each probe's own timing is Tempo's decision.
5. **Interesting once the simulation is understood?** Less than it looks. A player who knows every probe's arrival time as a function of release has a lookup problem (find an anchor, match the rest; in level 5, fix the middle probe and fit the others). This is also true of Tempo, but Tempo's single-probe table is small and Synchronize's relationship has more structure to discover, not more depth to play.
6. **Several levels without only changing tolerances?** Yes in kind: static arithmetic; matching windows; an ordered gap; a gate-then-target conflict; a three-probe chain. Levels 2 and 3 share a field and differ in the relationship (simultaneous versus ordered).

## 8. Where the mechanic is weak

1. **The plain rendezvous is not distinct from Tempo** (section 3). Level 1 is kept as an on-ramp and is labelled as arithmetic by its own test.
2. **The probes still do not interact.** Coordination is joint constraint satisfaction over independent probes; "Tempo with a shared deadline" is a fair description of levels 2-3. A stronger model would let probes affect each other (see section 9).
3. **Level 2's coordination is mild** (84% of the first probe's good releases have a partner).
4. **Levels 4-5 are numerically hard to find by guessing and structurally a search.** A fully informed player's remaining work is computation, not insight.
5. **Level 4's shape came from a failure.** The "two meetings" idea in the brief is redundant in a deterministic flow (section 3); the shipped version replaces it.
6. **Delays are limited to one cycle (12 s).** A longer range would let any probe wait a full period, turning the phase constraint into a mod-12 matching; that was not explored.
7. **Not playtested, and the front end was not seen on a screen.**

## 9. A stronger interaction model (not built)

If Synchronize is to be a fundamental rather than a prototype, probes should affect each other. The smallest model that keeps the engine unchanged in spirit: **one-way coupling**, where probe A carries a small source (a moving Gauss term following A's recorded path, one new `Term`) that deflects probe B. Then B's *path*, not only its arrival time, depends on when A passes: the relative delay decides whether and how the two interact, and the interaction changes the trajectory the other probe takes. Two probes at one place no longer simply share a future by symmetry, because only one of them is a source. This is a new engine concept (a term that follows a recorded trajectory) and was out of scope for this prototype.

## 10. Tests

`tests/sync.rs` (22 tests): the chapter and its isolation from the others, the design rules (relations between different probes, one group), `entry_time`, the condition arithmetic (`offset`, `miss`, `ok`), delays as field clock (static versus periodic), the game (snapping, launching, a kept win, early/late in the right direction, a missed goal), the search against the real joint flight, the blind range, timing sensitivity, the arithmetic/not-arithmetic split, the Tempo-only and anchor comparisons, each condition mattering, robustness to moved starts and changed windows, and the two structural guards on the front end (no flight except the Launch button; the verdict states early/late but never a delay to choose). The generic suites cover Synchronize too (`every_level_is_solvable`, `simulation_is_deterministic_for_every_level`, the catalog test); the other chapters' suites are unchanged.
