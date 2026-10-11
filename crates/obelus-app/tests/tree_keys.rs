//! The file list's tree, by the keys every tree on a desktop answers to.
//!
//! Right opens a directory and goes into an open one; left shuts an open
//! one and goes up from anything else; `ctrl+left` shuts them all. Only
//! while the tree is showing, which is while nothing is typed: a query
//! makes the list flat, and the arrows are the caret's again.

mod support;

use crossterm::event::KeyCode;
use obelus_app::app::App;
use obelus_buffer::Buffer;

/// Two directories and a file, with the file open: the tree opens on it.
fn a_tree(name: &str) -> (support::Scratch, App) {
    let scratch = support::Scratch::new(name);
    let root = scratch.path();
    for directory in ["docs/guide", "src"] {
        std::fs::create_dir_all(root.join(directory)).expect("making it");
    }
    for file in ["docs/guide/start.md", "docs/index.md", "src/main.rs", "top.rs"] {
        std::fs::write(root.join(file), "one\ntwo\n").expect("writing it");
    }
    let mut app = App::new(vec![
        Buffer::open(&root.join("top.rs")).expect("opening it"),
    ]);
    app.working_directory_for_test(root.to_path_buf());
    app.statuses_for_test(std::collections::HashMap::new());
    support::lay_out(&mut app, 60, 30);
    support::press_function(&mut app, 1);
    (scratch, app)
}

fn standing_on(app: &App) -> String {
    app.picker()
        .and_then(obelus_component::picker::Picker::selected_item)
        .map(|item| item.label.clone())
        .unwrap_or_default()
}

/// The labels of the rows, in order.
fn rows(app: &App) -> Vec<String> {
    app.picker()
        .map(|picker| picker.matches().map(|item| item.label.clone()).collect())
        .unwrap_or_default()
}

/// Right opens a shut directory and stays on it, and goes into an open one.
///
/// Broken deliberately by making the `Some(false)` arm of `tree_key` do
/// nothing: `docs` stayed shut and its rows never appeared.
#[test]
fn right_opens_a_directory_and_then_goes_into_it() {
    let (_scratch, mut app) = a_tree("tree-keys-right");
    support::press_control_key(&mut app, KeyCode::Home);
    assert_eq!(standing_on(&app), "docs");
    assert_eq!(rows(&app), ["docs", "src", "top.rs"]);

    support::press(&mut app, KeyCode::Right);
    assert_eq!(rows(&app), ["docs", "guide", "index.md", "src", "top.rs"]);
    assert_eq!(standing_on(&app), "docs", "opening it moved the row");

    support::press(&mut app, KeyCode::Right);
    assert_eq!(standing_on(&app), "guide");
    // A file is as far right as there is.
    support::press(&mut app, KeyCode::Down);
    support::press(&mut app, KeyCode::Right);
    assert_eq!(standing_on(&app), "index.md");
}

/// Left shuts an open directory, and goes up from a file or a shut one.
///
/// Broken deliberately two ways. Making the `Some(true)` arm go up rather
/// than shut: `docs` stayed open. And looking for the parent with `<=`
/// rather than `<` on the depth: left from `index.md` landed on `guide`,
/// the row above it, rather than on `docs`.
#[test]
fn left_shuts_a_directory_and_goes_up_from_anything_else() {
    let (_scratch, mut app) = a_tree("tree-keys-left");
    support::press_control_key(&mut app, KeyCode::Home);
    support::press(&mut app, KeyCode::Right);
    support::press(&mut app, KeyCode::Down);
    support::press(&mut app, KeyCode::Down);
    assert_eq!(standing_on(&app), "index.md");

    support::press(&mut app, KeyCode::Left);
    assert_eq!(standing_on(&app), "docs");
    support::press(&mut app, KeyCode::Left);
    assert_eq!(rows(&app), ["docs", "src", "top.rs"], "left did not shut it");
    assert_eq!(standing_on(&app), "docs");
    // At the top, and shut: nowhere further out to go.
    support::press(&mut app, KeyCode::Left);
    assert_eq!(standing_on(&app), "docs");

    // And from a directory that is shut but inside another.
    support::press(&mut app, KeyCode::Right);
    support::press(&mut app, KeyCode::Down);
    assert_eq!(standing_on(&app), "guide");
    support::press(&mut app, KeyCode::Left);
    assert_eq!(standing_on(&app), "docs");
}

/// `ctrl+left` shuts everything, and the reader lands on the directory at
/// the top that they were in.
///
/// Broken deliberately by leaving out the `stood_on` it sets: the row went
/// to `top.rs`, the file being read, rather than to `docs`.
#[test]
fn control_left_shuts_them_all_and_keeps_the_reader_near() {
    let (_scratch, mut app) = a_tree("tree-keys-all");
    support::press_control_key(&mut app, KeyCode::Home);
    support::press(&mut app, KeyCode::Right);
    support::press(&mut app, KeyCode::Right);
    support::press(&mut app, KeyCode::Right);
    support::press(&mut app, KeyCode::Right);
    assert_eq!(standing_on(&app), "start.md");
    support::press_control_key(&mut app, KeyCode::Left);
    assert_eq!(rows(&app), ["docs", "src", "top.rs"]);
    assert_eq!(standing_on(&app), "docs");
}

/// With something typed the arrows are the caret's, and the tree does not
/// take them.
///
/// Broken deliberately by dropping `picker.query().is_empty()` from
/// `showing_the_tree`: left was taken by the list, and the `x` landed at
/// the end of the query instead of between its letters.
#[test]
fn a_query_keeps_its_arrows() {
    let (_scratch, mut app) = a_tree("tree-keys-query");
    support::type_text(&mut app, "ab");
    support::press(&mut app, KeyCode::Left);
    support::type_text(&mut app, "x");
    let query = app.picker().map(|picker| picker.query()).unwrap_or_default();
    assert_eq!(query, "axb");
}

/// A directory's row shows what is in it below the list, so walking the
/// tree does not flash the bottom of the screen blank on every directory.
///
/// Broken deliberately by putting `PickerValue::Directory(_)` back among
/// the rows with nothing to show in `preview_target`: the pane under the
/// list was empty on `docs`.
#[test]
fn a_directory_shows_what_is_in_it() {
    let (_scratch, mut app) = a_tree("tree-keys-preview");
    let on_a_file = support::render(&mut app, 60, 30);
    support::press_control_key(&mut app, KeyCode::Home);
    assert_eq!(standing_on(&app), "docs");
    let on_a_directory = support::render(&mut app, 60, 30);
    let below: Vec<&str> = support::text_block(&on_a_directory)
        .lines()
        .skip(12)
        .collect();
    let below = below.join("\n");
    assert!(below.contains("guide/"), "{on_a_directory}");
    assert!(below.contains("index.md"), "{on_a_directory}");
    // And nothing else on the screen moved to make room for it.
    let rules = |dump: &str| -> Vec<usize> {
        support::text_block(dump)
            .lines()
            .enumerate()
            .filter(|(_, row)| row.contains("\u{2500}\u{2500}\u{2500}\u{2500}"))
            .map(|(at, _)| at)
            .collect()
    };
    assert_eq!(rules(&on_a_file), rules(&on_a_directory));
}

/// Turning a switch about which files are offered leaves the reader on the
/// row they were on, rather than on the file being read.
///
/// Broken deliberately by taking out the `stood_on` `listing_key` sets
/// before it lists again: `alt+i` on `src` put the row on `top.rs`.
#[test]
fn a_switch_does_not_move_the_row() {
    let (scratch, mut app) = a_tree("tree-keys-switch");
    app.config_file_for_test(scratch.join("config.toml"));
    support::press_control_key(&mut app, KeyCode::Home);
    support::press(&mut app, KeyCode::Down);
    assert_eq!(standing_on(&app), "src");
    support::press_alt_key(&mut app, KeyCode::Char('i'));
    assert!(app.config().ignored_files, "the key did nothing");
    assert_eq!(standing_on(&app), "src");
}
