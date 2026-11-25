# Level catalog

Generated from `src/levels.rs` by `src/catalog.rs`; do not edit by hand. Regenerate with
`cargo run --release --bin catalog > docs/LEVEL_CATALOG.md` (a test fails if this file is out of date).

World coordinates: x to the right, y up. Every level uses the bounds x in [-6, 6], y in [-4, 4]. Region indices are positions in the level's region list. Levels are listed in play order.

## pos-1 - Open Water (Position)

- objective: Drop the probe anywhere in the zone so it reaches the target.
- hint 1: The target pulls everything toward it. Arrows near it all point inward.
- hint 2: Almost any start works here. Watch where the arrows lead.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 40 s; seed: 307257169
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - uniform flow (0.6, 0)
  2. fixed - radial about (4, 0.5): v = k d / (|d|^2 + soft^2), k = -2.4, soft = 0.6 (attracts)
- regions:
  0. target 'T' centre (4, 0.5) radius 0.6
- rules:
  - mechanic: **Position**
  - release zone 1: rectangle x in [-5.5, -3.8], y in [-3, 3]; its probe must reach region 0 (T)
  - launch phase: fixed (0)
  - intensity: fixed (1)

## pos-2 - Two Roads (Position)

- objective: The vortex splits the flow. Find a start that ends at the target.
- hint 1: Clockwise vortex: above it the flow runs right, below it the flow runs left.
- hint 2: Starts above and below the centre line take very different roads.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 40 s; seed: 307257170
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - uniform flow (0.7, 0)
  2. fixed - vortex about (0, 0): v = k perp(d) / (|d|^2 + soft^2), k = -2.5, soft = 0.8 (clockwise)
- regions:
  0. target 'T' centre (4, -2) radius 0.6
- rules:
  - mechanic: **Position**
  - release zone 1: rectangle x in [-5.5, -4], y in [-3, 3]; its probe must reach region 0 (T)
  - launch phase: fixed (0)
  - intensity: fixed (1)

## pos-3 - Saddle Point (Position)

- objective: Only part of the zone can reach the target. Which part?
- hint 1: Flow squeezes in along the horizontal axis and is flung out vertically.
- hint 2: Probes above the axis are thrown up, those below are thrown down.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 40 s; seed: 307257171
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - saddle at (4, 0): v = k(-dx, +dy), k = 0.5 (inflow along x, outflow along y)
- regions:
  0. target 'T' centre (3, 2.5) radius 0.8
- rules:
  - mechanic: **Position**
  - release zone 1: circle centre (-4.2, 0) radius 1.8; its probe must reach region 0 (T)
  - launch phase: fixed (0)
  - intensity: fixed (1)

## pos-4 - Mirror Mirror (Position)

- objective: The field is symmetric. The goal is not. Avoid the hazard.
- hint 1: The two vortices are mirror images, so a path and its mirror path look equally good.
- hint 2: The mirror path of a winning start ends in the hazard.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 40 s; seed: 307257172
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - uniform flow (0.5, 0)
  2. fixed - vortex about (0, 1.6): v = k perp(d) / (|d|^2 + soft^2), k = 1.5, soft = 0.7 (counter-clockwise)
  3. fixed - vortex about (0, -1.6): v = k perp(d) / (|d|^2 + soft^2), k = -1.5, soft = 0.7 (clockwise)
- regions:
  0. target 'T' centre (4.5, 1.2) radius 0.6
  1. hazard 'X' centre (4.5, -1.2) radius 0.6
- rules:
  - mechanic: **Position**
  - release zone 1: rectangle x in [-5.5, -4], y in [-3, 3]; its probe must reach region 0 (T)
  - launch phase: fixed (0)
  - intensity: fixed (1)

## pos-5 - The Needle (Position)

- objective: A tiny target and a sensitive field. Aim carefully.
- hint 1: Near a saddle, tiny differences in the start grow quickly.
- hint 2: Use the arrow keys for fine adjustments of the release point.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 40 s; seed: 307257173
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - saddle at (5, 0): v = k(-dx, +dy), k = 0.7 (inflow along x, outflow along y)
- regions:
  0. target 'T' centre (3.8, 1.6) radius 0.3
- rules:
  - mechanic: **Position**
  - release zone 1: circle centre (-4.5, 0) radius 1.2; its probe must reach region 0 (T)
  - launch phase: fixed (0)
  - intensity: fixed (1)

## pre-1 - Straight Shot (Predict)

- objective: Which exit will the probe leave through?
- hint 1: A uniform flow moves everything in one direction at one speed.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 40 s; seed: 307333088
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - uniform flow (1, 0)
- regions:
  0. exit 'A' centre (5.2, 3) radius 0.85
  1. exit 'B' centre (5.2, 1) radius 0.85
  2. exit 'C' centre (5.2, -1) radius 0.85
  3. exit 'D' centre (5.2, -3) radius 0.85
- rules:
  - mechanic: **Predict**
  - start zone (release points are drawn from it): rectangle x in [-5.5, -4], y in [-3, 3]
  - rounds: 3; correct answers needed: 2
  - reveal before commit: nothing

## pre-2 - Gentle Curve (Predict)

- objective: The flow bends. Follow it.
- hint 1: A distant vortex adds a steady sideways push that bends every path.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 40 s; seed: 307333089
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - uniform flow (1, 0)
  2. fixed - vortex about (-1, -5): v = k perp(d) / (|d|^2 + soft^2), k = -4, soft = 1 (clockwise)
- regions:
  0. exit 'A' centre (5.2, 3) radius 0.85
  1. exit 'B' centre (5.2, 1) radius 0.85
  2. exit 'C' centre (5.2, -1) radius 0.85
  3. exit 'D' centre (5.2, -3) radius 0.85
- rules:
  - mechanic: **Predict**
  - start zone (release points are drawn from it): rectangle x in [-5.5, -4], y in [-3, 3]
  - rounds: 3; correct answers needed: 2
  - reveal before commit: nothing

## pre-3 - Spiral Out (Predict)

- objective: Probes spiral around the centre. Where do they escape?
- hint 1: The vortex turns the probe while the outward push makes it spiral outward.
- hint 2: The turning direction tells you which exit comes first.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 40 s; seed: 307333090
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - vortex about (0, 0): v = k perp(d) / (|d|^2 + soft^2), k = 2, soft = 0.6 (counter-clockwise)
  2. fixed - radial about (0, 0): v = k d / (|d|^2 + soft^2), k = 0.8, soft = 0.6 (repels)
- regions:
  0. exit 'A' centre (5.6, 3.6) radius 2.2
  1. exit 'B' centre (5.6, -3.6) radius 2.2
  2. exit 'C' centre (-5.6, -3.6) radius 2.2
  3. exit 'D' centre (-5.6, 3.6) radius 2.2
- rules:
  - mechanic: **Predict**
  - start zone (release points are drawn from it): circle centre (0, 0) radius 2.2
  - rounds: 3; correct answers needed: 2
  - reveal before commit: nothing

## pre-4 - False Lead (Predict)

- objective: The arrows near the start point at one exit. Think further ahead.
- hint 1: Do not trust the first arrows. A strong vortex waits halfway across.
- hint 2: Where the probe meets the vortex decides which way it is carried.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 40 s; seed: 307333091
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - uniform flow (0.9, 0)
  2. fixed - vortex about (1, 0): v = k perp(d) / (|d|^2 + soft^2), k = 3, soft = 0.5 (counter-clockwise)
- regions:
  0. exit 'A' centre (5.2, 3) radius 0.85
  1. exit 'B' centre (5.2, 1) radius 0.85
  2. exit 'C' centre (5.2, -1) radius 0.85
  3. exit 'D' centre (5.2, -3) radius 0.85
- rules:
  - mechanic: **Predict**
  - start zone (release points are drawn from it): rectangle x in [-5.5, -4], y in [-3, 3]
  - rounds: 3; correct answers needed: 2
  - reveal before commit: nothing

## pre-5 - Crosswinds (Predict)

- objective: Several features act in turn. Extrapolate step by step.
- hint 1: Break the path into stages: which feature does the probe meet first, and where does it leave that feature?
- hint 2: A pair of opposite vortices near the centre acts like a jet. The repeller on the right sends what reaches it outward.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 40 s; seed: 307333092
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - uniform flow (0.06, 0)
  2. fixed - vortex about (-1.17, -0.58): v = k perp(d) / (|d|^2 + soft^2), k = 2.52, soft = 0.6 (counter-clockwise)
  3. fixed - vortex about (-0.86, -0.99): v = k perp(d) / (|d|^2 + soft^2), k = -1.34, soft = 0.6 (clockwise)
  4. fixed - radial about (1.03, 0.55): v = k d / (|d|^2 + soft^2), k = 1.04, soft = 0.6 (repels)
- regions:
  0. exit 'A' centre (5.6, 3.6) radius 2.2
  1. exit 'B' centre (5.6, -3.6) radius 2.2
  2. exit 'C' centre (-5.6, -3.6) radius 2.2
  3. exit 'D' centre (-5.6, 3.6) radius 2.2
- rules:
  - mechanic: **Predict**
  - start zone (release points are drawn from it): circle centre (0, 0) radius 2.8
  - rounds: 3; correct answers needed: 2
  - reveal before commit: nothing

## tem-1 - Two Moods (Tempo)

- objective: The field has two phases. Launch in the right one.
- hint 1: For half the cycle the field lifts the probe, for the other half it pushes it down.
- hint 2: Watch the timeline while the field plays.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 40 s; seed: 310647577
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - uniform flow (3, 0)
  2. fixed - custom square wave: v = (0, tanh(4 sin(2 pi t / 12)))
- regions:
  0. target 'T' centre (4.5, 3) radius 0.9
- rules:
  - mechanic: **Tempo**
  - fixed start (-4.5, 0)
  - field period 12 s
  - goal region 0 (T)

## tem-2 - Slow Tide (Tempo)

- objective: A broad window of good launch times.
- hint 1: The vertical push swings smoothly up and down. The probe feels the sum over its whole trip.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 40 s; seed: 310647578
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - uniform flow (2.5, 0)
  2. fixed - modulated: [uniform flow (0, 1)] x (0 + 1 sin(0.5235987755982988 t + 0))
- regions:
  0. target 'T' centre (4.5, 2.6) radius 0.9
- rules:
  - mechanic: **Tempo**
  - fixed start (-4.5, 0)
  - field period 12 s
  - goal region 0 (T)

## tem-3 - Knife Edge (Tempo)

- objective: Only a narrow window works.
- hint 1: The probe must feel equal up and down pushes. That balance happens at very particular phases.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 40 s; seed: 310647579
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - uniform flow (2.5, 0)
  2. fixed - modulated: [uniform flow (0, 1)] x (0 + 1 sin(0.5235987755982988 t + 0))
- regions:
  0. target 'T' centre (4.5, 0) radius 0.4
- rules:
  - mechanic: **Tempo**
  - fixed start (-4.5, 0)
  - field period 12 s
  - goal region 0 (T)

## tem-4 - Double Beat (Tempo)

- objective: The field repeats twice per cycle: several windows exist.
- hint 1: Two full swings fit in one cycle, so the same outcome can be reached at more than one phase.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 40 s; seed: 310647580
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - uniform flow (2.5, 0)
  2. fixed - modulated: [uniform flow (0, 1.6)] x (0 + 1 sin(1.0471975511965976 t + 0))
- regions:
  0. target 'T' centre (4.5, 2) radius 0.7
- rules:
  - mechanic: **Tempo**
  - fixed start (-4.5, 0)
  - field period 12 s
  - goal region 0 (T)

## tem-5 - Moving Vortex (Tempo)

- objective: A vortex circles the centre. Time your pass.
- hint 1: Where the vortex is when the probe passes decides whether it is lifted or dropped.
- hint 2: The vortex travels one lap per cycle.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 40 s; seed: 310647581
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - uniform flow (2.5, 0)
  2. fixed - orbiting vortex source: centre (0, 0) + 2 (cos(a), sin(a)), a = 0.5235987755982988 t + 0; k = 3, soft = 0.6
- regions:
  0. target 'T' centre (4.5, 2) radius 0.7
- rules:
  - mechanic: **Tempo**
  - fixed start (-4.5, 0)
  - field period 12 s
  - goal region 0 (T)

## exp-1 - Which Way? (Experiment)

- objective: Learn how intensity moves the exit, then report the rule.
- hint 1: Run several intensities and keep the runs visible: compare where each one leaves.
- hint 2: Look at the order of the exits as the number grows.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 40 s; seed: 297363674
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - radial about (0, 0): v = k d / (|d|^2 + soft^2), k = 1.2, soft = 0.6 (repels)
  2. tunable - vortex about (0, 0): v = k perp(d) / (|d|^2 + soft^2), k = -1, soft = 0.6 (clockwise)
  (default intensity 1)
- regions:
  0. exit 'A' centre (5.6, 3.6) radius 2.2
  1. exit 'B' centre (5.6, -3.6) radius 2.2
  2. exit 'C' centre (-5.6, -3.6) radius 2.2
  3. exit 'D' centre (-5.6, 3.6) radius 2.2
- rules:
  - mechanic: **Experiment**
  - fixed start (1.4, 0)
  - intensity range [0, 2]
  - challenge: ExitShift about (0, 0): clockwise / counter-clockwise / neither

## exp-2 - The Switch (Experiment)

- objective: Find the intensity where the exit changes.
- hint 1: Find one value for each exit, then narrow the gap between them like a search.
- hint 2: Preserved runs show which side of the switch you are on.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 40 s; seed: 297363675
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - uniform flow (1.5, 0)
  2. tunable - vortex about (0, 0): v = k perp(d) / (|d|^2 + soft^2), k = 2, soft = 0.8 (counter-clockwise)
  (default intensity 1)
- regions:
  0. exit 'A' centre (4, 2) radius 2
  1. exit 'B' centre (4, -2) radius 2
- rules:
  - mechanic: **Experiment**
  - fixed start (-4.5, 0.5)
  - intensity range [0, 3]
  - challenge: Threshold: intensity where exit A becomes exit B, tolerance +/-0.15

## exp-3 - Round Trip (Experiment)

- objective: Does the probe ever come back to an earlier exit?
- hint 1: Sweep the whole range and note the exits in order.
- hint 2: The exit keeps moving around the ring as the vortex strengthens.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 40 s; seed: 297363676
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - radial about (0, 0): v = k d / (|d|^2 + soft^2), k = 1.2, soft = 0.6 (repels)
  2. tunable - vortex about (0, 0): v = k perp(d) / (|d|^2 + soft^2), k = -1, soft = 0.6 (clockwise)
  (default intensity 1)
- regions:
  0. exit 'A' centre (5.6, 3.6) radius 2.2
  1. exit 'B' centre (5.6, -3.6) radius 2.2
  2. exit 'C' centre (-5.6, -3.6) radius 2.2
  3. exit 'D' centre (-5.6, 3.6) radius 2.2
- rules:
  - mechanic: **Experiment**
  - fixed start (1.4, 0)
  - intensity range [0, 8]
  - challenge: Return: does the exit sequence revisit an exit?

## exp-4 - Beyond the Rig (Experiment)

- objective: The test rig stops at intensity 4. Predict what happens at 6.2.
- hint 1: You cannot test 6.2, so look for a rule. Measure how far each exit sits from the last one as intensity grows.
- hint 2: The winding angle grows steadily with intensity. A full extra turn brings the probe back to the same exit.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 40 s; seed: 297363677
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - radial about (0, 0): v = k d / (|d|^2 + soft^2), k = 1.2, soft = 0.6 (repels)
  2. tunable - vortex about (0, 0): v = k perp(d) / (|d|^2 + soft^2), k = -1, soft = 0.6 (clockwise)
  (default intensity 1)
- regions:
  0. exit 'A' centre (5.6, 3.6) radius 2.2
  1. exit 'B' centre (5.6, -3.6) radius 2.2
  2. exit 'C' centre (-5.6, -3.6) radius 2.2
  3. exit 'D' centre (-5.6, 3.6) radius 2.2
- rules:
  - mechanic: **Experiment**
  - fixed start (1.4, 0)
  - intensity range [0, 8]
  - challenge: Extrapolate: rig runs up to intensity 4, question asks about intensity 6.2

## exp-5 - The Window (Experiment)

- objective: Find the interval that sends the probe through exit D.
- hint 1: Exit D is reached for a band of intensities, not a single value.
- hint 2: Find both edges of the band.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 40 s; seed: 297363678
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - radial about (0, 0): v = k d / (|d|^2 + soft^2), k = 1.2, soft = 0.6 (repels)
  2. tunable - vortex about (0, 0): v = k perp(d) / (|d|^2 + soft^2), k = -1, soft = 0.6 (clockwise)
  (default intensity 1)
- regions:
  0. exit 'A' centre (5.6, 3.6) radius 2.2
  1. exit 'B' centre (5.6, -3.6) radius 2.2
  2. exit 'C' centre (-5.6, -3.6) radius 2.2
  3. exit 'D' centre (-5.6, 3.6) radius 2.2
- rules:
  - mechanic: **Experiment**
  - fixed start (1.4, 0)
  - intensity range [0, 5]
  - challenge: Interval: both ends of the intensity band leaving through exit D, tolerance +/-0.2

## ste-1 - First Gust (Steer)

- objective: Nudge the probe so it reaches the target. A gust changes its velocity for one beat.
- hint 1: Left alone, the probe drifts past just below the target.
- hint 2: A gust adds to the velocity for one beat only. The field carries the probe on afterwards.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 40 s; seed: 310163233
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - uniform flow (1.6, 0.05)
- regions:
  0. target 'T' centre (2.9, 0.1) radius 0.5
- rules:
  - mechanic: **Steer**
  - fixed start (-5, -0.9)
  - beats 5; gust budget 1; gust strength 1; beat length 1 s
  - gust directions: 8 (E, NE, N, NW, W, SW, S, SE)
  - goal region 0 (T)
  - launch phase: fixed (0)
  - start point: fixed
  - sources: none
  - companion drone: none
  - dial: not offered

## ste-2 - Leverage (Steer)

- objective: The same gust matters more in some places than others. Find where.
- hint 1: Watch how the arrows spread apart around the middle of the field: offsets grow there.
- hint 2: A small push before the spreading zone is magnified. The same push afterwards is not.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 40 s; seed: 310163234
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - custom leverage field: dz = exp(-(x/1.5)^2); k = 1.3 dz - 0.35 (1 - dz) [x > 0]; v = (1.4, k y)
- regions:
  0. target 'T' centre (3.2, -1.6) radius 0.5
- rules:
  - mechanic: **Steer**
  - fixed start (-5, 0.15)
  - beats 6; gust budget 2; gust strength 0.35; beat length 1 s
  - gust directions: 8 (E, NE, N, NW, W, SW, S, SE)
  - goal region 0 (T)
  - launch phase: fixed (0)
  - start point: fixed
  - sources: none
  - companion drone: none
  - dial: not offered

## ste-3 - Tipping Point (Steer)

- objective: A saddle splits the flow. Cross to the other side in time.
- hint 1: Above the centre line the flow is thrown up, below it down. The probe starts just above it.
- hint 2: The dividing line is close now. Every beat the probe drifts further from it.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 40 s; seed: 310163235
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - uniform flow (1.6, 0)
  2. fixed - saddle at (-0.7, 0): v = k(-dx, +dy), k = 0.27 (inflow along x, outflow along y)
- regions:
  0. target 'T' centre (2.62, -1.41) radius 0.7
- rules:
  - mechanic: **Steer**
  - fixed start (-5, 0.3)
  - beats 6; gust budget 1; gust strength 1; beat length 1 s
  - gust directions: 8 (E, NE, N, NW, W, SW, S, SE)
  - goal region 0 (T)
  - launch phase: fixed (0)
  - start point: fixed
  - sources: none
  - companion drone: none
  - dial: not offered

## ste-4 - The Long Way (Steer)

- objective: The flow circles the centre. Pushing straight at the target will not get you there.
- hint 1: The flow carries the probe around the centre, anticlockwise, and slowly inward.
- hint 2: The target is on a wider orbit than the probe. Which way does the probe need to move to join it?
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 40 s; seed: 310163236
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - vortex about (0, 0): v = k perp(d) / (|d|^2 + soft^2), k = 8.5, soft = 0.8 (counter-clockwise)
  2. fixed - radial about (0, 0): v = k d / (|d|^2 + soft^2), k = -0.45, soft = 0.8 (attracts)
- regions:
  0. target 'T' centre (3.3, 0.3) radius 0.5
- rules:
  - mechanic: **Steer**
  - fixed start (-2.1, 0)
  - beats 7; gust budget 2; gust strength 1; beat length 1 s
  - gust directions: 8 (E, NE, N, NW, W, SW, S, SE)
  - goal region 0 (T)
  - launch phase: fixed (0)
  - start point: fixed
  - sources: none
  - companion drone: none
  - dial: not offered

## ste-5 - Set Up (Steer)

- objective: The first gust is for the second one. The target is small.
- hint 1: A good first move changes what the second move can do. It may not look like progress.
- hint 2: Do not just close the distance. Look for the flow that will do the work for you.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 40 s; seed: 310163237
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - uniform flow (0.65, 0)
  2. fixed - vortex about (-1.9, 0.43): v = k perp(d) / (|d|^2 + soft^2), k = 1.41, soft = 0.7 (counter-clockwise)
  3. fixed - vortex about (-0.06, 0.6): v = k perp(d) / (|d|^2 + soft^2), k = -1.21, soft = 0.7 (clockwise)
  4. fixed - radial about (1.71, -0.27): v = k d / (|d|^2 + soft^2), k = 1.37, soft = 0.7 (repels)
- regions:
  0. target 'T' centre (0.82, 1.79) radius 0.3
- rules:
  - mechanic: **Steer**
  - fixed start (-5, 0.94)
  - beats 8; gust budget 2; gust strength 1; beat length 1 s
  - gust directions: 8 (E, NE, N, NW, W, SW, S, SE)
  - goal region 0 (T)
  - launch phase: fixed (0)
  - start point: fixed
  - sources: none
  - companion drone: none
  - dial: not offered

## sur-1 - Something in the Stream (Survey)

- objective: Something hidden pushes the flow away from itself. Fly scouts, then mark where it is.
- hint 1: The grey arrows show the flow with nothing hidden. Where a scout's path bends away from them, something hidden is pushing it.
- hint 2: The small arrows on a trail show that push, and they point away from the source. Extend them backwards in your head, or follow the faint lines: where they cross is the source.
- hint 3: A scout only feels the source from about 2.4 away. If your first scouts felt nothing, release the next one nearer to where you suspect it.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 5 s; seed: 310205517
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - uniform flow (1, 0)
- regions:
- rules:
  - mechanic: **Survey**
  - the field layers above are the *charted background*; the hidden source is added to them and is not listed
  - hidden source: local repeller (Gauss term), strength k = 1.6, width sigma = 1, position unknown
  - scouts may be placed anywhere in rectangle x in [-5.5, 5.5], y in [-3.5, 3.5]; each leaves a 5 s trail
  - free scout flown at the start of every attempt from (-5, 0); 2 further scouts can be placed
  - one commit per attempt; correct within 0.75 of the source
  - estimate tests per attempt: 3 (each shows what the scouts would have done if the source were at the marker)
  - reading aid: faint rays back along each push arrow on
  - reading aid: shaded band where a source would already have been felt off
  - planning aid: preview of where a scout would fly under the charted background alone, shown while aiming off
  - the level is complete after 1 attempt(s) found; attempts use the positions below in order on the first pass, then reshuffled each pass
  - one source is hidden per attempt
  - hidden positions: (-2.4, 2), (-1.5, -1.9), (-3.25, -2.25), (-0.25, 2.25), (3.25, -1.75), (2.25, 2.25)

## sur-2 - Which Way It Pushes (Survey)

- objective: Something hidden pushes the flow. It may push away, pull in, or push sideways. Say which, and where.
- hint 1: Look at the small arrows around the spot where the trails bend. Pointing away from it: a repeller. Pointing toward it: an attractor. Going around it: a swirl. The key under the field shows all three.
- hint 2: The faint lines run along each arrow. For a repeller or an attractor they cross at the source. A swirl's arrows point sideways, so those lines never cross: press P to switch to lines at right angles, which cross at a swirl's centre.
- hint 3: Set the marker's kind with Y, then test it. A marker of the wrong kind never matches the dashes, even in the right place.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 5 s; seed: 310205518
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - uniform flow (1, 0)
- regions:
- rules:
  - mechanic: **Survey**
  - the field layers above are the *charted background*; the hidden source is added to them and is not listed
  - hidden source: a local repeller, attractor or swirl (Gauss terms), kind and position unknown and to be given by the marker; strength k = 1.3, width sigma = 1
  - scouts may be placed anywhere in rectangle x in [-5.5, 5.5], y in [-3.5, 3.5]; each leaves a 5 s trail
  - free scout flown at the start of every attempt from (-5, 0); 2 further scouts can be placed
  - one commit per attempt; correct within 0.75 of the source
  - estimate tests per attempt: 3 (each shows what the scouts would have done if the source were at the marker)
  - reading aid: faint rays back along each push arrow on
  - reading aid: shaded band where a source would already have been felt off
  - planning aid: preview of where a scout would fly under the charted background alone, shown while aiming off
  - the level is complete when a repeller, an attractor and a swirl have each been found; each attempt hides the first of those not yet found (so a miss gives another of the same kind), using the positions below in order
  - one source is hidden per attempt
  - hidden positions: repeller (-2.5, 1.2), attractor (-1.5, -1.3), swirl CCW (-3, 1.1), repeller (-2.6, -0.9), attractor (-2.2, 1.4), swirl CW (-1, -1.2), swirl CCW (-1.8, 0.4)

## sur-3 - One Look Is a Line (Survey)

- objective: Something hidden pushes the flow. One scout may only tell you which line it is on. No guide lines now.
- hint 1: Extend the small arrows: the source is somewhere along each of those lines (at right angles to them, if it is a swirl).
- hint 2: To find where along the line, send a scout from a bit above or below it. Its arrows give a different line, and where the lines cross is the source.
- hint 3: The shaded ground is where a source would already have been felt. It is not there.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 5 s; seed: 310205519
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - uniform flow (1, 0)
- regions:
- rules:
  - mechanic: **Survey**
  - the field layers above are the *charted background*; the hidden source is added to them and is not listed
  - hidden source: a local repeller, attractor or swirl (Gauss terms), kind and position unknown and to be given by the marker; strength k = 1.65, width sigma = 0.8
  - scouts may be placed anywhere in rectangle x in [-5.5, 5.5], y in [-3.5, 3.5]; each leaves a 5 s trail
  - free scout flown at the start of every attempt from (-5, 0); 2 further scouts can be placed
  - one commit per attempt; correct within 0.6 of the source
  - estimate tests per attempt: 3 (each shows what the scouts would have done if the source were at the marker)
  - reading aid: faint rays back along each push arrow off
  - reading aid: shaded band where a source would already have been felt on
  - planning aid: preview of where a scout would fly under the charted background alone, shown while aiming off
  - the level is complete after 3 attempt(s) found; attempts use the positions below in order on the first pass, then reshuffled each pass
  - one source is hidden per attempt
  - hidden positions: repeller (-1.5, 1.6), attractor (1.55, 0.05), swirl CCW (3, 3), swirl CW (0.5, -1.6), repeller (0.4, 1.5), attractor (2.5, -3.3)

## sur-4 - Two of Them (Survey)

- objective: Two sources are hidden, and from far away they can look like one. Mark both.
- hint 1: From far away two sources can look like one in the middle. If both markers sitting on one spot fit the dashes, that may be what you are seeing.
- hint 2: A scout that passes close to one source feels it almost alone, and one that flies along the line between them feels both in turn. Release one there.
- hint 3: Test your markers. If no pair of markers on one spot matches the dashes, pull them apart until it does.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 5 s; seed: 310205520
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - uniform flow (1, 0)
- regions:
- rules:
  - mechanic: **Survey**
  - the field layers above are the *charted background*; the hidden source is added to them and is not listed
  - hidden source: local repeller (Gauss term), strength k = 1.65, width sigma = 0.8, position unknown
  - scouts may be placed anywhere in rectangle x in [-5.5, 5.5], y in [-3.5, 3.5]; each leaves a 5 s trail
  - free scout flown at the start of every attempt from (-5, 0); 3 further scouts can be placed
  - one commit per attempt; correct within 0.6 of the source
  - estimate tests per attempt: 3 (each shows what the scouts would have done if the source were at the marker)
  - reading aid: faint rays back along each push arrow on
  - reading aid: shaded band where a source would already have been felt off
  - planning aid: preview of where a scout would fly under the charted background alone, shown while aiming off
  - the level is complete after 4 attempt(s) found; attempts use the positions below in order on the first pass, then reshuffled each pass
  - two sources are hidden at once (identical repellers); the player has two markers, and a commit is found when each marker is within the tolerance of a different source, in either assignment
  - hidden positions: (-3.7, 1.7) + (-0.75, -1.7), (-0.3, 1.7) + (-2.4, -1.7), (-0.3, 1.6) + (-3.5, 1.6), (0.2, -1.6) + (-2.6, -1.6), (0.3, 1.5) + (-2.1, 1.5), (-1.9, -1.5) + (-4.1, -1.5)

## sur-5 - The Gyre (Survey)

- objective: The flow circulates, so a scout curves as it flies. Something hidden pushes it. Release scouts where their paths will pass it.
- hint 1: The pale line that follows your cursor is where a scout released there would fly with nothing hidden. A scout only feels the source if that line passes within about 2 of it.
- hint 2: The shaded ground is where a source would already have been felt. Release the next scout so its pale line crosses ground that is not shaded: the part of a line the free scout already flew will show you nothing new.
- hint 3: The flow carries every scout round the same way, and the free scout stopped after five seconds. Its pale line from where it stopped shows where the flow takes a scout next.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 5 s; seed: 310205521
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - vortex about (0, 0): v = k perp(d) / (|d|^2 + soft^2), k = 4, soft = 1.5 (counter-clockwise)
  2. fixed - radial about (0, 0): v = k d / (|d|^2 + soft^2), k = -0.6, soft = 1.5 (attracts)
- regions:
- rules:
  - mechanic: **Survey**
  - the field layers above are the *charted background*; the hidden source is added to them and is not listed
  - hidden source: local repeller (Gauss term), strength k = 1.65, width sigma = 0.8, position unknown
  - scouts may be placed anywhere in rectangle x in [-5.5, 5.5], y in [-3.5, 3.5]; each leaves a 5 s trail
  - free scout flown at the start of every attempt from (-4, 0); 2 further scouts can be placed
  - one commit per attempt; correct within 0.6 of the source
  - estimate tests per attempt: 3 (each shows what the scouts would have done if the source were at the marker)
  - reading aid: faint rays back along each push arrow off
  - reading aid: shaded band where a source would already have been felt on
  - planning aid: preview of where a scout would fly under the charted background alone, shown while aiming on
  - the level is complete after 3 attempt(s) found; attempts use the positions below in order on the first pass, then reshuffled each pass
  - one source is hidden per attempt
  - hidden positions: (-3, 1), (2.5, -0.5), (0.5, 2), (1, -1.5), (-1.5, 2.5), (3.5, -1.5)

## shp-1 - Deflect (Shape)

- objective: Place a repeller so the flow carries the probe around the obstacle and onto the target.
- hint 1: Press Launch before placing anything: the probe flies the field as it is, and you see where it goes wrong.
- hint 2: A repeller pushes the flow away from itself, so the probe is pushed away from wherever you put it. The arrows redraw as you move it.
- hint 3: A source only changes the path while the probe is near it. Think about which side of the path the push has to come from, and whether it has to act before or after the obstacle.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 25 s; seed: 309816312
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - uniform flow (1, 0)
- regions:
  0. target 'T' centre (4.5, 1.3) radius 0.7
  1. hazard 'X' centre (-0.5, 0) radius 0.6
- rules:
  - mechanic: **Shape**
  - the field layers above are the fixed background; the player adds sources to them
  - probe starts at (-5, 0); it must reach region 0 (T), and any hazard ends the run
  - source kinds offered: repeller
  - up to 1 source(s), each with strength k = 1.65 and width sigma = 0.8 (Gauss terms); none on a target or obstacle, none within 0.5 of another
  - placement areas: rectangle x in [-4, 3], y in [-3, 3]
  - sources can be moved and removed freely; launching is free and repeatable; no path is shown before a launch
  - launch phase: fixed (0)
  - intensity: fixed (1)
  - run time limit: 25 s

## shp-2 - Recover (Shape)

- objective: The current drifts the probe below the target. Draw it back with a source.
- hint 1: Launch with nothing placed: the drift carries the probe past the target. Which way does it have to be moved?
- hint 2: An attractor pulls the flow toward itself, so a probe passing near it is drawn sideways toward it. Place one and watch how the arrows around it change.
- hint 3: A pull does not know about the obstacle. Check what lies between your source and the target, and where the probe will be while the pull acts.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 25 s; seed: 309816313
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - uniform flow (1, -0.08)
- regions:
  0. target 'T' centre (3.8, 1.2) radius 0.7
  1. hazard 'X' centre (0.2, 1.2) radius 0.5
- rules:
  - mechanic: **Shape**
  - the field layers above are the fixed background; the player adds sources to them
  - probe starts at (-5, 0); it must reach region 0 (T), and any hazard ends the run
  - source kinds offered: repeller, attractor
  - up to 1 source(s), each with strength k = 1.65 and width sigma = 0.8 (Gauss terms); none on a target or obstacle, none within 0.5 of another
  - placement areas: rectangle x in [-1, 2.8], y in [0.2, 2.2]
  - sources can be moved and removed freely; launching is free and repeatable; no path is shown before a launch
  - launch phase: fixed (0)
  - intensity: fixed (1)
  - run time limit: 25 s

## shp-3 - Turn (Shape)

- objective: The obstacle is on the stream and the target is right behind it. Get past it and come back.
- hint 1: Try a repeller and an attractor beside the path. The probe goes around, but does it come back to the line of the target?
- hint 2: A swirl pushes the flow around itself rather than away or toward it. Look at which way its arrows point on the side where the probe arrives, and on the side where it leaves.
- hint 3: Which way the swirl turns decides which way it pushes the probe first. The probe has to be moved clear of the obstacle while it passes it.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 25 s; seed: 309816314
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - uniform flow (0.7, 0)
- regions:
  0. target 'T' centre (4.5, 0) radius 0.5
  1. hazard 'X' centre (0, 0) radius 0.55
- rules:
  - mechanic: **Shape**
  - the field layers above are the fixed background; the player adds sources to them
  - probe starts at (-5, 0); it must reach region 0 (T), and any hazard ends the run
  - source kinds offered: repeller, attractor, swirl CCW, swirl CW
  - up to 1 source(s), each with strength k = 1.65 and width sigma = 0.8 (Gauss terms); none on a target or obstacle, none within 0.5 of another
  - placement areas: rectangle x in [-2.2, 2.2], y in [-1.8, 1.8]
  - sources can be moved and removed freely; launching is free and repeatable; no path is shown before a launch
  - launch phase: fixed (0)
  - intensity: fixed (1)
  - run time limit: 25 s

## shp-4 - Compose (Shape)

- objective: The target is higher than one source can reach. Use two, one after the other.
- hint 1: Place one source and launch. Where does it leave the probe, and is the target in reach from there?
- hint 2: The second source acts on the probe where the first left it, not where it started. Moving the first one changes where the second one can be useful.
- hint 3: The fields add. Each source only matters near the path, so the order along the stream matters too.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 25 s; seed: 309816315
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - uniform flow (1, 0)
- regions:
  0. target 'T' centre (4.5, 2.8) radius 0.7
  1. hazard 'X' centre (-0.5, 0) radius 0.55
- rules:
  - mechanic: **Shape**
  - the field layers above are the fixed background; the player adds sources to them
  - probe starts at (-5, 0); it must reach region 0 (T), and any hazard ends the run
  - source kinds offered: repeller, attractor
  - up to 2 source(s), each with strength k = 1.65 and width sigma = 0.8 (Gauss terms); none on a target or obstacle, none within 0.5 of another
  - placement areas: rectangle x in [-3, 3], y in [0.3, 3.2]
  - sources can be moved and removed freely; launching is free and repeatable; no path is shown before a launch
  - launch phase: fixed (0)
  - intensity: fixed (1)
  - run time limit: 25 s

## syn-1 - Rendezvous (Synchronize)

- objective: Two probes must reach the target together. Choose when to release each.
- hint 1: Launch both once and watch the timeline: each bar is how long that probe takes to arrive.
- hint 2: In this field a probe takes the same time whenever it leaves. What matters is how much later one leaves than the other.
- hint 3: The shaded band on the timeline is how far apart the two arrivals may be.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 20 s; seed: 310320837
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - uniform flow (1.2, 0)
  2. fixed - radial about (4.5, 0): v = k d / (|d|^2 + soft^2), k = -5, soft = 1 (attracts)
- regions:
  0. target 'T' centre (4.5, 0) radius 0.8
- rules:
  - mechanic: **Synchronize**
  - 2 probes fly the same field; the player chooses only each probe's release delay (0 to 12 s, in steps of 0.1 s; the field clock at release)
  - field period 12 s
  - probe 0 starts at (-4.5, -2.5) and must reach region 0 (T)
  - probe 1 starts at (-1, 1) and must reach region 0 (T)
  - condition: probe 1 must enter region 0 (T) between -0.4 and 0.4 s after probe 0 enters region 0 (T)
  - run time limit: 20 s per probe

## syn-2 - Windows (Synchronize)

- objective: Each probe can reach the target only at certain times. Find a time they can share.
- hint 1: Send each probe on its own first and see which release times get it to the target at all.
- hint 2: A probe that can only leave in a window arrives in a window too. The two arrival windows have to overlap.
- hint 3: The shaded band on the timeline is how far apart the arrivals may be; a probe that misses the target has no arrival to compare.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 20 s; seed: 310320838
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - uniform flow (2, 0)
  2. fixed - modulated: [local radial push about (-1.5, -1.9): v = k d exp(-|d|^2 / (2 sigma^2)), k = 1.65, sigma = 0.8 (repels; peak speed k sigma e^-1/2 at distance sigma; 5.5% of the peak at 3 sigma, 0.2% at 4 sigma)] x (0 + 1 sin(0.5235987755982988 t + 4))
  3. fixed - modulated: [local radial push about (1, 1.9): v = k d exp(-|d|^2 / (2 sigma^2)), k = 1.65, sigma = 0.8 (repels; peak speed k sigma e^-1/2 at distance sigma; 5.5% of the peak at 3 sigma, 0.2% at 4 sigma)] x (0 + 1 sin(0.5235987755982988 t + 4))
- regions:
  0. target 'T' centre (4.5, 0) radius 0.9
- rules:
  - mechanic: **Synchronize**
  - 2 probes fly the same field; the player chooses only each probe's release delay (0 to 12 s, in steps of 0.1 s; the field clock at release)
  - field period 12 s
  - probe 0 starts at (-5, -1.2) and must reach region 0 (T)
  - probe 1 starts at (-3.5, 1.2) and must reach region 0 (T)
  - condition: probe 1 must enter region 0 (T) between -0.6 and 0.6 s after probe 0 enters region 0 (T)
  - run time limit: 20 s per probe

## syn-3 - Order (Synchronize)

- objective: The second probe must arrive after the first, not with it.
- hint 1: The band on the timeline now sits after the first arrival: the second probe has to be late by a small, particular amount.
- hint 2: Each probe still has its own times at which it can reach the target at all.
- hint 3: Choose the first arrival you want, then see whether the second probe can arrive 1 to 2 seconds later.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 20 s; seed: 310320839
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - uniform flow (2, 0)
  2. fixed - modulated: [local radial push about (-1.5, -1.9): v = k d exp(-|d|^2 / (2 sigma^2)), k = 1.65, sigma = 0.8 (repels; peak speed k sigma e^-1/2 at distance sigma; 5.5% of the peak at 3 sigma, 0.2% at 4 sigma)] x (0 + 1 sin(0.5235987755982988 t + 4))
  3. fixed - modulated: [local radial push about (1, 1.9): v = k d exp(-|d|^2 / (2 sigma^2)), k = 1.65, sigma = 0.8 (repels; peak speed k sigma e^-1/2 at distance sigma; 5.5% of the peak at 3 sigma, 0.2% at 4 sigma)] x (0 + 1 sin(0.5235987755982988 t + 3.5))
- regions:
  0. target 'T' centre (4.5, 0) radius 0.9
- rules:
  - mechanic: **Synchronize**
  - 2 probes fly the same field; the player chooses only each probe's release delay (0 to 12 s, in steps of 0.1 s; the field clock at release)
  - field period 12 s
  - probe 0 starts at (-5, -1.2) and must reach region 0 (T)
  - probe 1 starts at (-3.5, 1.2) and must reach region 0 (T)
  - condition: probe 1 must enter region 0 (T) between 1 and 2.2 s after probe 0 enters region 0 (T)
  - run time limit: 20 s per probe

## syn-4 - Meet, Then Part (Synchronize)

- objective: The probes must meet at the gate, then reach the target one after the other.
- hint 1: Meeting at the gate and arriving apart at the target are two conditions on the same two release times.
- hint 2: The field speeds probes up and slows them down in a cycle, so how far apart they arrive depends on when they pass through it.
- hint 3: A pair that meets at the gate can still arrive together at the target. Check both bands on the timeline after every launch.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 20 s; seed: 310320840
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - uniform flow (1, 0)
  2. fixed - modulated: [uniform flow (1.2, 0)] x (0 + 0.8 sin(0.5235987755982988 t + 4))
  3. fixed - modulated: [uniform flow (0, 1)] x (0 + 0.8 sin(0.5235987755982988 t + 6))
  4. fixed - radial about (4.5, 0): v = k d / (|d|^2 + soft^2), k = -5, soft = 1 (attracts)
- regions:
  0. target 'T' centre (4.5, 0) radius 0.8
  1. gate 'G' centre (0.5, 0) radius 1.3
- rules:
  - mechanic: **Synchronize**
  - 2 probes fly the same field; the player chooses only each probe's release delay (0 to 12 s, in steps of 0.1 s; the field clock at release)
  - field period 12 s
  - probe 0 starts at (-4.5, -2) and must reach region 0 (T)
  - probe 1 starts at (-4.5, 0.8) and must reach region 0 (T)
  - condition: probe 1 must enter region 1 (G) between -1 and 1 s after probe 0 enters region 1 (G)
  - condition: probe 1 must enter region 0 (T) between 0.4 and 1.2 s after probe 0 enters region 0 (T)
  - run time limit: 20 s per probe

## syn-5 - Chain (Synchronize)

- objective: The first and second meet at the gate. The second and third meet at the target.
- hint 1: Two pairs have to line up, and the middle probe belongs to both.
- hint 2: Decide when the middle probe goes, then fit the other two around it.
- hint 3: The third probe cannot reach the target at every time. Try it alone first.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 20 s; seed: 310320841
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - uniform flow (1, 0)
  2. fixed - modulated: [uniform flow (1.2, 0)] x (0 + 0.8 sin(0.5235987755982988 t + 4))
  3. fixed - modulated: [uniform flow (0, 1)] x (0 + 0.8 sin(0.5235987755982988 t + 6))
  4. fixed - radial about (4.5, 0): v = k d / (|d|^2 + soft^2), k = -5, soft = 1 (attracts)
- regions:
  0. target 'T' centre (4.5, 0) radius 0.8
  1. gate 'G' centre (0.5, 0) radius 1.3
- rules:
  - mechanic: **Synchronize**
  - 3 probes fly the same field; the player chooses only each probe's release delay (0 to 12 s, in steps of 0.1 s; the field clock at release)
  - field period 12 s
  - probe 0 starts at (-4.5, -2) and must reach region 0 (T)
  - probe 1 starts at (-4.5, 0.8) and must reach region 0 (T)
  - probe 2 starts at (-4.5, 3) and must reach region 0 (T)
  - condition: probe 1 must enter region 1 (G) between -0.8 and 0.8 s after probe 0 enters region 1 (G)
  - condition: probe 2 must enter region 0 (T) between -0.8 and 0.8 s after probe 1 enters region 0 (T)
  - run time limit: 20 s per probe

## cmb-1 - Through the Gate (Combinations)

- objective: Position + corridor: pass the gate, then reach the target.
- hint 1: The gate sits on a route you already know from Two Roads.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 40 s; seed: 295175477
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - uniform flow (0.7, 0)
  2. fixed - vortex about (0, 0): v = k perp(d) / (|d|^2 + soft^2), k = -2.5, soft = 0.8 (clockwise)
- regions:
  0. target 'T' centre (4, -2) radius 0.6
  1. gate 'G' centre (0, 2) radius 0.4
- rules:
  - mechanic: **Position**
  - release zone 1: rectangle x in [-5.5, -4], y in [-3, 3]; its probe must reach region 0 (T)
  - launch phase: fixed (0)
  - intensity: fixed (1)

## cmb-2 - Early Clue (Combinations)

- objective: Predict + partial visibility: use the revealed opening of the path.
- hint 1: The first seconds are shown. Extrapolate from the heading it has when it ends.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 40 s; seed: 295175478
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - uniform flow (0.9, 0)
  2. fixed - vortex about (1, 0): v = k perp(d) / (|d|^2 + soft^2), k = 3, soft = 0.5 (counter-clockwise)
- regions:
  0. exit 'A' centre (5.2, 3) radius 0.85
  1. exit 'B' centre (5.2, 1) radius 0.85
  2. exit 'C' centre (5.2, -1) radius 0.85
  3. exit 'D' centre (5.2, -3) radius 0.85
- rules:
  - mechanic: **Predict**
  - start zone (release points are drawn from it): rectangle x in [-5.5, -4], y in [-3, 3]
  - rounds: 3; correct answers needed: 2
  - reveal before commit: the first 2.5 s of the true path

## cmb-3 - Dodge (Combinations)

- objective: Tempo + constraint: reach the target without touching the hazard.
- hint 1: Some launch times lift the probe straight through the hazard. Later ones pass above it.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 40 s; seed: 295175479
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - uniform flow (2.5, 0)
  2. fixed - modulated: [uniform flow (0, 1)] x (0 + 1 sin(0.5235987755982988 t + 0))
- regions:
  0. target 'T' centre (4.5, 2.6) radius 0.9
  1. hazard 'X' centre (0.5, 0.8) radius 0.7
- rules:
  - mechanic: **Tempo**
  - fixed start (-4.5, 0)
  - field period 12 s
  - goal region 0 (T)

## cmb-4 - Three Exits (Combinations)

- objective: Experiment + multiple exits: find where exit C turns into exit D.
- hint 1: Both exits must be seen before you can report the change.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 40 s; seed: 295175480
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - radial about (0, 0): v = k d / (|d|^2 + soft^2), k = 1.2, soft = 0.6 (repels)
  2. tunable - vortex about (0, 0): v = k perp(d) / (|d|^2 + soft^2), k = -1, soft = 0.6 (clockwise)
  (default intensity 1)
- regions:
  0. exit 'A' centre (5.6, 3.6) radius 2.2
  1. exit 'B' centre (5.6, -3.6) radius 2.2
  2. exit 'C' centre (-5.6, -3.6) radius 2.2
  3. exit 'D' centre (-5.6, 3.6) radius 2.2
- rules:
  - mechanic: **Experiment**
  - fixed start (1.4, 0)
  - intensity range [0, 3]
  - challenge: Threshold: intensity where exit C becomes exit D, tolerance +/-0.15

## cmb-5 - Right Place, Right Time (Combinations)

- objective: Position + Tempo: choose where and when.
- hint 1: Both choices matter: a good place may need a different time than a poor one.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 40 s; seed: 295175481
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - uniform flow (2.5, 0)
  2. fixed - orbiting vortex source: centre (0, 0) + 2 (cos(a), sin(a)), a = 0.5235987755982988 t + 0; k = 3, soft = 0.6
- regions:
  0. target 'T' centre (4.5, 2) radius 0.7
- rules:
  - mechanic: **Position**
  - release zone 1: rectangle x in [-5.5, -4], y in [-1, 1]; its probe must reach region 0 (T)
  - launch phase is also chosen, period 12 s
  - intensity: fixed (1)

## cmb-6 - Shared Dial (Combinations)

- objective: Two probes, one intensity: make both reach their targets.
- hint 1: One dial bends both paths at once. A setting that suits the top probe can strand the bottom one.
- hint 2: Try a dial value first, then look for a release point in each zone.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 40 s; seed: 295175482
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - uniform flow (0.75, 0)
  2. tunable - vortex about (-0.63, -0.94): v = k perp(d) / (|d|^2 + soft^2), k = 2.7, soft = 0.8 (counter-clockwise)
  (default intensity 1)
- regions:
  0. target 'T' centre (4, 3.02) radius 0.6
  1. target 'T' centre (4, -2.13) radius 0.6
- rules:
  - mechanic: **Position**
  - release zone 1: rectangle x in [-5.5, -4], y in [0.3, 1.33]; its probe must reach region 0 (T)
  - release zone 2: rectangle x in [-5.5, -4], y in [-1.33, -0.3]; its probe must reach region 1 (T)
  - launch phase: fixed (0)
  - shared intensity is also chosen, range [0, 3]

## cmb-7 - Source on the Tide (Combinations)

- objective: Shape + Tempo: the flow pulses. Choose when to launch and where to put one source.
- hint 1: Launch with nothing placed and scrub the timeline: the pulse lifts the probe at some times and not at others, but never onto the target.
- hint 2: A source does not pulse. It pushes the same way whenever the probe passes it, so the same source can help at one launch time and be useless at another.
- hint 3: Find a launch time that gets the probe close, then place the source where it adds the rest. Moving the launch time moves where the probe is when it reaches the source.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 25 s; seed: 295175483
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - uniform flow (1, 0)
  2. fixed - modulated: [uniform flow (0, 1)] x (0 + 0.6 sin(0.5235987755982988 t + 2.22529479629277))
- regions:
  0. target 'T' centre (4.5, 2.7) radius 0.6
  1. hazard 'X' centre (-0.5, 0) radius 0.6
- rules:
  - mechanic: **Shape**
  - the field layers above are the fixed background; the player adds sources to them
  - probe starts at (-5, 0); it must reach region 0 (T), and any hazard ends the run
  - source kinds offered: repeller, attractor
  - up to 1 source(s), each with strength k = 1.65 and width sigma = 0.8 (Gauss terms); none on a target or obstacle, none within 0.5 of another
  - placement areas: rectangle x in [-4, 3], y in [-3, 3]
  - sources can be moved and removed freely; launching is free and repeatable; no path is shown before a launch
  - launch phase is also chosen, period 12 s (sources do not change with it)
  - intensity: fixed (1)
  - run time limit: 25 s

## cmb-8 - Compensate (Combinations)

- objective: Shape + dial: set the strength of the current, then place one source to finish the approach.
- hint 1: Launch with nothing placed at a few dial settings. The big obstacle sits on the stream: the current has to carry the probe round it.
- hint 2: A source only acts near where you put it, downstream of the obstacle. It cannot carry the probe round the obstacle; the dial has to.
- hint 3: Too weak and the probe meets the obstacle; too strong and it is flung past the target by more than one source can undo. Find the band between, then place the source.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 25 s; seed: 295175484
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - uniform flow (1, 0)
  2. tunable - vortex about (-0.7, 2.5): v = k perp(d) / (|d|^2 + soft^2), k = -3.6, soft = 0.8 (clockwise)
  (default intensity 1)
- regions:
  0. target 'T' centre (4.5, 0.9) radius 0.6
  1. hazard 'X' centre (-0.15, 0.45) radius 1.25
- rules:
  - mechanic: **Shape**
  - the field layers above are the fixed background; the player adds sources to them
  - probe starts at (-5, 0); it must reach region 0 (T), and any hazard ends the run
  - source kinds offered: repeller, attractor
  - up to 1 source(s), each with strength k = 1.65 and width sigma = 0.8 (Gauss terms); none on a target or obstacle, none within 0.5 of another
  - placement areas: rectangle x in [0, 3.5], y in [-3, 3]
  - sources can be moved and removed freely; launching is free and repeatable; no path is shown before a launch
  - launch phase: fixed (0)
  - background intensity is also chosen, range [0, 3] (sources ignore it)
  - run time limit: 25 s

## cmb-9 - Make Them Meet (Combinations)

- objective: Shape + Synchronize: place one source, then time both probes to arrive together.
- hint 1: Launch both probes with no source and read the timeline. Each can reach the target at some release times, but their arrival times never come within the allowed band of each other.
- hint 2: A source changes the route of any probe that passes near it, and so changes when that probe arrives. It does not pulse: it pushes the same way at every release time.
- hint 3: Place the source first, then try release times. A source that moves one probe's arrival into reach of the other's is the start; the delays decide the rest.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 20 s; seed: 295175485
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - uniform flow (1.9, 0)
  2. fixed - modulated: [local radial push about (-0.08, -1.67): v = k d exp(-|d|^2 / (2 sigma^2)), k = 1.65, sigma = 0.8 (repels; peak speed k sigma e^-1/2 at distance sigma; 5.5% of the peak at 3 sigma, 0.2% at 4 sigma)] x (0 + 1 sin(0.5235987755982988 t + 9.43))
  3. fixed - modulated: [local radial push about (0.97, 1.85): v = k d exp(-|d|^2 / (2 sigma^2)), k = 1.65, sigma = 0.8 (repels; peak speed k sigma e^-1/2 at distance sigma; 5.5% of the peak at 3 sigma, 0.2% at 4 sigma)] x (0 + 1 sin(0.5235987755982988 t + 5.47))
- regions:
  0. target 'T' centre (4.5, 0) radius 0.9
- rules:
  - mechanic: **Synchronize**
  - 2 probes fly the same field; the player chooses only each probe's release delay (0 to 12 s, in steps of 0.1 s; the field clock at release)
  - field period 12 s
  - probe 0 starts at (-5, -1.46) and must reach region 0 (T)
  - probe 1 starts at (-2.69, 1.13) and must reach region 0 (T)
  - condition: probe 1 must enter region 0 (T) between -0.6 and 0.6 s after probe 0 enters region 0 (T)
  - sources are also placed (they change the field every probe flies): up to 1 of repeller / attractor (strength 1.65, width 0.8), inside the placement area
  - run time limit: 20 s per probe

## cmb-10 - Route and Rendezvous (Combinations)

- objective: Synchronize + Position: choose where probe B starts, then time both probes to arrive together.
- hint 1: Probe B can start from any of the numbered markers. Launch both probes from the first marker and read the timeline: B can reach the target at some release times, but never within the allowed band of A's arrival, whatever you do with the delays.
- hint 2: Each start gives B a different route through the pulsing field, and so different release times at which it arrives, and different arrival times. Try the other markers and see which of them puts B's arrival into reach of A's.
- hint 3: Two of the five starts can work. Choose one, then use the delays to line the arrivals up; releasing both together does not work from any start.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 20 s; seed: 9150439835
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - uniform flow (1.26, -0.16)
  2. fixed - modulated: [local radial push about (-2.58, -0.68): v = k d exp(-|d|^2 / (2 sigma^2)), k = 1.65, sigma = 0.8 (repels; peak speed k sigma e^-1/2 at distance sigma; 5.5% of the peak at 3 sigma, 0.2% at 4 sigma)] x (0 + 1 sin(0.5235987755982988 t + 9.469999999999999))
  3. fixed - modulated: [local radial push about (-1.06, -1.1): v = k d exp(-|d|^2 / (2 sigma^2)), k = 1.65, sigma = 0.8 (repels; peak speed k sigma e^-1/2 at distance sigma; 5.5% of the peak at 3 sigma, 0.2% at 4 sigma)] x (0 + 1 sin(0.5235987755982988 t + 4.2))
- regions:
  0. target 'T' centre (4.5, 0.25) radius 0.9
- rules:
  - mechanic: **Synchronize**
  - 2 probes fly the same field; the player chooses only each probe's release delay (0 to 12 s, in steps of 0.1 s; the field clock at release)
  - field period 12 s
  - probe 0 starts at (-5, 1.05) and must reach region 0 (T)
  - probe 1 starts at (-4.58, -2.14) and must reach region 0 (T)
  - condition: probe 1 must enter region 0 (T) between -0.6 and 0.6 s after probe 0 enters region 0 (T)
  - probe 1 is also released from one of 5 numbered start slots (slot 1 is its default): 1 at (-4.58, -2.14), 2 at (-2.18, 1.09), 3 at (-3.05, 0.51), 4 at (-2.52, -1.15), 5 at (-4.83, -1.2)
  - run time limit: 20 s per probe

## cmb-23 - Dial and Rendezvous (Combinations)

- objective: Synchronize + Experiment: set the dial, then time both probes to arrive together.
- hint 1: The dial scales only part of the field: the two pulsing lanes and a sideways drift. Launch both probes at the first setting and read the timeline: with this setting they can never be made to arrive together, whatever the delays.
- hint 2: Each setting changes when and where each probe reaches the target, so it changes the times at which they can arrive. Try the other settings, and watch which of them brings the two arrival windows into reach of each other.
- hint 3: Three neighbouring settings work. Choose one, then use the delays to line the arrivals up; releasing both together does not work at any setting.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 20 s; seed: 9150439869
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - uniform flow (1.41, 0.05)
  2. tunable - modulated: [local radial push about (2.33, 0.89): v = k d exp(-|d|^2 / (2 sigma^2)), k = 1.65, sigma = 0.8 (repels; peak speed k sigma e^-1/2 at distance sigma; 5.5% of the peak at 3 sigma, 0.2% at 4 sigma)] x (0 + 1 sin(0.5235987755982988 t + 9.77))
  3. tunable - modulated: [local radial push about (-2.71, -1.6): v = k d exp(-|d|^2 / (2 sigma^2)), k = 1.65, sigma = 0.8 (repels; peak speed k sigma e^-1/2 at distance sigma; 5.5% of the peak at 3 sigma, 0.2% at 4 sigma)] x (0 + 1 sin(0.5235987755982988 t + 8.879999999999999))
  4. tunable - uniform flow (-0.43, 0.34)
  (default intensity 1)
- regions:
  0. target 'T' centre (4.5, 0.85) radius 0.9
- rules:
  - mechanic: **Synchronize**
  - 2 probes fly the same field; the player chooses only each probe's release delay (0 to 12 s, in steps of 0.1 s; the field clock at release)
  - field period 12 s
  - probe 0 starts at (-5, -1.28) and must reach region 0 (T)
  - probe 1 starts at (-5, 0.08) and must reach region 0 (T)
  - condition: probe 1 must enter region 0 (T) between -0.6 and 0.6 s after probe 0 enters region 0 (T)
  - the dial is also chosen first, from 13 numbered settings (1, 0, 0.2, 0.4, 0.6, 0.8, 1.2, 1.4, 1.6, 1.8, 2, 2.2, 2.4; the first is the default)
  - run time limit: 20 s per probe

## cmb-24 - Dial and Time It (Combinations)

- objective: Tempo + Experiment: set the dial, then choose when to launch.
- hint 1: The dial scales only the two pulsing pushes; the steady stream ignores it. At the first setting there is no launch time that reaches the target: scrub the whole cycle and see.
- hint 2: Each setting changes how far the pulses push the probe, so it changes which part of the cycle carries it to the target. Try the other settings and scrub the clock again to find where the probe gets through.
- hint 3: Two of the five settings work. Launching at phase 0, or close to it, never works at any setting.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 40 s; seed: 9150439870
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - uniform flow (2.45, 0.03)
  2. tunable - modulated: [uniform flow (0, 0.77)] x (0 + 0.92 sin(0.5235987755982988 t + 1.02))
  3. tunable - modulated: [uniform flow (-0.42, -0.77)] x (0 + 0.66 sin(0.5235987755982988 t + 0.6))
  (default intensity 1)
- regions:
  0. target 'T' centre (4.5, -1.93) radius 0.7
- rules:
  - mechanic: **Tempo**
  - fixed start (-4.5, 0)
  - field period 12 s
  - goal region 0 (T)
  - the dial is also chosen before the launch, from 5 numbered settings (1, 0, 0.5, 1.5, 2; the first is the default)

## cmb-28 - Meet the Drone (Combinations)

- objective: Steer + Synchronize: release the drone, then steer to arrive with it.
- hint 1: The drone (A) flies on its own and its path and arrival time are shown for the release you choose. At some releases it never reaches the target: choose another.
- hint 2: Your probe needs its gust to reach the target at all, and the gust shifts when it arrives only a little. Compare when the drone arrives with when your probe would arrive with no gust, and pick the release that brings the two closest.
- hint 3: The release cannot be changed once the first beat is played. Spend the gust on the beat that brings your probe to the target at the drone's time.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 40 s; seed: 9150439874
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - uniform flow (1.14, 0.2)
  2. fixed - modulated: [uniform flow (0, 0.84)] x (0 + 0.95 sin(0.5235987755982988 t + 6.14))
- regions:
  0. target 'T' centre (4.4, 0.5) radius 0.6
- rules:
  - mechanic: **Steer**
  - fixed start (-5, -2.38)
  - beats 10; gust budget 1; gust strength 2; beat length 1 s
  - gust directions: 8 (E, NE, N, NW, W, SW, S, SE)
  - goal region 0 (T)
  - launch phase: fixed (0)
  - start point: fixed
  - sources: none
  - a drone starts at (-5, -2.41) and flies the unsteered field by itself, released at one of 25 numbered delays (0, 0.5, 1, 1.5, 2, 2.5, 3, 3.5, 4, 4.5, 5, 5.5, 6, 6.5, 7, 7.5, 8, 8.5, 9, 9.5, 10, 10.5, 11, 11.5, 12 s; the first is the default); the steered probe must reach the goal within 0.6 s of the drone's arrival
  - dial: not offered

## cmb-22 - Set the Dial, Then Steer (Combinations)

- objective: Steer + Experiment: set the dial, then steer with one gust.
- hint 1: One gust is not enough to reach the target at the dial setting you start with. Try the other settings and watch how the arrows change: the dial scales only part of the field.
- hint 2: Pick the setting whose flow carries the probe closest to the target on its own (a gust-free run shows it), then spend the gust where the field still falls short. The dial cannot be changed once the first beat is played.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 40 s; seed: 9150439868
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - uniform flow (1.02, -0.26)
  2. tunable - uniform flow (-0.22, 0.39)
  3. tunable - vortex about (2.17, -1.32): v = k perp(d) / (|d|^2 + soft^2), k = -1.6, soft = 0.8 (clockwise)
  (default intensity 1)
- regions:
  0. target 'T' centre (4.4, -0.47) radius 0.7
- rules:
  - mechanic: **Steer**
  - fixed start (-5, 0)
  - beats 8; gust budget 1; gust strength 1; beat length 1 s
  - gust directions: 8 (E, NE, N, NW, W, SW, S, SE)
  - goal region 0 (T)
  - launch phase: fixed (0)
  - start point: fixed
  - sources: none
  - companion drone: none
  - the dial is also chosen before the first beat, from 13 numbered settings (1, 0, 0.2, 0.4, 0.6, 0.8, 1.2, 1.4, 1.6, 1.8, 2, 2.2, 2.4; the first is the default)

## cmb-11 - Catch the Gust (Combinations)

- objective: Steer + Tempo: choose when to start, then steer with two gusts.
- hint 1: Two gusts are not enough to reach the target from the start you are given. Scrub the start time and watch which way the pulse pushes the probe.
- hint 2: Start when the pulse carries the probe up for most of the run, then use the gusts for the rest. The start time cannot be changed once the first beat is played.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 40 s; seed: 9150439836
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - uniform flow (1.2, 0)
  2. fixed - modulated: [uniform flow (0, 1)] x (0 + 0.6 sin(0.5235987755982988 t + 2.617993877991494))
- regions:
  0. target 'T' centre (4.4, 3.1) radius 0.5
- rules:
  - mechanic: **Steer**
  - fixed start (-5, 0)
  - beats 8; gust budget 2; gust strength 1; beat length 1 s
  - gust directions: 8 (E, NE, N, NW, W, SW, S, SE)
  - goal region 0 (T)
  - launch phase is also chosen before the first beat, period 12 s
  - start point: fixed
  - sources: none
  - companion drone: none
  - dial: not offered

## cmb-12 - Fewer Gusts (Combinations)

- objective: Steer + Position: choose where to start, then steer with two gusts.
- hint 1: Two gusts are not enough from the start you are given. Click on the start line to move where the probe begins, before the first beat.
- hint 2: A higher start saves gusts, but the obstacle above the lane punishes a start that is too high. Look for the band between.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 40 s; seed: 9150439837
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - uniform flow (1.15, 0)
  2. fixed - vortex about (-2.7, 0): v = k perp(d) / (|d|^2 + soft^2), k = -2.1, soft = 0.8 (clockwise)
- regions:
  0. target 'T' centre (4, 2.85) radius 0.5
  1. hazard 'X' centre (-0.15, 3.35) radius 0.9
- rules:
  - mechanic: **Steer**
  - fixed start (-5, 0)
  - beats 8; gust budget 2; gust strength 1; beat length 1 s
  - gust directions: 8 (E, NE, N, NW, W, SW, S, SE)
  - goal region 0 (T)
  - launch phase: fixed (0)
  - start point is also chosen inside [-5, -5] x [-3, 3] before the first beat
  - sources: none
  - companion drone: none
  - dial: not offered

## cmb-13 - Pave the Way (Combinations)

- objective: Steer + Shape: place one source first, then steer with a single gust.
- hint 1: With no source, play the run and watch where the field takes the probe: one gust cannot bring it to the target.
- hint 2: A source acts the whole way along the run, not just on one beat. Place it where it bends the flow toward the target and away from the obstacle.
- hint 3: You cannot change the source once the first beat is played. Place it, then spend the single gust where the field still falls short.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 40 s; seed: 9150439838
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - uniform flow (1.12, 0)
  2. fixed - vortex about (-2, 0.97): v = k perp(d) / (|d|^2 + soft^2), k = 1.05, soft = 0.8 (counter-clockwise)
- regions:
  0. target 'T' centre (3.76, -2.05) radius 0.5
  1. hazard 'X' centre (-0.94, -0.9) radius 0.6
- rules:
  - mechanic: **Steer**
  - fixed start (-5, 0)
  - beats 8; gust budget 1; gust strength 1; beat length 1 s
  - gust directions: 8 (E, NE, N, NW, W, SW, S, SE)
  - goal region 0 (T)
  - launch phase: fixed (0)
  - start point: fixed
  - sources are also placed before the first beat: up to 1 of repeller / attractor (strength 1.65, width 0.8), inside the placement area
  - companion drone: none
  - dial: not offered

## cmb-14 - Scout, Then Release (Combinations)

- objective: Survey + Position: scout the hidden source, then choose where to release the probe.
- hint 1: The probe can be released from any of the numbered diamonds, but only some of them reach the target, and which ones depends on the hidden source. Scouts are optional, but the source changes every attempt.
- hint 2: A scout starts in the shaded strip and shows you what the source does to a flight: the yellow arrows are its push. Work out where the source is and which way it bends the flow the probe will ride.
- hint 3: Release from a slot on the side the source would push the probe away from, so that it is bent back onto the target. Three right choices in a row.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 20 s; seed: 9150439839
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - uniform flow (1.22, 0.11)
  2. fixed - vortex about (-0.75, -2.3): v = k perp(d) / (|d|^2 + soft^2), k = 1.9, soft = 0.8 (counter-clockwise)
- regions:
  0. target 'T' centre (4.5, 1.25) radius 0.8
- rules:
  - mechanic: **Scout, then release (Survey + Position)**
  - the main probe starts at (-5, 0) unless a choice moves it
  - the field layers above are the visible background; a hidden source is added to it in every flight
  - hidden sources (one is drawn at random each attempt, never the one just used; strength 2.01, width 0.8): attractor at (-1.4, 2.48), attractor at (-2.28, -1.02), attractor at (-0.57, -2.33)
  - up to 3 scouts per attempt, started anywhere in [-5.6, -4.4] x [-3, 3], each flying 6 s; the push each feels is shown (as in Survey)
  - then choose the release slot of the main probe from 7 numbered slots (1 at (-5, -2.52), 2 at (-5, -1.74), 3 at (-5, -0.86), 4 at (-5, 0.12), 5 at (-5, 0.88), 6 at (-5, 1.57), 7 at (-5, 2.51)), which must reach the target at (4.5, 1.25); the choice is committed before the probe flies
  - 3 correct choices in a row complete the level; a wrong one resets the streak
  - run time limit: 20 s

## cmb-15 - Cancel It (Combinations)

- objective: Survey + Shape: scout the hidden source, then choose a source to add that cancels it.
- hint 1: A hidden source bends the probe off the target. You may add one source of your own: pick one of the eight marked options (orange plus: a repeller, blue minus: an attractor). The same kind of source in the same place would double the push.
- hint 2: Scout the hidden source and read its kind from the yellow arrows: they point away from a repeller and toward an attractor. Then add the opposite kind in the same place, so that the two push cancel.
- hint 3: The hidden source is in one of four places and changes every attempt. Three right choices in a row.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 20 s; seed: 9150439840
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - uniform flow (1.06, -0.05)
  2. fixed - vortex about (1.78, 0.94): v = k perp(d) / (|d|^2 + soft^2), k = -0.86, soft = 0.8 (clockwise)
- regions:
  0. target 'T' centre (4.5, 0.11) radius 0.9
- rules:
  - mechanic: **Scout, then cancel (Survey + Shape)**
  - the main probe starts at (-5, 0) unless a choice moves it
  - the field layers above are the visible background; a hidden source is added to it in every flight
  - hidden sources (one is drawn at random each attempt, never the one just used; strength 2.01, width 0.8): attractor at (-2.17, 1.26), repeller at (0.59, 0.63), attractor at (2.22, 0.56)
  - up to 3 scouts per attempt, started anywhere in [-5.6, -4.4] x [-3, 3], each flying 6 s; the push each feels is shown (as in Survey)
  - then choose one source to add from 8 numbered options (1 repeller at (-2.17, 1.26), 2 repeller at (-0.74, -0.26), 3 repeller at (0.59, 0.63), 4 repeller at (2.22, 0.56), 5 attractor at (-2.17, 1.26), 6 attractor at (-0.74, -0.26), 7 attractor at (0.59, 0.63), 8 attractor at (2.22, 0.56)), same strength and width as the hidden ones; the probe from (-5, 0) must reach the target at (4.5, 0.11); the choice is committed before the probe flies
  - 3 correct choices in a row complete the level; a wrong one resets the streak
  - run time limit: 20 s

## cmb-16 - Scout, Then Call It (Combinations)

- objective: Survey + Predict: scout the hidden source, then call which exit the probe takes.
- hint 1: The arrows show the background only. A hidden source is also bending every flight, yours included. Launch scouts from the strip on the left and read the small push arrows: they show what the hidden source did to each scout.
- hint 2: A push points along the line to the source: away from it for a repeller, toward it for an attractor. Two scouts that felt it are enough to see where those lines cross, and the direction tells you the kind.
- hint 3: Once you know where it is and which kind, picture the main probe (the diamond) flying through the same field, then call the exit. The hidden source is different every attempt, so a call that worked once will not work again: three right calls in a row.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 20 s; seed: 9150439841
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - uniform flow (1.08, 0.09)
- regions:
  0. target 'A' centre (4.5, 2.7) radius 1
  1. target 'B' centre (4.5, 0.9) radius 1
  2. target 'C' centre (4.5, -0.9) radius 1
  3. target 'D' centre (4.5, -2.7) radius 1
- rules:
  - mechanic: **Scout, then call (Survey + Predict)**
  - the main probe starts at (-5, 0)
  - the field layers above are the visible background; a hidden source is added to it in every flight
  - hidden sources (one is drawn at random each attempt, never the one just used; strength 2.01, width 0.8): repeller at (-0.38, -1.43), attractor at (-2.06, 1.64), attractor at (0.91, -0.68)
  - up to 3 scouts per attempt, started anywhere in [-5.6, -4.4] x [-3, 3], each flying 6 s; the push each feels is shown (as in Survey)
  - then call which of 4 exits the main probe will take: A at (4.5, 2.7), B at (4.5, 0.9), C at (4.5, -0.9), D at (4.5, -2.7); the call is committed before the probe flies
  - 3 correct choices in a row complete the level; a wrong one resets the streak
  - run time limit: 20 s

## cmb-17 - Scout, Then Steer (Combinations)

- objective: Survey + Steer: scout the hidden source, then choose the gust plan that gets the probe to the target.
- hint 1: Four gust plans are on offer. Each gusts in a fixed direction at fixed beats; the probe gets six beats of one second to reach the target. Only one plan suits each hidden source.
- hint 2: The hidden source changes every attempt. A scout shows you where it is and which kind it is (the yellow arrows point away from a repeller and toward an attractor), and so which way it will push the probe.
- hint 3: Read each plan as a push in a direction at a time, and pick the one that cancels the source's effect on the probe. Three right choices in a row.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 20 s; seed: 9150439842
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - uniform flow (1.14, 0.13)
  2. fixed - vortex about (-1.45, 1.5): v = k perp(d) / (|d|^2 + soft^2), k = 2.85, soft = 0.8 (counter-clockwise)
- regions:
  0. target 'T' centre (4.5, 1.82) radius 1
- rules:
  - mechanic: **Scout, then steer (Survey + Steer)**
  - the main probe starts at (-5, 0) unless a choice moves it
  - the field layers above are the visible background; a hidden source is added to it in every flight
  - hidden sources (one is drawn at random each attempt, never the one just used; strength 3, width 0.8): attractor at (0.48, -1.83), attractor at (0.52, 1.24), repeller at (1.71, -1.47)
  - up to 3 scouts per attempt, started anywhere in [-5.6, -4.4] x [-3, 3], each flying 6 s; the push each feels is shown (as in Survey)
  - then choose one of 4 fixed gust plans (strength 1, 1 s per beat; 1: NW at beat 2; 2: S at beat 2, N at beat 5; 3: NE at beat 1; 4: S at beat 4) so that the probe from (-5, 0) reaches the target at (4.5, 1.82); the choice is committed before the probe flies
  - 3 correct choices in a row complete the level; a wrong one resets the streak
  - run time limit: 20 s

## cmb-18 - Scout Through the Swirl (Combinations)

- objective: Survey + Predict: the background curves every flight. Scout the hidden source, then call the exit.
- hint 1: This background is not a straight stream: a swirl bends every flight. The small push arrows already subtract the swirl, so they show only what the hidden source did.
- hint 2: The swirl also bends the main probe, so do not call the exit from a straight line. Locate the source from the push arrows, then picture the probe flying through the swirl and the source together.
- hint 3: The hidden source is different every attempt: three right calls in a row.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 20 s; seed: 9150439843
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - uniform flow (1.29, -0.14)
  2. fixed - vortex about (-0.81, -0.8): v = k perp(d) / (|d|^2 + soft^2), k = -2.54, soft = 0.8 (clockwise)
- regions:
  0. target 'A' centre (4.5, 2.7) radius 1
  1. target 'B' centre (4.5, 0.9) radius 1
  2. target 'C' centre (4.5, -0.9) radius 1
  3. target 'D' centre (4.5, -2.7) radius 1
- rules:
  - mechanic: **Scout, then call (Survey + Predict)**
  - the main probe starts at (-5, 0)
  - the field layers above are the visible background; a hidden source is added to it in every flight
  - hidden sources (one is drawn at random each attempt, never the one just used; strength 2.01, width 0.8): attractor at (1.1, -0.58), repeller at (1.16, -1.87), repeller at (-1.4, 0.55)
  - up to 3 scouts per attempt, started anywhere in [-5.6, -4.4] x [-3, 3], each flying 6 s; the push each feels is shown (as in Survey)
  - then call which of 4 exits the main probe will take: A at (4.5, 2.7), B at (4.5, 0.9), C at (4.5, -0.9), D at (4.5, -2.7); the call is committed before the probe flies
  - 3 correct choices in a row complete the level; a wrong one resets the streak
  - run time limit: 20 s

## cmb-19 - Scout, Then Time It (Combinations)

- objective: Survey + Tempo: scout the hidden source, then choose when to launch the probe.
- hint 1: The background pulses with a twelve-second rhythm, and the probe is released at the phase you choose: 1 is phase 0, 2 is 2 s into the rhythm, and so on. Launching at phase 0 never works, whatever the hidden source.
- hint 2: Scouts fly from phase 0 and show you the hidden source (yellow arrows point toward an attractor). Work out which way it will pull the probe, then think about which part of the pulse leaves the probe where the source's pull can finish the job.
- hint 3: The hidden source changes every attempt, and each one has its own right phase. Three right choices in a row.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 20 s; seed: 9150439844
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - uniform flow (1.13, 0.2)
  2. fixed - modulated: [uniform flow (0, 0.88)] x (0 + 0.8 sin(0.5235987755982988 t + 0))
- regions:
  0. target 'T' centre (4.5, -0.29) radius 1
- rules:
  - mechanic: **Scout, then time it (Survey + Tempo)**
  - the main probe starts at (-5, 0) unless a choice moves it
  - the field layers above are the visible background; a hidden source is added to it in every flight
  - hidden sources (one is drawn at random each attempt, never the one just used; strength 2.01, width 0.8): attractor at (-1.84, 2.21), attractor at (-1.81, 0.61), attractor at (-2.68, -2.06)
  - up to 3 scouts per attempt, started anywhere in [-5.6, -4.4] x [-3, 3], each flying 6 s; the push each feels is shown (as in Survey)
  - then choose the launch phase of the main probe from 6 numbered options (0, 2, 4, 6, 8, 10 s; the first is the default), which must reach the target at (4.5, -0.29); the choice is committed before the probe flies
  - 3 correct choices in a row complete the level; a wrong one resets the streak
  - run time limit: 20 s

## cmb-29 - Scout, Then Time It II (Combinations)

- objective: Survey + Tempo: scout the hidden source (an attractor or a repeller), then choose when to launch the probe.
- hint 1: The background pulses with a twelve-second rhythm, and the probe is released at the phase you choose: 1 is phase 0, 2 is half a second into the rhythm, and so on. Launching at phase 0 never works, whatever the hidden source.
- hint 2: Scouts fly from phase 0 and show you the hidden source. The yellow arrows point toward an attractor and away from a repeller, and the kind changes which way the source pushes the probe, so it changes which part of the pulse can make up for it.
- hint 3: The hidden source changes every attempt, and each one has its own stretch of good phases, several launches wide. Aim for the middle of it. Three right choices in a row.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 20 s; seed: 9150439875
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - uniform flow (1.37, 0.2)
  2. fixed - modulated: [uniform flow (0, 1.14)] x (0 + 0.66 sin(0.5235987755982988 t + 3.09))
- regions:
  0. target 'T' centre (4.5, -0.66) radius 1
- rules:
  - mechanic: **Scout, then time it (Survey + Tempo)**
  - the main probe starts at (-5, 0) unless a choice moves it
  - the field layers above are the visible background; a hidden source is added to it in every flight
  - hidden sources (one is drawn at random each attempt, never the one just used; strength 3, width 0.8): repeller at (-2, 1), repeller at (2, -2), attractor at (-3, 0)
  - up to 3 scouts per attempt, started anywhere in [-5.6, -4.4] x [-3, 3], each flying 6 s; the push each feels is shown (as in Survey)
  - then choose the launch phase of the main probe from 24 numbered options (0, 0.5, 1, 1.5, 2, 2.5, 3, 3.5, 4, 4.5, 5, 5.5, 6, 6.5, 7, 7.5, 8, 8.5, 9, 9.5, 10, 10.5, 11, 11.5 s; the first is the default), which must reach the target at (4.5, -0.66); the choice is committed before the probe flies
  - 3 correct choices in a row complete the level; a wrong one resets the streak
  - run time limit: 20 s

## cmb-20 - Cancel and Launch (Combinations)

- objective: Survey + Shape + Position: scout the hidden source, add a source to cancel it, and choose where to release the probe.
- hint 1: Two groups of choices: a source to add (orange plus: a repeller, blue minus: an attractor, or none) and a release slot (numbered diamonds). Commit them together. The first source and the first slot are the defaults and are never enough.
- hint 2: Scout the hidden source and read its kind from the yellow arrows, then add the opposite kind in the same place so that the two push cancel. That alone does not get the probe to the target.
- hint 3: With the hidden push gone, the swirl and the stream still decide where the probe ends up. Release it from a slot that lets them carry it onto the target; fly a scout from a slot to see. Three right commits in a row.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 20 s; seed: 9150439866
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - uniform flow (1.2, 0.21)
  2. fixed - vortex about (0.94, -1.43): v = k perp(d) / (|d|^2 + soft^2), k = 0.46, soft = 0.8 (counter-clockwise)
- regions:
  0. target 'T' centre (4.5, 1.63) radius 0.9
- rules:
  - mechanic: **Scout, then act in several steps (Survey plus more than one other mechanic)**
  - the main probe starts at (-5, 0) unless a choice moves it
  - the field layers above are the visible background; a hidden source is added to it in every flight
  - hidden sources (one is drawn at random each attempt, never the one just used; strength 3, width 0.8): attractor at (-2.29, 1.2), repeller at (-0.88, 0.28), attractor at (1.96, 1.4)
  - up to 3 scouts per attempt, started anywhere in [-5.6, -4.4] x [-3, 3], each flying 6 s; the push each feels is shown (as in Survey)
  - then, one after another: (1) choose one source to add from 9 numbered options, the last being none (1 repeller at (-2.29, 1.2), 2 repeller at (-0.88, 0.28), 3 repeller at (0.61, -1.56), 4 repeller at (1.96, 1.4), 5 attractor at (-2.29, 1.2), 6 attractor at (-0.88, 0.28), 7 attractor at (0.61, -1.56), 8 attractor at (1.96, 1.4)), same strength and width as the hidden ones; the probe from (-5, 0) must reach the target at (4.5, 1.63); (2) choose the release slot of the main probe from 8 numbered slots (1 at (-5, -3.3), 2 at (-5, -2.47), 3 at (-5, -1.65), 4 at (-5, -0.71), 5 at (-5, 0.07), 6 at (-5, 0.77), 7 at (-5, 1.59), 8 at (-5, 2.36)), which must reach the target at (4.5, 1.63); all the choices are committed together and one flight judges them
  - 3 correct choices in a row complete the level; a wrong one resets the streak
  - run time limit: 20 s

## cmb-21 - Cancel and Steer (Combinations)

- objective: Survey + Shape + Steer: scout the hidden source, add a source to cancel it, and choose the gust plan that finishes the job.
- hint 1: Two groups of choices: a source to add (orange plus: a repeller, blue minus: an attractor, or none) and a gust plan (or none). Commit them together. Neither none is ever enough.
- hint 2: Scout the hidden source, read its kind from the yellow arrows, and add the opposite kind in the same place to cancel it. That alone leaves the probe off the target.
- hint 3: With the hidden push gone, the swirl and the stream still carry the probe past the target. Of the four gust plans, some finish the job. Fly a scout to see where the visible field takes a probe, then pick the plan that corrects it. Three right commits in a row.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 20 s; seed: 9150439867
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - uniform flow (1, 0.23)
  2. fixed - vortex about (-0.35, 1.03): v = k perp(d) / (|d|^2 + soft^2), k = 2.86, soft = 0.8 (counter-clockwise)
- regions:
  0. target 'T' centre (4.5, 0.27) radius 0.8
- rules:
  - mechanic: **Scout, then act in several steps (Survey plus more than one other mechanic)**
  - the main probe starts at (-5, 0) unless a choice moves it
  - the field layers above are the visible background; a hidden source is added to it in every flight
  - hidden sources (one is drawn at random each attempt, never the one just used; strength 3, width 0.8): repeller at (-2.13, -1.45), repeller at (0.48, -1.51), attractor at (2.33, 0)
  - up to 3 scouts per attempt, started anywhere in [-5.6, -4.4] x [-3, 3], each flying 6 s; the push each feels is shown (as in Survey)
  - then, one after another: (1) choose one source to add from 9 numbered options, the last being none (1 repeller at (-2.13, -1.45), 2 repeller at (-1, -1.34), 3 repeller at (0.48, -1.51), 4 repeller at (2.33, 0), 5 attractor at (-2.13, -1.45), 6 attractor at (-1, -1.34), 7 attractor at (0.48, -1.51), 8 attractor at (2.33, 0)), same strength and width as the hidden ones; the probe from (-5, 0) must reach the target at (4.5, 0.27); (2) choose one of 5 fixed gust plans, the last being no gust (strength 1, 1 s per beat; 1: N at beat 3; 2: E at beat 1; 3: SE at beat 4; 4: S at beat 1) so that the probe from (-5, 0) reaches the target at (4.5, 0.27); all the choices are committed together and one flight judges them
  - 3 correct choices in a row complete the level; a wrong one resets the streak
  - run time limit: 20 s

## cmb-25 - Cancel and Dial (Combinations)

- objective: Survey + Shape + Experiment: scout the hidden source, add a source to cancel it, and set the dial that brings the probe to the target.
- hint 1: Two groups of choices: a source to add (orange plus: a repeller, blue minus: an attractor, or none) and a dial setting. Commit them together. The first dial setting and "none" are never enough.
- hint 2: Scout the hidden source, read its kind from the yellow arrows, and add the opposite kind in the same place to cancel it. That alone leaves the probe off the target at the first dial setting.
- hint 3: With the hidden push gone the visible field decides. The dial scales only its tunable drift (the hidden source and the one you add ignore it), so some settings carry the probe onto the target and others do not. Fly a scout to see where the visible field takes a probe. Three right commits in a row.
- bounds: x in [-6, 6], y in [-4, 4]; simulation time limit: 20 s; seed: 9150439871
- field layers (summed; a *tunable* layer is multiplied by the intensity):
  1. fixed - uniform flow (1.16, 0.15)
  2. tunable - uniform flow (-0.21, -0.38)
  3. tunable - vortex about (0.09, 0.08): v = k perp(d) / (|d|^2 + soft^2), k = 0.13, soft = 0.8 (counter-clockwise)
  (default intensity 1)
- regions:
  0. target 'T' centre (4.5, -0.68) radius 1
- rules:
  - mechanic: **Scout, then act in several steps (Survey plus more than one other mechanic)**
  - the main probe starts at (-5, 0) unless a choice moves it
  - the field layers above are the visible background; a hidden source is added to it in every flight
  - hidden sources (one is drawn at random each attempt, never the one just used; strength 3, width 0.8): attractor at (-1.06, 0.73), repeller at (0.82, 0.11), attractor at (1.89, -1.74)
  - up to 3 scouts per attempt, started anywhere in [-5.6, -4.4] x [-3, 3], each flying 6 s; the push each feels is shown (as in Survey)
  - then, one after another: (1) choose one source to add from 9 numbered options, the last being none (1 repeller at (-2.33, -1.31), 2 repeller at (-1.06, 0.73), 3 repeller at (0.82, 0.11), 4 repeller at (1.89, -1.74), 5 attractor at (-2.33, -1.31), 6 attractor at (-1.06, 0.73), 7 attractor at (0.82, 0.11), 8 attractor at (1.89, -1.74)), same strength and width as the hidden ones; the probe from (-5, 0) must reach the target at (4.5, -0.68); (2) choose the dial setting from 6 numbered options (1, 0.5, 0.75, 1.25, 1.5, 2; the first is the default; hidden sources ignore the dial), so that the probe reaches the target at (4.5, -0.68); all the choices are committed together and one flight judges them
  - 3 correct choices in a row complete the level; a wrong one resets the streak
  - run time limit: 20 s
