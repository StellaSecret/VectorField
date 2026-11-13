//! The game must be playable with a mouse alone. Every key the front end listens for is either a shortcut for something
//! that has an on-screen button (and the button's label is checked to exist), or one of a few documented exceptions
//! that the mouse covers another way. A new keyboard-only control fails this test until it gets a button.
use std::collections::BTreeSet;

const APP: &str = include_str!("../src/app.rs");

/// Keys that duplicate an on-screen button, with the label of that button.
const WITH_BUTTON: &[(&str, &str)] = &[
    ("Escape", "Back (Esc)"),
    ("Escape", "Menu (Esc)"),
    ("Space", "Launch (Space)"),
    ("Enter", "Launch (Space)"),
    ("H", "Hint (H)"),
    ("R", "Retry (R)"),
    ("V", "Replay (V)"),
    ("N", "Next (N)"),
    ("G", "Arrows: "),
    ("K", "Contrast: "),
    ("Y", "Marker kind (Y)"),
    ("T", "Test estimate (T)"),
    ("C", "Commit"),
    ("S", "Scout (S)"),
    ("M", "Marker (M)"),
    ("P", "Guide lines (P)"),
    ("Delete", "Remove (Del)"),
    ("Backspace", "Remove (Del)"),
    ("X", "Remove (Del)"),
    ("Key1", "i + 1, k.short()"), // the numbered kind buttons of Shape (and the probe rows of Synchronize)
    ("Key2", "i + 1, k.short()"), // the numbered kind buttons of Shape (and the probe rows of Synchronize)
    ("Key3", "i + 1, k.short()"), // the numbered kind buttons of Shape (and the probe rows of Synchronize)
    ("Key4", "i + 1, k.short()"), // the numbered kind buttons of Shape (and the probe rows of Synchronize)
    ("Tab", "Marker (M)"),
];

/// Keys the mouse replaces another way (nothing is lost without them), and modifiers.
const MOUSE_COVERS: &[(&str, &str)] = &[
    ("Left", "arrow keys nudge a delay, move a cursor or scrub a value: the -/+ buttons, sliders, clicking and dragging do the same"),
    ("Right", "as Left"),
    ("Up", "as Left"),
    ("Down", "as Left"),
    ("Minus", "a slider adjusts the same value"),
    ("Equal", "a slider adjusts the same value"),
    ("Comma", "a slider adjusts the same value"),
    ("Period", "a slider adjusts the same value"),
    ("LeftShift", "modifier for finer steps"),
    ("RightShift", "modifier for finer steps"),
    ("Unknown", "placeholder for 'no key'"),
    ("U", "the unlock-every-level cheat, not part of play"),
    // the Steer gust letters and its wait key: the 3 x 3 grid of buttons chooses the same actions
    ("Q", "gust button"),
    ("W", "gust button"),
    ("E", "gust button"),
    ("A", "gust button"),
    ("D", "gust button"),
    ("Z", "gust button"),
    ("Key0", "the Wait button"),
    ("Kp0", "the Wait button"),
    // Predict: letters choose a region; clicking the region does the same
    ("B", "click a region"),
];

fn keys_used() -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut rest = APP;
    while let Some(i) = rest.find("KeyCode::") {
        rest = &rest[i + "KeyCode::".len()..];
        let name: String = rest.chars().take_while(|c| c.is_ascii_alphanumeric()).collect();
        out.insert(name);
    }
    out
}

#[test]
fn every_key_the_game_listens_for_has_a_mouse_equivalent() {
    let known: BTreeSet<&str> = WITH_BUTTON.iter().map(|(k, _)| *k).chain(MOUSE_COVERS.iter().map(|(k, _)| *k)).collect();
    for k in keys_used() {
        assert!(known.contains(k.as_str()), "KeyCode::{k} is used with no on-screen equivalent: give it a button (and list it in tests/mouse_only.rs)");
    }
}

#[test]
fn the_buttons_that_replace_the_keys_exist() {
    for (key, label) in WITH_BUTTON {
        assert!(APP.contains(label), "{key}: no button labelled {label:?} in app.rs");
    }
}

#[test]
fn the_synchronize_replay_button_is_enabled_by_a_flight_not_only_by_an_animation() {
    // Replay used to be greyed out in Synchronize (it looked for an animation the mode does not use).
    assert!(APP.contains("Game::Sync(g) => !g.flights.is_empty() && !s.st.sync_playing"));
    assert!(APP.contains("Game::Sync(g) => sync_start_playback(g, &mut s.st)"));
}

#[test]
fn the_settings_buttons_are_drawn_after_the_game_frame() {
    // `end_area` repaints the header strip, so buttons drawn before it vanish (found by looking at the running game)
    let frame = APP.find("fn play_frame").unwrap();
    let body = &APP[frame..];
    let area = body.find("Game::Sync(g) => sync_frame").expect("the frame dispatch");
    let buttons = body.find("settings_buttons(inp, prefs, 996.0").expect("the settings buttons");
    assert!(buttons > area, "settings buttons must be drawn after the game frame");
}
