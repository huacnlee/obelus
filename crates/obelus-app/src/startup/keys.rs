//! `ob keys` and `obg keys`: which keys Obelus starts from, asked and
//! chosen without opening anything.
//!
//! Before either front end takes the screen, and said on stderr: stdout is
//! the drawing surface, and a word left there by a command that draws
//! nothing is still a word on it for whoever pipes one into the other.

use std::{io::Write, path::Path};

use anyhow::{Context as _, Result};

/// The words `keys set` takes, which are the settings file's own.
pub const PRESETS: &[&str] = obelus_config::KEY_LAYOUTS;

/// Says which layout the reader's keys start from, or chooses one, in the
/// reader's own settings.
///
/// `program` is the name the reader typed, so what to type next is said in
/// it.
pub fn run(program: &str, word: Option<&str>) -> Result<()> {
    let path = obelus_config::path().context("this system has nowhere to keep settings")?;
    run_at(&path, program, word, &mut std::io::stderr())
}

/// The same, on a settings file the caller names and saying it into
/// `said`, so a test can do it without the reader's file or a terminal.
pub fn run_at(path: &Path, program: &str, word: Option<&str>, said: &mut impl Write) -> Result<()> {
    if let Some(word) = word {
        obelus_config::write_keymap(path, word)
            .with_context(|| format!("could not choose {word} in {}", path.display()))?;
    }
    let config = match obelus_config::read_from(path) {
        obelus_config::Reading::Settings { config, .. } => *config,
        obelus_config::Reading::Nothing | obelus_config::Reading::Nowhere => {
            obelus_config::Config::default()
        }
        obelus_config::Reading::Unreadable(why, _) => {
            anyhow::bail!("could not read {}: {why}", path.display())
        }
    };
    let current = config.keys_from.as_str();
    let name = obelus_config::key_layout(current).map_or(current, |layout| layout.name);
    match word {
        Some(_) => writeln!(said, "Keys start from {name} now")?,
        None => writeln!(said, "Keys start from {name}")?,
    }
    // A reader's own keys are laid over whichever it is, so a switch keeps
    // them -- and a reader who expected the layout's key on a command they
    // had moved is owed the reason.
    if !config.keys.is_empty() {
        writeln!(said, "Your own keys in {} are laid over it", path.display())?;
    }
    if word.is_none() {
        writeln!(said)?;
        let widest = PRESETS.iter().map(|word| word.len()).max().unwrap_or(0);
        for layout in obelus_config::KEY_LAYOUT_CHOICES {
            let mark = if layout.word == current { '*' } else { ' ' };
            writeln!(
                said,
                "{mark} {:widest$}  {}: {}",
                layout.word, layout.name, layout.about
            )?;
        }
        writeln!(said)?;
        writeln!(said, "Choose one with {program} keys set <layout>")?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::run_at;

    fn scratch(name: &str) -> PathBuf {
        let directory =
            std::env::temp_dir().join(format!("obelus-keys-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&directory);
        directory
    }

    /// Choosing writes the layout into the file named and says so, keeping
    /// the reader's own keys; asking says which one is in force, by name,
    /// and every one there is.
    ///
    /// Broken deliberately by not writing in `run_at` (the file never
    /// named the layout), and by marking the first layout rather than the
    /// one in force (`obelus` wore the mark after `vscode` was chosen).
    #[test]
    fn choosing_and_asking_go_through_the_file() {
        let directory = scratch("choosing");
        let file = directory.join("config.toml");
        std::fs::create_dir_all(&directory).expect("a directory");
        std::fs::write(&file, "[keys]\nopen-file = \"Ctrl+o\"\n").expect("the file");

        let mut said = Vec::new();
        run_at(&file, "ob", Some("vscode"), &mut said).expect("choosing");
        let said = String::from_utf8(said).expect("words");
        assert!(said.starts_with("Keys start from VS Code now\n"), "{said}");
        assert!(said.contains("Your own keys"), "{said}");
        let config = obelus_config::from_toml(&std::fs::read_to_string(&file).expect("the file"));
        assert_eq!(config.keys_from, "vscode");
        assert_eq!(
            config.keys.get("open-file").map(String::as_str),
            Some("Ctrl+o")
        );

        let mut said = Vec::new();
        run_at(&file, "obg", None, &mut said).expect("asking");
        let said = String::from_utf8(said).expect("words");
        assert!(said.starts_with("Keys start from VS Code\n"), "{said}");
        assert!(said.contains("\n* vscode    VS Code: "), "{said}");
        assert!(said.contains("\n  obelus    Obelus: "), "{said}");
        assert!(said.contains("\n  textmate  TextMate: "), "{said}");
        assert!(said.contains("obg keys set <layout>"), "{said}");
        let _ = std::fs::remove_dir_all(&directory);
    }

    /// Asking where there is no file says the default and writes nothing,
    /// and a file that will not read is an error that leaves it alone.
    ///
    /// Broken deliberately by choosing in `run_at` whatever the word (an
    /// ask wrote a file), and by treating an unreadable file as the
    /// defaults (the ask said `Obelus` about a file it could not read).
    #[test]
    fn asking_writes_nothing_and_a_broken_file_is_said() {
        let directory = scratch("asking");
        let file = directory.join("config.toml");
        let mut said = Vec::new();
        run_at(&file, "ob", None, &mut said).expect("asking");
        assert!(
            String::from_utf8(said)
                .expect("words")
                .starts_with("Keys start from Obelus\n")
        );
        assert!(!file.exists(), "asking wrote the file");

        std::fs::create_dir_all(&directory).expect("a directory");
        std::fs::write(&file, "keys_from = [").expect("the file");
        assert!(run_at(&file, "ob", None, &mut Vec::new()).is_err());
        assert!(run_at(&file, "ob", Some("textmate"), &mut Vec::new()).is_err());
        assert_eq!(
            std::fs::read_to_string(&file).expect("the file"),
            "keys_from = ["
        );
        let _ = std::fs::remove_dir_all(&directory);
    }
}
