# Vector Field

A 2D puzzle game about steering a probe through a vector field, written in Rust
([macroquad](https://github.com/not-fl3/macroquad)). It is built around eight *different* kinds of reasoning (Survey, Shape and Synchronize are prototypes, see below):

| Mechanic | Question | What you decide | Skill |
|---|---|---|---|
| **Position** | Where should I start? | the release point(s) | spatial reasoning about initial conditions |
| **Predict** | Where will it go? | a committed forecast of the exit | mentally extrapolating flow |
| **Tempo** | When should I launch? | the launch phase of a changing field | reasoning about time-dependent geometry |
| **Experiment** | How does the system behave? | a parameter, then a *conclusion* | discovering a rule by controlled tests |
| **Steer** | What should I do next? | a gust (or nothing) every beat, with feedback | closed-loop control: planning from observed state |
| **Survey** *(prototype)* | Where is the hidden source? | where to release a few scouts, then one estimate | choosing what to observe to infer a hidden structure |
| **Shape** *(prototype)* | How should I change the field? | which sources to add, and where, then launch | creating the effect you want by changing the field |
| **Synchronize** *(prototype)* | How must these probes be coordinated? | when each of several probes is released | matching the timing of several trajectories (see docs/11-synchronize.md for how far this differs from Tempo) |

25 fundamental levels (5 for each of the first five mechanics), five Survey, four Shape and five Synchronize prototype levels and 6 combination levels. Shape's evidence is the same kind as Survey's: an automated validation against the real simulation, not playtests (see `docs/10-shape.md`). Survey's evidence so far is an ideal-solver validation, not playtests (see `docs/09-survey.md`). See [docs/DESIGN.md](docs/DESIGN.md) for the per-mechanic
design, and how the "these are really different" requirement is checked by automated tests.

## Run

```bash
cargo run --release            # desktop game
cargo test --release           # simulation, level and mechanic validation
cargo run --release --bin report [-- --detail]   # per-level validation numbers
cargo run --release --bin report -- --steer      # Steer validation report (see docs/STEER_VALIDATION.md)
cargo run --release --bin report -- --survey     # Survey validation report (see docs/SURVEY_VALIDATION.md)
cargo run --release --bin report -- --shape      # Shape validation report (see docs/SHAPE_VALIDATION.md)
cargo run --release --bin report -- --sync       # Synchronize validation report (see docs/SYNC_VALIDATION.md)
```

Controls: mouse; Space/Enter launch or confirm; in Steer, Q W E / A S D / Z X C pick a gust direction (S = wait) or use the arrow keys; arrow keys nudge; in Survey, `S` scout / `M` marker (or `Tab`), click or Space to place, `C` commit; in Shape, click to place a source, drag to move it, `1`-`4` or `Tab` pick the kind, `Del`/`X`/right-click remove, `P` place at the keyboard cursor, Space launch; in Synchronize, drag a release handle on the timeline (or 1-3 and Left / Right), Space launch; `H` hint; `R` retry/reset; `V` replay;
`G` arrow-grid density; `K` high-contrast arrows; `U` unlock all levels; `Esc` back.
Progress and display settings are saved automatically (see Saving below).


## Saving

Finished levels and the display settings (`G` arrow density, `K` high contrast) are saved automatically, so a player does not replay levels after closing the game. The desktop build writes `vfield_save.txt` in the per-user data folder (`~/.local/share/vfield` on Linux, `~/Library/Application Support/vfield` on macOS, `%APPDATA%\vfield` on Windows; `VFIELD_SAVE_DIR` overrides it). The web build uses the browser's `localStorage` (key `vfield-save`), so progress is per browser and is lost if site data is cleared. If saving fails, the menu says so. The unlock-all switch (`U`) is for one session and is not saved. A `vfield_progress.txt` from an earlier build in the working directory is read once, as the starting point of the new save. To start over, delete the file (or clear the site's data). Format and tests: `src/save.rs`, `tests/save.rs`.

## Documentation

[`docs/`](docs/README.md) is a precise, code-derived reference for each mechanic (Position, Predict, Tempo, Experiment, Steer, Survey): information, controls, state machine, exact rules, feedback, level properties, known weaknesses and the tests that pin them. [`docs/DESIGN.md`](docs/DESIGN.md) is the design rationale. The level catalog (`docs/LEVEL_CATALOG.md`, rendered by `cargo run --release --bin catalog > docs/LEVEL_CATALOG.md`; a test fails if it is stale) and the Steer and Survey validation reports (`docs/STEER_VALIDATION.md`, `docs/SURVEY_VALIDATION.md`) are the files the code produces.

## Layout

```
src/field.rs        field model v(x, y, t; theta)      src/sim.rs      RK4 engine, regions, outcomes
src/level.rs        level data types                   src/levels.rs   all content, as data
src/modes/          the eight gameplay modes (no UI)    src/validate.rs automated mechanic checks
src/app.rs, render.rs, ui.rs, main.rs  macroquad front end        tests/mechanics.rs  validation suite
src/save.rs, storage.rs  saved progress (file / browser)       tests/             one test file per area
web/index.html      the web page and its save plugin   docs/              documentation
```

## CI/CD (`.github/workflows/build.yml`)

Secret scan (TruffleHog) and supply-chain scan (cargo-deny, config in `deny.toml`) gate everything.
`rust-core` runs the tests, the validation report and a `wasm32` check. `web-build` produces the WASM bundle
that `deploy-web` publishes to GitHub Pages. On `main`, `build-desktop` builds Linux, Windows and macOS
binaries and `release` publishes them plus the web bundle as `build-N`.

One-time setup: enable **Settings -> Pages -> Source: GitHub Actions**, and create a `release` environment.
