use super::{HELP, commands_page, commands_reference, help, refused};
use skein_shell::{Argument, Command, Reach, Set, Table};

const PUBLIC: Command = Command {
    reach: Reach::Words(&["show"]),
    set: Set::Public,
    forms: &["NAME", "--all"],
    line: "Show one item or all items.",
    arguments: &[Argument { shown: "NAME", line: "The item to show." }],
    notes: "Reads no store.",
    ..HELP
};
const INTERNAL: Command = Command {
    reach: Reach::Words(&["internal", "child"]),
    set: Set::Internal,
    forms: &[""],
    line: "Run a child.",
    arguments: &[],
    notes: "",
    ..HELP
};
const DEMO: Table = Table {
    binary: "temper-demo",
    line: "A demonstration.",
    version: "1",
    commit: "unknown",
    // Rendering puts public entries first without changing order within a set.
    commands: &[
        INTERNAL,
        PUBLIC,
        HELP,
        Command { reach: Reach::Words(&[]), forms: &[""], ..PUBLIC },
        Command { reach: Reach::RunAs("git"), ..INTERNAL },
    ],
    settings: &[],
    variables: &[],
    more: "temper-demo help pages lists the pages.",
};

fn words(words: &[&str]) -> Vec<Box<[u8]>> {
    words.iter().map(|word| word.as_bytes().into()).collect()
}

#[test]
fn help_answers_pages_commands_and_searches_with_their_exits() {
    assert_eq!(help(&DEMO, &[]), Ok(DEMO.help()));
    assert_eq!(help(&DEMO, &words(&["pages"])), Ok(temper_docs::find(b"index").expect("index").text.into()));
    assert_eq!(crate::help::request(&words(&["pages"])), Some(crate::help::Request::Index));
    assert_eq!(help(&DEMO, &words(&["commands"])), Ok(commands_page(&DEMO)));
    assert_eq!(help(&DEMO, &words(&["index"])), Ok(temper_docs::find(b"index").expect("index").text.into()));
    assert_eq!(
        help(&DEMO, &words(&["nope"])),
        Err((1, "temper-demo: no page nope; temper-demo help lists them\n".into()))
    );
    let search = help(&DEMO, &words(&["--search", "temper"])).expect("matching search");
    assert!(!search.is_empty());
    assert!(search.lines().all(|line| line.split(": ").count() >= 3 || line == "and more; narrow the words"));
    assert_eq!(
        help(&DEMO, &words(&["--search", "zzzznotadocword"])),
        Err((1, "temper-demo: nothing matches\n".into()))
    );
    for (arguments, why) in [
        (&["--search", " \t"][..], "nothing to search for"),
        (&["--search", "a a a a a a a a a"], "at most 8 words"),
        (&["--search", "123456789012345678901234567890123"], "a word is at most 32 bytes"),
    ] {
        assert_eq!(help(&DEMO, &words(arguments)), Err((2, refused(&DEMO, &HELP, why))));
    }
    for arguments in [&["pages", "extra"][..], &["--search"], &["--unknown"]] {
        assert_eq!(help(&DEMO, &words(arguments)), Err((2, refused(&DEMO, &HELP, "invalid arguments"))));
    }
}

#[test]
fn refused_names_the_command_then_each_form() {
    assert_eq!(
        refused(&DEMO, &PUBLIC, "missing NAME"),
        "temper-demo show: missing NAME\nusage: temper-demo show NAME\nusage: temper-demo show --all\n"
    );
    assert_eq!(
        refused(&DEMO, &INTERNAL, "extra argument"),
        "temper-demo internal child: extra argument\nusage: temper-demo internal child\n"
    );
}

#[test]
fn commands_page_keeps_the_reference_shape_and_sets_order() {
    let page = commands_page(&DEMO);
    assert!(page.starts_with("# temper's commands\n\nKind: reference\n\nEvery command of the temper-demo binary, by who runs it: its usage, what it does, its arguments and its notes.\n"));
    let public = "\n## temper-demo show\n\n`temper-demo show NAME`\n\n`temper-demo show --all`\n\nShow one item or all items.\n\n- `NAME`: The item to show.\n\nReads no store.\n";
    let internal = "\n## temper-demo internal child\n\n`temper-demo internal child`\n\nRun a child. Internal: it may change without notice.\n";
    assert!(page.contains(public), "{page}");
    assert!(page.contains(internal), "{page}");
    assert!(page.find(public) < page.find(internal));
    assert!(page.contains("\n## temper-demo\n\n`temper-demo`\n"));
    assert!(page.contains("\n## run as git\n\n`git`\n"));
}

#[test]
fn commands_reference_names_each_binary_and_shares_equal_rows() {
    const RUN_AS: Command = Command { reach: Reach::RunAs("git"), ..INTERNAL };
    const FIRST: Table =
        Table { binary: "first", line: "The first binary.", commands: &[INTERNAL, PUBLIC, RUN_AS], ..DEMO };
    const SECOND: Table = Table {
        binary: "second",
        line: "The second binary.",
        commands: &[RUN_AS, Command { line: "Run a different child.", ..INTERNAL }, PUBLIC],
        ..DEMO
    };
    assert_eq!(
        commands_reference(&[&FIRST, &SECOND]),
        "# temper's commands

Kind: reference

Every command of temper's binaries, a section each: its usage, what it does, its arguments and its notes. A command two binaries share is written under the first and named under the others.

## first

The first binary.

### first show

`first show NAME`

`first show --all`

Show one item or all items.

- `NAME`: The item to show.

Reads no store.

### first internal child

`first internal child`

Run a child. Internal: it may change without notice.

### run as git

`git`

Run a child. Internal: it may change without notice.

## second

The second binary.

### second show

`second show NAME`

`second show --all`

As first show, above.

### run as git

`git`

As first's run as git, above.

### second internal child

`second internal child`

Run a different child. Internal: it may change without notice.
"
    );
}
