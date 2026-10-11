# Obelus

A code editor that fits how I work, **with the agent built in** — in a
terminal.

It is not trying to be a powerful editor: no multiple cursors, no macros.
In the AI era most lines you read are lines you did not write, so much of
the time in it is spent reading what an agent changed, beside the code it
changed. The point is to join what a language server
knows about the code to what git knows about its history — jump to a definition
from inside a diff, ask for a symbol's history rather than a file's — which no
terminal tool does today.

An *obelus* (÷, †) is the mark a scholar put in a manuscript's margin to flag a
line as doubtful. A reader's mark, made while reading.

[obelus-editor.github.io/obelus](https://obelus-editor.github.io/obelus/) — what it does,
how to use it, and why it is shaped this way. Also in
[中文](https://obelus-editor.github.io/obelus/zh/).

## Getting it

Linux, macOS and Windows, on x86_64 and ARM64.

**macOS / Linux**

```sh
curl -fsSL https://raw.githubusercontent.com/obelus-editor/obelus/master/contrib/install.sh | sh
```

**Windows PowerShell**

```powershell
irm https://raw.githubusercontent.com/obelus-editor/obelus/master/contrib/install.ps1 | iex
```

**mise**

```sh
mise use -g github:obelus-editor/obelus
```

The two scripts take the latest release, check what they downloaded against
the release's own `SHA256SUMS`, and put **`ob`** on your PATH. Everything
after `-s --` goes to the shell script — `--dir` says where to put it, and
`--help` lists the rest:

```sh
curl -fsSL .../contrib/install.sh | sh -s -- --dir /usr/local/bin
```

On Linux the script also adds a desktop entry, which opens `ob` in a
terminal, and an icon under `~/.local/share`, which every launcher reads:
omarchy's `Apps` menu among them. `--no-desktop` leaves them out. On Windows
the script adds its directory to your PATH. Installing over an older
release that put `obg`, the window, there takes `obg` and its launcher
entry away, and says so.

`--uninstall` (`-Uninstall` on Windows), with the same `--dir` it was
installed with, takes back what the script put there: `ob` — and `obg`, the
window an older release installed — the desktop entries and their icon,
and, on omarchy, the theme template and the link to what omarchy renders. On
Windows it also takes back `obg`'s Start menu entry, and the directory and
its PATH entry where nothing else is left in it. Neither script touches your
settings.

mise installs only the executable: no desktop entry and no icon. Upgrade it
with `mise upgrade`.

[The releases](https://github.com/obelus-editor/obelus/releases) carry an
archive for every platform as well as a `.deb` and an `.rpm`.

```
ob                  # the welcome screen, on this directory
ob src/main.rs      # read a file — its repository is the project
ob ../another-repo  # a directory: that is the project
```

A language server is optional, and without one reading still works. So is a
Nerd Font: Obelus draws plain characters in place of its marks unless the
`icons` setting is on.

## Building

```
cargo build --workspace                  # stable
cargo +nightly fmt --all                 # nightly only: the rustfmt options require it
cargo clippy --workspace --all-features --all-targets
cargo test --workspace
UPDATE_FIXTURES=1 cargo test --workspace # after an intended rendering change
cargo test --workspace -- --ignored      # the slow real-server tests, and the diff sweep
```

Tests assert on the terminal cell grid, colours included: a widget can write
every character correctly and paint none of them, and an assertion on the text
cannot tell.
