//! The tables Obelus ships with, read out of `keymaps/`.
//!
//! A layout is data, and the reasons stay beside the keys. A file rather
//! than a function, because what it says is which command is on which key,
//! in which context, and why; written as code it was six hundred lines of
//! `Binding { .. }` with the reasons between them; the reasons are still
//! between them, as `//` comments, which is why the files are JSON with
//! comments, named `.json` as VS Code's own `keybindings.json` is -- and why a
//! comma before a closing bracket is allowed, so a line added at the end of a
//! list is a line and not two.
//!
//! What a file may say:
//!
//! * `extends` -- the name of the layout it starts from;
//! * `move` -- commands that lose every key they were given and take these, in
//!   each context they were bound in;
//! * `bindings` -- one command, one context, one key each, put after the moves.
//!
//! A key a command takes, by either, is taken off whatever had it in that
//! context, so the first binding of a chord is never a stale one. The words
//! are the reader's: a command by the name `[keys]` uses, a key spelled the
//! way `[keys]` spells it, so a line here can be copied into the settings.
//! Case is nothing to either (`KeyChord::parse`); these files write keys
//! small, and shift as `shift+`.
//!
//! Compiled in and read once. A file that does not read is a bug in Obelus,
//! not in the reader's machine, and the test that reads every layout is what
//! keeps one from shipping.

use std::sync::OnceLock;

use obelus_command::Command;
use serde::Deserialize;

use crate::{Binding, Context, KeyChord, Keymap, Layout};

/// What a layout's file says, before any of its words are read.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Written {
    extends: Option<String>,
    #[serde(default, rename = "move")]
    moves: Vec<Moved>,
    #[serde(default)]
    bindings: Vec<Bound>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Moved {
    command: String,
    keys: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Bound {
    command: String,
    context: String,
    key: String,
}

/// The file a layout is written in.
pub(crate) const fn source(layout: Layout) -> &'static str {
    match layout {
        Layout::Classic => include_str!("../keymaps/classic.json"),
        Layout::Mnemonic => include_str!("../keymaps/mnemonic.json"),
        Layout::VsCode => include_str!("../keymaps/vscode.json"),
        Layout::TextMate => include_str!("../keymaps/textmate.json"),
    }
}

/// The table a layout's file makes, read the first time it is asked for.
pub(crate) fn table(layout: Layout) -> &'static [Binding] {
    static TABLES: [OnceLock<Vec<Binding>>; Layout::ALL.len()] =
        [const { OnceLock::new() }; Layout::ALL.len()];
    let index = Layout::ALL
        .iter()
        .position(|each| *each == layout)
        .unwrap_or_default();
    TABLES[index].get_or_init(|| {
        read(layout).unwrap_or_else(|why| panic!("keymaps/{}.json: {why}", layout.name()))
    })
}

/// The table a layout's file makes, or what is wrong with the file.
pub(crate) fn read(layout: Layout) -> Result<Vec<Binding>, String> {
    build(source(layout), Some(layout))
}

/// The table some text makes. `own` is the layout it is the file of, so one
/// that extends itself is refused rather than read for ever.
fn build(text: &str, own: Option<Layout>) -> Result<Vec<Binding>, String> {
    let written: Written =
        serde_json::from_str(&plain_json(text)).map_err(|error| error.to_string())?;
    let mut keymap = match &written.extends {
        Some(name) => {
            let base = Layout::named(name).ok_or_else(|| format!("no layout called {name}"))?;
            if Some(base) == own {
                return Err(format!("{name} extends itself"));
            }
            Keymap::from_bindings(read(base)?)
        }
        None => Keymap::from_bindings(Vec::new()),
    };

    let mut moves = Vec::new();
    for moved in &written.moves {
        let command = command(&moved.command)?;
        let chords = moved
            .keys
            .iter()
            .map(|key| chord(key))
            .collect::<Result<Vec<_>, _>>()?;
        moves.push((command, chords));
    }
    // Where each was bound, asked of the table before any of them moves: one
    // moving onto another's old key would otherwise take that context away
    // from it -- the files on `ctrl+p` took the palette out of the file it is
    // in, and left it the terminal's.
    let contexts: Vec<Vec<Context>> = moves
        .iter()
        .map(|(command, _)| {
            let mut contexts = Vec::new();
            for binding in &keymap.bindings {
                if binding.command == *command && !contexts.contains(&binding.context) {
                    contexts.push(binding.context);
                }
            }
            contexts
        })
        .collect();
    keymap
        .bindings
        .retain(|binding| !moves.iter().any(|(command, _)| *command == binding.command));
    for ((command, chords), contexts) in moves.iter().zip(&contexts) {
        if contexts.is_empty() {
            return Err(format!(
                "{} is moved but has no key to move",
                command.name()
            ));
        }
        for context in contexts {
            for chord in chords {
                keymap.put(*command, *context, *chord);
            }
        }
    }

    for bound in &written.bindings {
        let context = Context::named(&bound.context)
            .ok_or_else(|| format!("no context called {}", bound.context))?;
        keymap.put(command(&bound.command)?, context, chord(&bound.key)?);
    }
    Ok(keymap.bindings)
}

fn command(name: &str) -> Result<Command, String> {
    obelus_command::by_name(name).ok_or_else(|| format!("no command called {name}"))
}

fn chord(key: &str) -> Result<KeyChord, String> {
    KeyChord::parse(key).ok_or_else(|| format!("{key} is not a key"))
}

/// JSON with comments, as JSON: every comment and every comma before a
/// closing bracket turned into blanks, so what `serde_json` says about a line
/// is about the line in the file.
pub(crate) fn plain_json(text: &str) -> String {
    let mut out: Vec<char> = Vec::with_capacity(text.len());
    let mut characters = text.chars().peekable();
    let mut in_string = false;
    while let Some(character) = characters.next() {
        if in_string {
            out.push(character);
            match character {
                '\\' => out.extend(characters.next()),
                '"' => in_string = false,
                _ => {}
            }
            continue;
        }
        match (character, characters.peek()) {
            ('"', _) => {
                in_string = true;
                out.push(character);
            }
            ('/', Some('/')) => {
                out.push(' ');
                for next in characters.by_ref() {
                    if next == '\n' {
                        out.push('\n');
                        break;
                    }
                    out.push(' ');
                }
            }
            ('/', Some('*')) => {
                // The opening star is not half of a closing one: `/*/` has
                // not ended.
                characters.next();
                out.extend([' ', ' ']);
                let mut last = ' ';
                for next in characters.by_ref() {
                    out.push(if next == '\n' { '\n' } else { ' ' });
                    if last == '*' && next == '/' {
                        break;
                    }
                    last = next;
                }
            }
            (']' | '}', _) => {
                // The comma this closes over, if the last thing before it
                // was one.
                if let Some(at) = out.iter().rposition(|each| !each.is_whitespace())
                    && out[at] == ','
                {
                    out[at] = ' ';
                }
                out.push(character);
            }
            _ => out.push(character),
        }
    }
    out.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Broken deliberately by having `source` hand back `classic.json` for
    /// [`Layout::VsCode`], which still read and failed on the keys.
    #[test]
    fn every_layout_reads() {
        for layout in Layout::ALL {
            let table = read(layout).unwrap_or_else(|why| panic!("{layout:?}: {why}"));
            assert!(!table.is_empty(), "{layout:?}");
        }
        let has = |layout, command, key| {
            let chord = KeyChord::parse(key).unwrap();
            read(layout)
                .unwrap()
                .iter()
                .any(|binding| binding.command == command && binding.chord == chord)
        };
        assert!(has(Layout::Classic, Command::FileOpen, "f1"));
        assert!(has(Layout::VsCode, Command::FileOpen, "ctrl+p"));
        assert!(has(Layout::TextMate, Command::FileOpen, "ctrl+t"));
        assert!(!has(Layout::VsCode, Command::FileOpen, "f1"));
    }

    /// Broken deliberately three ways, each failing it: leaving the comma in
    /// place, which `serde_json` refused at the closing bracket; never
    /// entering a string, which cut the URL at its `//`; and letting the
    /// opening star of `/*` count as half of the closing `*/`.
    #[test]
    fn comments_and_a_last_comma_are_blanks() {
        let text = "// a\n{ \"a\": \"http://x\", /* b */ \"c\": [1, 2,], // d\n}";
        let value: serde_json::Value = serde_json::from_str(&plain_json(text)).unwrap();
        assert_eq!(value["a"], "http://x");
        assert_eq!(value["c"], serde_json::json!([1, 2]));
        // A line of the file is still that line.
        assert_eq!(plain_json(text).lines().count(), text.lines().count());
        // `/*/` has not ended the comment.
        assert_eq!(plain_json("/*/ x */1").trim(), "1");
    }

    /// Broken deliberately by skipping the check for a moved command with no
    /// context, which bound nothing and read as though it had been moved;
    /// and, apart, by reading an unknown context as `normal`.
    #[test]
    fn a_mistake_in_a_file_is_said_and_not_skipped() {
        let fails = |text: &str| build(text, None).unwrap_err();
        // `new-file` has no key in the classic table.
        let moved =
            r#"{ "extends": "classic", "move": [{ "command": "new-file", "keys": ["alt+z"] }] }"#;
        assert!(fails(moved).contains("no key to move"), "{}", fails(moved));
        let context =
            r#"{ "bindings": [{ "command": "redo", "context": "nowhere", "key": "ctrl+y" }] }"#;
        assert!(fails(context).contains("no context called nowhere"));
        let command =
            r#"{ "bindings": [{ "command": "redoo", "context": "normal", "key": "ctrl+y" }] }"#;
        assert!(fails(command).contains("no command called redoo"));
        let key =
            r#"{ "bindings": [{ "command": "redo", "context": "normal", "key": "ctrl+yy" }] }"#;
        assert!(fails(key).contains("is not a key"));
        assert!(build(r#"{ "extends": "classic" }"#, Some(Layout::Classic)).is_err());
        assert!(build(r#"{ "binding": [] }"#, None).is_err());
    }
}
