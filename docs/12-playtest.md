# Playtest sheet for the combination levels

Nobody has played the levels listed here, and none of their screens has been looked at on a real display. Every claim in `docs/07-combinations.md` comes from the simulation and from reader models that stand in for a player. This sheet says what to look for, so that a playtest can confirm or contradict those claims. Fill in one row per level and per screen.

Run the game with `cargo run --release` and open the Combinations chapter. Levels unlock in order, so play them in the order listed.

## What to record for each level

| field | what to write |
|---|---|
| found the informed strategy | yes / no. Did the player work out what the level wants (scout, then cancel, then choose; or scan the dial settings) without being told? |
| attempts to the first win | how many attempts (scout levels) or launches (the others) until the first success |
| time to the first win | minutes from opening the level |
| solved by guessing | could the level be won by trying options without reading any evidence? If so, how many tries did it take? |
| where the player was lost | the screen, the control or the sentence that confused them |
| hints | which hint they opened, and whether it helped |

## What would count as a problem

- A level won by guessing in a handful of tries. The scout levels need three right choices in a row, so a random guesser among eight options should almost never win; a dial or slot level with 13 settings, of which 3 win, can be won by scanning the dial, and that is intended. What is not intended is a level where the second or third try at random always works.
- A player who reads the evidence correctly and still cannot tell which option to choose (the reader models claim the evidence is enough, with an estimate error of 0.3).
- A control that does nothing the player can see (a dial that changes no arrow, a slot that changes no flight).
- A hint that states the answer, or one that is wrong for the level.

## Screens to check on a real display

| screen | check |
|---|---|
| Combinations select screen (27 levels) | Two columns under five headings. The left column has 14 rows and the right one 13; in both columns the objective line is dropped because the rows are under 40 px. Nothing overlaps the Back button, and titles are cut with "..." only when they are too long. |
| Scout levels with several choices (`cmb-20`, `cmb-21`, `cmb-25`) | One group of numbered buttons per choice, a Commit button, and the result text. Nothing is cut off at the bottom of the panel. The selected buttons are outlined. |
| Scout levels with one choice (`cmb-14`, `cmb-15`, `cmb-17`, `cmb-19`) | The numbered release slots or the source sites are drawn on the field and match the buttons. Keys 1 to 9 pick an option. |
| Dial rows (`cmb-22`, `cmb-23`, `cmb-24`) | 13 numbered buttons in rows (5 per row on the Steer screen, 7 on the Synchronize and Tempo screens); the field's arrows change when the dial changes; the Steer dial is locked after the first beat. |
| Drone release row (`cmb-28`) | 25 buttons labelled with the release in seconds, in rows of 7; the drone's path and "A arrives X s" label change with the release; the buttons are locked once the first beat is played. |
| Start slots (`cmb-10`) | Five numbered markers on the field; clicking a marker or a number moves probe B's start. |

## Levels

| level | pair | what a good play looks like |
|---|---|---|
| `cmb-10` Route and Rendezvous | Synchronize + Position | Release both probes from the first marker and read the timeline: no delays work. Try the other markers. |
| `cmb-14` Scout, Then Release | Survey + Position | Fly one or two scouts, find the source from the push arrows, release from a slot on the side that bends the probe back. |
| `cmb-15` Cancel It | Survey + Shape | Read the kind from the arrows, add the opposite kind at the same place. |
| `cmb-16`, `cmb-18` | Survey + Predict | Locate the source, picture the probe's flight, call the exit. `cmb-18` has a swirl in the background. |
| `cmb-17` Scout, Then Steer | Survey + Steer | Locate the source, choose the gust plan that cancels its push. |
| `cmb-19` Scout, Then Time It | Survey + Tempo | Scouts fly from phase 0; choose the launch phase that lets the pulse and the source carry the probe to the target. |
| `cmb-29` Scout, Then Time It II | Survey + Tempo | As `cmb-19`, but the sources are of both kinds and the launch phases are half a second apart: read the kind from the arrows, then aim at the middle of a stretch of good phases. |
| `cmb-20` Cancel and Launch | Survey + Shape + Position | Scout and cancel as in `cmb-15`, then choose a release slot that the visible swirl carries to the target. |
| `cmb-21` Cancel and Steer | Survey + Shape + Steer | Scout and cancel, then choose the gust plan that corrects the visible flight. |
| `cmb-25` Cancel and Dial | Survey + Shape + Experiment | Scout and cancel, then choose the dial setting that carries the probe onto the target. |
| `cmb-22` Set the Dial, Then Steer | Steer + Experiment | Scan the dial settings, watching the arrows and a gust-free run, then spend the single gust. |
| `cmb-23` Dial and Rendezvous | Synchronize + Experiment | Find the dial setting at which the two arrival windows come within reach, then line them up with the delays. |
| `cmb-28` Meet the Drone | Steer + Synchronize | Compare when the drone arrives with when your probe would arrive with no gust, choose the release that brings them closest, then spend the gust. |
| `cmb-24` Dial and Time It | Tempo + Experiment | Scrub the clock at each dial setting and look for a stretch of the cycle that reaches the target. This level has only five settings and does not meet the protocol. |

## Answer key (spoilers)

Dial and slot levels. Buttons are numbered in the order the screen lists them; the dial list is 1.0 (the default) first and then the other values in ascending order.

| level | winning buttons |
|---|---|
| `cmb-10` | slots 2 and 3 (with suitable delays) |
| `cmb-22` | dial buttons 3, 4 and 5 (values 0.2, 0.4 and 0.6) |
| `cmb-23` | dial buttons 3, 4 and 5 (values 0.2, 0.4 and 0.6) |
| `cmb-24` | dial buttons 4 and 5 (values 1.5 and 2.0) |
| `cmb-28` | releases 0.5, 1.0 and 1.5 s (with a suitable gust; at 1.0 s only one gust plan works) |

Scout levels. The hidden source changes every attempt and is never the one just used. For each possible source, the options that win (labels as on the screen: a number, an exit letter, or "source-slot", "source-plan" or "source-dial" for the multi-step levels, where sources 1 to 4 are repellers and 5 to 8 attractors at the same four places).

| level | hidden source | winning options |
|---|---|---|
| `cmb-14` | attractor at (-1.40, 2.48) | slot 3 or 4 |
| | attractor at (-2.28, -1.02) | slot 2 |
| | attractor at (-0.57, -2.33) | slot 1 or 5 |
| `cmb-15` | attractor at (-2.17, 1.26) | source 1 |
| | repeller at (0.59, 0.63) | source 7 or 8 |
| | attractor at (2.22, 0.56) | source 3 or 4 |
| `cmb-16` | repeller at (-0.38, -1.43) | exit B |
| | attractor at (-2.06, 1.64) | exit A |
| | attractor at (0.91, -0.68) | exit C |
| `cmb-17` | attractor at (0.48, -1.83) | plan 1 |
| | attractor at (0.52, 1.24) | plan 2 |
| | repeller at (1.71, -1.47) | plan 4 |
| `cmb-18` | attractor at (1.10, -0.58) | exit D |
| | repeller at (1.16, -1.87) | exit C |
| | repeller at (-1.40, 0.55) | exit B |
| `cmb-19` | attractor at (-1.84, 2.21) | phase 4 (6 s) |
| | attractor at (-1.81, 0.61) | phase 2 (2 s) |
| | attractor at (-2.68, -2.06) | phase 3 (4 s) |
| `cmb-29` | repeller at (-2.00, 1.00) | options 2 to 5 (0.5 to 2.0 s) |
| | repeller at (2.00, -2.00) | options 20 to 24 (9.5 to 11.5 s) |
| | attractor at (-3.00, 0.00) | options 12 to 15 (5.5 to 7.0 s) |
| `cmb-20` | attractor at (-2.29, 1.20) | 1-4, 1-5, 1-6 or 3-3 |
| | repeller at (-0.88, 0.28) | 6-4, 6-5 or 6-6 |
| | attractor at (1.96, 1.40) | 4-4, 4-5 or 4-6 |
| `cmb-21` | repeller at (-2.13, -1.45) | 5-3, 5-4 or 6-3 |
| | repeller at (0.48, -1.51) | 7-3 or 7-4 |
| | attractor at (2.33, 0.00) | 4-3 or 4-4 |
| `cmb-25` | attractor at (-1.06, 0.73) | 2-2, 2-3 or 4-4 |
| | repeller at (0.82, 0.11) | 7-2 or 7-3 |
| | attractor at (1.89, -1.74) | 4-2 or 4-3 |

In `cmb-20`, `cmb-21` and `cmb-25` the second part of each winning option is nearly the same for every source (the visible field decides it; `cmb-20` has one exception, 3-3, and `cmb-25` one, 4-4), and only the first part, the cancelling source, depends on the scouting. If a playtester finds the second part by trial in the first attempt and then only has to scout, that is the expected play, and the level then tests one decision of the three. Whether that is acceptable is a design question for the owner of the plan.
