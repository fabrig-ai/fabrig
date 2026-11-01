use jig_protocol_store::{Generations, Older, OpenRefusal};
use jig_store_world::World;
use skein_fake_machine::Item;

fn directory(path: &str) -> Item {
    Item::directory(path.as_bytes()).mode(0o700)
}

fn file(path: &str, bytes: &[u8]) -> Item {
    Item::file(path.as_bytes(), bytes).mode(0o600)
}

fn generation(world: &mut World, number: u64) {
    world.lay(&[
        directory(&format!("generations/{number}")),
        directory(&format!("generations/{number}/store")),
        directory(&format!("generations/{number}/secrets")),
    ]);
}

fn active_generation(number: u64) -> World {
    let mut world = World::unopened(801);
    world.lay(&[directory("generations")]);
    generation(&mut world, number);
    world.lay(&[file("active", format!("{number}\n").as_bytes())]);
    world
}

fn contains(paths: &[Vec<u8>], path: &str) -> bool {
    paths.iter().any(|found| found == path.as_bytes())
}

fn absent_tree(paths: &[Vec<u8>], path: &str) {
    let prefix = format!("{path}/");
    assert!(
        !paths.iter().any(|found| found == path.as_bytes() || found.starts_with(prefix.as_bytes())),
        "stray {path} remains: {paths:?}"
    );
}

fn first_start_defects(world: &mut World) -> Vec<String> {
    let mut defects = Vec::new();
    let active = world.active();
    if active != b"1\n" {
        defects.push(format!("active is {active:?}, wanted 1\\n"));
    }
    let paths = world.paths();
    for path in ["generations/1/store", "generations/1/secrets", "generations/1/manifest", "transcripts"] {
        if !contains(&paths, path) {
            defects.push(format!("missing {path}"));
        }
    }
    if paths.iter().any(|path| path.starts_with(b"active.skein-")) {
        defects.push("durable-replace temporary survives".to_owned());
    }
    defects
}

fn clean_first_start(world: &mut World) {
    let defects = first_start_defects(world);
    assert!(defects.is_empty(), "first-start defects: {defects:?}");
}

#[test]
fn first_start_is_empty_and_durable() {
    // store.md 2: The first start makes generation 1's manifest; the maps are empty until the root's first commit.
    let mut world = World::unopened(802);
    assert_eq!(world.open_switched(), Some(0));
    clean_first_start(&mut world);
    world.crash(803);
    let restored = world.reopen();
    assert_eq!(restored.last, 0);
    assert!(restored.rows.is_empty());
    assert!(restored.application.is_empty());
    assert!(restored.transcripts.is_empty());
    clean_first_start(&mut world);
}

#[test]
fn largest_generation_fits_the_active_limit() {
    // store.md 10.1.2: active loads up to 21 bytes of decimal digits and a newline.
    let mut world = active_generation(u64::MAX);
    assert_eq!(world.open_switched(), Some(0));
    assert_eq!(world.active(), b"18446744073709551615\n");
}

fn stray_fixture() -> World {
    let mut world = active_generation(3);
    world.lay(&[
        directory("generations/2"),
        file("generations/2/kept", b"rollback"),
        directory("generations/4"),
        directory("generations/4/deep"),
        directory("generations/4/deep/nested"),
        file("generations/4/deep/nested/data", b"unfinished"),
        directory("generations/dropped"),
        file("generations/dropped/partial", b"old"),
        file("generations/stray-file", b"stray"),
        file("active.skein-0123456789abcdef", b"4\n"),
    ]);
    world
}

fn strays_gone(world: &mut World) {
    assert_eq!(world.active(), b"3\n");
    let paths = world.paths();
    assert!(contains(&paths, "generations/2/kept"));
    assert!(contains(&paths, "generations/3/store"));
    assert!(contains(&paths, "generations/3/secrets"));
    assert!(contains(&paths, "transcripts"));
    for path in ["generations/4", "generations/dropped", "generations/stray-file", "active.skein-0123456789abcdef"] {
        absent_tree(&paths, path);
    }
}

#[test]
fn recursive_cleanup_restarts_directory_pages() {
    // store.md 10.1.5: List each directory from its first page until empty, then remove it deepest first.
    let mut world = stray_fixture();
    for number in 0..160 {
        world.lay(&[file(&format!("generations/4/deep/file-{number:03}"), b"partial")]);
    }
    assert_eq!(world.open_switched(), Some(0));
    strays_gone(&mut world);
}

#[test]
fn generation_refusal_words_match_operator_instructions() {
    // store.md 10.1: Each Generations refusal's text, whole, is what OpenRefusal::words gives.
    for (why, expected) in [
        (
            Generations::Busy,
            "Another process has <deployment> open, and this start has changed nothing in it. Stop that process first, then start again.",
        ),
        (
            Generations::Unreadable,
            "The `active` file in <deployment> does not name a generation, and this start has changed nothing in it. Restore <deployment> from a backup, or write `active` again to name a generation under `generations/` that stands, and start again.",
        ),
        (
            Generations::Missing { generation: 17 },
            "The generation `active` names in <deployment> has lost its `store/` or `secrets/`, and this start has changed nothing in it. Restore <deployment> from a backup, or write `active` again to name a generation under `generations/` that stands, and start again.",
        ),
        (
            Generations::Unnamed,
            "<deployment> holds generations that no `active` names, and this start has changed nothing in it. Name one again from a backup of <deployment>, or write `active` to name the generation under `generations/` to open, and start again.",
        ),
    ] {
        assert_eq!(OpenRefusal::Generations { why }.words().join("<deployment>"), expected);
    }
}

#[test]
fn an_older_builds_refusal_names_the_whole_layout_to_move() {
    // store.md 2: Older::Directory carries the whole instruction to move store/, secrets/ and transcripts/ together.
    let words = OpenRefusal::OlderBuild { why: Older::Directory }.words().join("<deployment>");
    assert_eq!(
        words,
        "<deployment> holds a store from an older build, in the layout before generations (`store/`, `secrets/` or `transcripts/` at its top), and this start has changed nothing in it. Before the first release a store is not translated, so every start refuses this one until it is moved. To start afresh, stop the service, move `store/`, `secrets/` and `transcripts/` out of <deployment> together, and start again: nothing is deleted, and the next start makes a new, empty store. The new store has no tasks, sign-ins, accounts or keys, and every client signs in again."
    );
}

#[test]
fn interrupted_empty_generations_restart() {
    for (label, files) in [
        ("active temporary", vec![file("active.skein-0123456789abcdef", b"unfinished")]),
        ("partial manifest", vec![file("generations/1/manifest", b"partial manifest")]),
        (
            "manifest temporary",
            vec![
                file("generations/1/manifest.skein-0123456789abcdef", b"partial"),
                file("active.skein-0123456789abcdef", b"1\n"),
            ],
        ),
    ] {
        let mut world = World::unopened(804);
        world.lay(&[directory("generations")]);
        generation(&mut world, 1);
        world.lay(&files);
        assert_eq!(world.open_switched(), Some(0), "{label}");
        clean_first_start(&mut world);
    }
}

type LayoutCase = (String, Vec<Item>, OpenRefusal);

fn malformed_active_cases() -> Vec<LayoutCase> {
    let mut cases = Vec::new();
    for bytes in [
        b"".as_slice(),
        b"1",
        b"00\n",
        b"01\n",
        b"+1\n",
        b"-1\n",
        b" 1\n",
        b"1 \n",
        b"1\r\n",
        b"1\n\n",
        b"1\ntrailing",
        b"1\0\n",
        b"\xff\n",
        b"18446744073709551616\n",
        b"184467440737095516150\n",
        b"18446744073709551615\nX",
    ] {
        cases.push((
            format!("malformed active {bytes:?}"),
            vec![
                directory("generations"),
                directory("generations/1"),
                directory("generations/1/store"),
                directory("generations/1/secrets"),
                file("active", bytes),
                directory("generations/stray"),
                file("active.skein-0123456789abcdef", b"keep"),
            ],
            OpenRefusal::Generations { why: Generations::Unreadable },
        ));
    }
    cases
}

#[test]
fn invalid_layouts_refuse_without_changing_files() {
    let older = OpenRefusal::OlderBuild { why: Older::Directory };
    let unnamed = OpenRefusal::Generations { why: Generations::Unnamed };
    let mut cases = Vec::new();
    for name in ["store", "secrets", "transcripts"] {
        cases.push((
            format!("old {name}"),
            vec![directory(name), file(&format!("{name}/kept"), b"operator data")],
            older.clone(),
        ));
    }
    cases.push((
        "old layout before interrupted start".into(),
        vec![
            directory("generations"),
            directory("generations/1"),
            directory("generations/1/store"),
            directory("generations/1/secrets"),
            directory("transcripts"),
            file("transcripts/kept", b"turns"),
            file("active.skein-0123456789abcdef", b"1\n"),
        ],
        older,
    ));
    for map in ["store", "secrets"] {
        cases.push((
            format!("unnamed {map}"),
            vec![
                directory("generations"),
                directory("generations/1"),
                directory("generations/1/store"),
                directory("generations/1/secrets"),
                file(&format!("generations/1/{map}/kept"), b"durable data"),
            ],
            unnamed.clone(),
        ));
    }
    for items in [
        vec![directory("generations/2")],
        vec![file("generations/unknown", b"keep")],
        vec![directory("generations/1"), directory("generations/2")],
    ] {
        let label = format!("unnamed entries {items:?}");
        let mut all = vec![directory("generations")];
        all.extend(items);
        cases.push((label, all, unnamed.clone()));
    }
    cases.extend(malformed_active_cases());
    for items in [
        vec![],
        vec![directory("generations/7/store")],
        vec![directory("generations/7/secrets")],
        vec![file("generations/7/store", b"file"), directory("generations/7/secrets")],
        vec![directory("generations/7/store"), file("generations/7/secrets", b"file")],
    ] {
        let label = format!("missing maps {items:?}");
        let mut all = vec![
            directory("generations"),
            directory("generations/7"),
            directory("generations/8"),
            file("generations/8/kept", b"stray"),
            file("active", b"7\n"),
            file("active.skein-0123456789abcdef", b"keep"),
        ];
        all.extend(items);
        cases.push((label, all, OpenRefusal::Generations { why: Generations::Missing { generation: 7 } }));
    }
    cases.push((
        "largest active names absent generation".into(),
        vec![directory("generations"), file("active", b"18446744073709551615\n")],
        OpenRefusal::Generations { why: Generations::Missing { generation: u64::MAX } },
    ));
    for (label, items, expected) in cases {
        let mut world = World::unopened(806);
        world.lay(&items);
        let before = world.contents();
        assert_eq!(world.open_switched(), None, "{label}");
        assert_eq!(world.refusal(), Some(&expected), "{label}");
        assert_eq!(world.contents(), before, "{label}: refusal changes no file");
    }
}

#[test]
fn cleanup_keeps_previous_shared_and_unrelated_files() {
    for (label, active, items, kept, removed) in [
        (
            "opaque previous",
            10,
            vec![
                directory("generations/9"),
                file("generations/9/store", b"not a map"),
                file("generations/9/secrets", b"not a map either"),
                file("generations/9/manifest", b"opaque"),
                directory("generations/8"),
                file("generations/8/drop", b"old"),
                directory("generations/11"),
                file("generations/11/drop", b"unfinished"),
            ],
            vec![
                "generations/9/store",
                "generations/9/secrets",
                "generations/9/manifest",
                "generations/10/store",
                "generations/10/secrets",
            ],
            vec!["generations/8", "generations/11"],
        ),
        (
            "noncanonical strays",
            7,
            vec![
                directory("generations/6"),
                file("generations/6/kept", b"previous"),
                directory("generations/06"),
                directory("generations/007"),
                directory("generations/7-old"),
            ],
            vec!["generations/6/kept"],
            vec!["generations/06", "generations/007", "generations/7-old"],
        ),
        (
            "replace temporaries",
            1,
            vec![
                file("active.skein-0123456789abcde", b"unrelated"),
                file("active.skein-0123456789abcdef0", b"unrelated"),
                file("active.skein-zzzzzzzzzzzzzzzz", b"unrelated"),
                file("active.notes", b"unrelated"),
                file("active.skein-0123456789abcdef", b"unfinished"),
                file("active.skein-fedcba9876543210", b"unfinished"),
            ],
            vec![
                "active.skein-0123456789abcde",
                "active.skein-0123456789abcdef0",
                "active.skein-zzzzzzzzzzzzzzzz",
                "active.notes",
            ],
            vec!["active.skein-0123456789abcdef", "active.skein-fedcba9876543210"],
        ),
        (
            "shared transcripts",
            2,
            vec![directory("transcripts"), directory("transcripts/42"), file("transcripts/42/kept", b"turns")],
            vec!["transcripts/42/kept"],
            vec![],
        ),
    ] {
        let mut world = active_generation(active);
        world.lay(&items);
        let before = world.contents();
        assert_eq!(world.open_switched(), Some(0), "{label}");
        assert_eq!(world.active(), format!("{active}\n").as_bytes(), "{label}");
        let after = world.contents();
        for path in kept {
            assert!(after.contains_key(path.as_bytes()), "{label}: missing {path}");
            assert_eq!(after.get(path.as_bytes()), before.get(path.as_bytes()), "{label}: changed {path}");
        }
        let paths = world.paths();
        for path in removed {
            absent_tree(&paths, path);
        }
    }
}
