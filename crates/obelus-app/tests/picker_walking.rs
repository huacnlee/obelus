//! The letters with control that walk a list.
//!
//! `ctrl+n` and `ctrl+p` a row, `ctrl+d` and `ctrl+u` a screenful, in any
//! list with a query -- and a list is in front, so the same chords that open
//! the palette, the files or the pull requests in the file behind it mean
//! the list's own thing while it is up, and what the key table says they
//! mean again once it is gone.

mod support;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use obelus_app::app::App;
use obelus_buffer::Buffer;
use obelus_command::Command;
use obelus_keymap::{Keymap, Layout};

/// A project of thirty files at the root, with the eleventh open.
fn a_project(name: &str) -> (support::Scratch, App) {
    let scratch = support::Scratch::new(name);
    let root = scratch.path();
    let body: String = (0..40).map(|n| format!("line {n}\n")).collect();
    for n in 0..30 {
        std::fs::write(root.join(format!("a{n:02}.rs")), &body).expect("writing it");
    }
    let mut app = App::new(vec![
        Buffer::open(&root.join("a10.rs")).expect("opening it"),
    ]);
    app.working_directory_for_test(root.to_path_buf());
    app.statuses_for_test(std::collections::HashMap::new());
    support::lay_out(&mut app, 60, 34);
    (scratch, app)
}

fn standing_on(app: &App) -> String {
    app.picker()
        .and_then(obelus_component::picker::Picker::selected_item)
        .map(|item| item.label.clone())
        .unwrap_or_default()
}

fn which_list(app: &App) -> Option<Command> {
    app.picker()
        .and_then(obelus_component::picker::Picker::opener)
}

fn is_the_file_list(app: &App) -> bool {
    app.picker()
        .is_some_and(obelus_component::picker::Picker::is_listing)
}

/// In the file list's tree, each of the four moves the row, and `ctrl+p`
/// does not open the palette over it -- which is what it means in a file.
///
/// Broken deliberately by answering `None` from `Move::in_picker`: `ctrl+n`
/// left the row on `a10.rs`, and `ctrl+p` put the palette in front.
#[test]
fn the_four_walk_the_file_list_and_the_palette_stays_shut() {
    let (_scratch, mut app) = a_project("walking-files");
    support::press_function(&mut app, 1);
    assert_eq!(standing_on(&app), "a10.rs");

    support::press_control(&mut app, 'n');
    assert_eq!(standing_on(&app), "a11.rs");
    support::press_control(&mut app, 'p');
    support::press_control(&mut app, 'p');
    assert_eq!(standing_on(&app), "a09.rs");
    assert!(is_the_file_list(&app), "ctrl+p left the list");

    // A whole screenful each way, which is ten rows here and not five.
    support::press_control(&mut app, 'd');
    assert_eq!(standing_on(&app), "a19.rs");
    support::press_control(&mut app, 'u');
    assert_eq!(standing_on(&app), "a09.rs");
    // And held at the ends rather than wrapping past them.
    support::press_control(&mut app, 'u');
    support::press_control(&mut app, 'u');
    assert_eq!(standing_on(&app), "a00.rs");
    support::press_control(&mut app, 'd');
    support::press_control(&mut app, 'd');
    support::press_control(&mut app, 'd');
    support::press_control(&mut app, 'd');
    assert_eq!(standing_on(&app), "a29.rs");
}

/// The letters themselves are typed, and the query outlives the walking.
///
/// Broken deliberately by letting `Move::in_picker` take a bare letter as
/// well: the query came out empty and the list never narrowed.
#[test]
fn the_letters_are_typed_and_the_query_is_kept() {
    let (_scratch, mut app) = a_project("walking-query");
    // The flat list a query is asked against is the walk's, which answers
    // on the channel.
    let events = support::drive(&mut app);
    support::press_function(&mut app, 1);
    support::type_text(&mut app, "a2");
    let until = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while app.picker().is_some_and(|picker| picker.match_count() < 10)
        && let Ok(event) =
            events.recv_timeout(until.saturating_duration_since(std::time::Instant::now()))
    {
        app.handle(event);
    }
    let query = |app: &App| {
        app.picker()
            .map(|picker| picker.query())
            .unwrap_or_default()
    };
    assert_eq!(query(&app), "a2");
    let first = standing_on(&app);
    support::press_control(&mut app, 'n');
    assert_ne!(standing_on(&app), first, "ctrl+n did not move");
    support::press_control(&mut app, 'd');
    support::press_control(&mut app, 'u');
    support::press_control(&mut app, 'p');
    assert_eq!(query(&app), "a2");
    // What the four are with no control held.
    support::type_text(&mut app, "npdu");
    assert_eq!(query(&app), "a2npdu");
}

/// Only control, exactly: `ctrl+shift+n` and `super+n` are not a step.
///
/// Broken deliberately by testing `contains(CONTROL)` rather than equality
/// in `Move::in_picker`: `ctrl+shift+n` moved the row.
#[test]
fn another_modifier_held_is_not_a_step() {
    let (_scratch, mut app) = a_project("walking-exact");
    support::press_function(&mut app, 1);
    for modifiers in [
        KeyModifiers::CONTROL | KeyModifiers::SHIFT,
        KeyModifiers::CONTROL | KeyModifiers::ALT,
        KeyModifiers::CONTROL | KeyModifiers::SUPER,
        KeyModifiers::SUPER,
    ] {
        app.handle(obelus_app::event::Event::Key(KeyEvent::new(
            KeyCode::Char('n'),
            modifiers,
        )));
        assert_eq!(standing_on(&app), "a10.rs", "{modifiers:?} moved the row");
    }
}

/// Once the list has gone, `ctrl+p` is the palette again: the list took
/// the chord while it was in front, and gave it back.
///
/// Broken deliberately by answering `None` from `Move::in_picker`: the
/// palette was swapped in for the file list on the first `ctrl+p`. The
/// half after escape is the other side of the same rule and is not broken
/// from the list's end: nothing the list does reaches the key table once it
/// has gone.
#[test]
fn out_of_the_list_ctrl_p_is_the_palette_again() {
    let (_scratch, mut app) = a_project("walking-back");
    support::press_function(&mut app, 1);
    support::press_control(&mut app, 'p');
    assert!(is_the_file_list(&app));
    support::press(&mut app, KeyCode::Esc);
    assert!(app.picker().is_none(), "escape did not leave");
    support::press_control(&mut app, 'p');
    assert_eq!(which_list(&app), Some(Command::CommandPalette));
}

/// A search's list walks the same way, and `ctrl+p` in it is a step rather
/// than the palette swapped in for the search.
///
/// Broken deliberately by answering `None` from `Move::in_picker`: the
/// search was put away and the palette came up in its place.
#[test]
fn a_search_is_walked_the_same_way() {
    let (_scratch, mut app) = a_project("walking-search");
    support::press_function(&mut app, 5);
    support::type_text(&mut app, "line");
    let first = standing_on(&app);
    support::press_control(&mut app, 'n');
    let second = standing_on(&app);
    assert_ne!(first, second, "ctrl+n did not move");
    support::press_control(&mut app, 'p');
    assert_eq!(standing_on(&app), first);
    assert!(
        app.picker()
            .is_some_and(obelus_component::picker::Picker::is_searching),
        "ctrl+p put the search away"
    );
}

/// Under VS Code's keys `ctrl+p` opens the files and `ctrl+u` the pull
/// requests. In the list they walk it, and outside it they are those again.
///
/// Broken deliberately by answering `None` from `Move::in_picker`:
/// `ctrl+u` put the pull requests in place of the files.
#[test]
fn under_vscode_the_list_has_the_keys_its_layout_gave_away() {
    let (_scratch, mut app) = a_project("walking-vscode");
    app.set_keymap(Keymap::laid_out(Layout::VsCode));
    support::press_control(&mut app, 'p');
    assert!(is_the_file_list(&app), "ctrl+p did not open the files");
    assert_eq!(standing_on(&app), "a10.rs");

    support::press_control(&mut app, 'd');
    assert_eq!(standing_on(&app), "a20.rs");
    support::press_control(&mut app, 'u');
    assert_eq!(standing_on(&app), "a10.rs");
    assert!(is_the_file_list(&app), "ctrl+u left the files");
    support::press_control(&mut app, 'p');
    assert_eq!(standing_on(&app), "a09.rs");

    support::press(&mut app, KeyCode::Esc);
    assert!(app.picker().is_none(), "escape did not leave");
    support::press_control(&mut app, 'p');
    assert!(is_the_file_list(&app), "ctrl+p is not the files again");
}

/// Under TextMate's, the files are `ctrl+t` and the walking is the same.
///
/// Broken deliberately by answering `None` from `Move::in_picker`: the
/// row stayed where the list opened.
#[test]
fn under_textmate_the_list_is_walked_the_same_way() {
    let (_scratch, mut app) = a_project("walking-textmate");
    app.set_keymap(Keymap::laid_out(Layout::TextMate));
    support::press_control(&mut app, 't');
    assert!(is_the_file_list(&app), "ctrl+t did not open the files");
    support::press_control(&mut app, 'n');
    assert_eq!(standing_on(&app), "a11.rs");
    support::press_control(&mut app, 'p');
    assert_eq!(standing_on(&app), "a10.rs");
}

/// The row of the screen a word is drawn on, from the bottom up.
fn row_saying(dump: &str, words: &str) -> Option<usize> {
    let rows: Vec<&str> = support::text_block(dump).lines().collect();
    rows.iter().rposition(|row| row.contains(words))
}

/// The foot says the four, after the list's own keys: where there is room
/// they are there, and where there is not they are what goes.
///
/// Broken deliberately by putting `walking` in front of `every_hint` in
/// `obelus_ui::picker::hints`: at sixty columns the foot said `Rows` and
/// lost `Ignored files`.
#[test]
fn the_foot_says_the_four_after_the_lists_own_keys() {
    let (_scratch, mut app) = a_project("walking-foot");
    support::press_function(&mut app, 1);
    let wide = support::render(&mut app, 140, 34);
    let foot = row_saying(&wide, "Ignored files").expect("a foot");
    assert_eq!(row_saying(&wide, "Ctrl+n Ctrl+p"), Some(foot), "{wide}");
    assert_eq!(row_saying(&wide, "Ctrl+d Ctrl+u"), Some(foot), "{wide}");

    let narrow = support::render(&mut app, 60, 34);
    assert!(row_saying(&narrow, "Ignored files").is_some(), "{narrow}");
    assert_eq!(row_saying(&narrow, "Rows"), None, "{narrow}");
}

/// The foot is as tall on the changed files as on the tree, so the bottom
/// of the screen does not move as the reader walks between them.
///
/// Broken deliberately by giving `walking` the old condition, a search or
/// a list that offers ignored files: on the changed tab the foot went, and
/// the rule over the prompt was two rows lower than on the tree.
#[test]
fn the_foot_stays_where_it_is_on_the_changed_files() {
    let (scratch, mut app) = a_project("walking-changed");
    let mut statuses = std::collections::HashMap::new();
    statuses.insert(
        scratch.join("a03.rs"),
        obelus_git::Standing::from(obelus_git::FileStatus::Changed),
    );
    app.statuses_for_test(statuses);
    support::press_function(&mut app, 1);
    let tree = support::render(&mut app, 100, 34);
    support::press(&mut app, KeyCode::Tab);
    let changed = support::render(&mut app, 100, 34);
    assert!(changed.contains("a03.rs"), "not the changed tab: {changed}");
    assert_eq!(
        row_saying(&changed, "Rows"),
        row_saying(&tree, "Rows"),
        "the foot moved between the tabs:\n{tree}\n{changed}"
    );
    assert!(row_saying(&changed, "Rows").is_some(), "{changed}");
}
