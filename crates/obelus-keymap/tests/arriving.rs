//! Which key is shown where control and shift on a letter do not arrive, in
//! a binary of its own: the switch is a global, and a test beside the
//! others that turned it off would change what `chord_for` says underneath
//! them.

use std::{collections::BTreeMap, sync::Mutex};

use crossterm::event::{KeyCode, KeyModifiers};
use obelus_command::Command;
use obelus_keymap::{KeyChord, Keymap, Layout, shifted_control_arrives};

/// Held by each test for as long as it has the switch moved: the tests in a
/// binary run at once, and two of them moving one global would be reading
/// each other's.
static SWITCH: Mutex<()> = Mutex::new(());

fn chord(key: &str) -> KeyChord {
    KeyChord::parse(key).expect("a key")
}

/// Where the terminal cannot send `ctrl+shift+p`, the palette is shown on
/// the alt chord beside it, and where it can, on the chord VS Code has --
/// and the keys themselves are the same either way.
///
/// Broken deliberately by having `chord_for` take the first binding again,
/// whatever arrives: the palette was shown on `Ctrl+Shift+p` with the
/// switch off.
#[test]
fn a_key_that_cannot_be_pressed_gives_way_to_one_that_can() {
    let _taken = SWITCH.lock().expect("the switch");
    let vscode = Keymap::laid_out(Layout::VsCode);

    shifted_control_arrives(false);
    let without = vscode.chord_for(Command::CommandPalette);
    shifted_control_arrives(true);
    let with = vscode.chord_for(Command::CommandPalette);

    assert_eq!(without, Some(chord("alt+shift+p")));
    assert_eq!(with, Some(chord("ctrl+shift+p")));
    // A key with no shift on control is shown the same either way.
    shifted_control_arrives(false);
    assert_eq!(vscode.chord_for(Command::FileOpen), Some(chord("ctrl+p")));
    // And the same key with shift on alt, which every terminal sends.
    assert!(
        vscode
            .lookup(
                &crossterm::event::KeyEvent::new(
                    KeyCode::Char('P'),
                    KeyModifiers::ALT | KeyModifiers::SHIFT
                ),
                obelus_keymap::Context::Normal
            )
            .is_some_and(|command| command == Command::CommandPalette)
    );
    shifted_control_arrives(true);
}

/// A command whose only key cannot be pressed here is still shown on it:
/// the reader wrote it, and seeing it is what says why it does nothing.
///
/// Broken deliberately by having `chord_for` answer `None` where nothing
/// arrives: undo was shown on no key at all.
#[test]
fn a_readers_only_key_is_shown_even_where_it_cannot_arrive() {
    let _taken = SWITCH.lock().expect("the switch");
    let lines = BTreeMap::from([("undo".to_string(), "ctrl+shift+u".to_string())]);
    let (keymap, unbound) = Keymap::with(Layout::Obelus, &lines);
    assert!(unbound.is_empty(), "the key was not bound");

    shifted_control_arrives(false);
    let shown = keymap.chord_for(Command::Undo);
    shifted_control_arrives(true);

    assert_eq!(shown, Some(chord("ctrl+shift+u")));
}
