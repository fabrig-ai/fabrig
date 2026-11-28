use super::*;
use crate::{pages, parse, parse::parse_at, render, render::render_at, tests::facts};

pub(crate) const TWO: Formats = Formats {
    newest: 2,
    moves: &[
        Move { format: 2, key: "git_binary", kind: MoveKind::Renamed { new: "programs.git" } },
        Move {
            format: 2,
            key: "listen_on",
            kind: MoveKind::Converted { new: "listen", convert: socket_from_array, words: "a socket string in listen" },
        },
        Move {
            format: 2,
            key: "seed_count",
            kind: MoveKind::MovedOut { home: "fixture settings", words: "now in the fixture settings" },
        },
        Move { format: 2, key: "retired_flag", kind: MoveKind::Gone { release: "test release" } },
    ],
};

fn socket_from_array(value: Value) -> Result<Value, &'static str> {
    if let Value::Array(mut sockets) = value
        && sockets.len() == 1
        && sockets[0].is_str()
    {
        return Ok(sockets.remove(0));
    }
    Err("an array with one socket string")
}

#[test]
fn format_one_fixtures_round_trip_unchanged() {
    let fixtures = [
        (Shape::Standalone, include_bytes!("../../../../docs/examples/formats/temper-toml/1/box.toml").as_slice()),
        (Shape::EngineAlone, include_bytes!("../../../../docs/examples/formats/temper-toml/1/engine.toml").as_slice()),
        (Shape::WorkerAlone, include_bytes!("../../../../docs/examples/formats/temper-toml/1/worker.toml").as_slice()),
    ];
    let facts = facts();
    for (shape, text) in fixtures {
        let frozen = parse(text, shape, &facts).expect("frozen format 1");
        assert_eq!(frozen.format, 1);
        assert_eq!(frozen.notices, Vec::<String>::new());
        let written = [b"format = 1\n", text].concat();
        let explicit = parse(&written, shape, &facts).expect("explicit format 1");
        assert_eq!(explicit.format, 1);
        assert_eq!(frozen.file, explicit.file);
        assert_eq!(frozen.document, explicit.document);
        let rendered = render(&explicit.file, shape);
        let canonical = parse(&rendered, shape, &facts).expect("rendered format 1");
        assert!(canonical.keys.line(KEY).is_none(), "format 1 is not printed");
        assert_eq!(canonical.file, frozen.file);
        assert_eq!(canonical.document, frozen.document);
    }
}

#[test]
fn newer_formats_refuse_before_other_keys() {
    let facts = facts();
    let text = b"format = 2\nlisten = '127.0.0.1:7100'\nbogus = 1\n";
    for (shape, name) in [
        (Shape::Standalone, "temper-box"),
        (Shape::EngineAlone, "temper-engine"),
        (Shape::WorkerAlone, "temper-worker"),
    ] {
        assert_eq!(
            parse(text, shape, &facts).expect_err("newer format").problems,
            [format!("temper.toml: format 2; this {name} reads format 1: upgrade it, or write the file at format 1")]
        );
    }
    assert_eq!(
        parse_at(b"format = 3\nbogus = 1\n", Shape::Standalone, &facts, TWO)
            .expect_err("above the test table")
            .problems,
        ["temper.toml: format 3; this temper-box reads formats 1 to 2: upgrade it, or write the file at format 2"]
    );
}

#[test]
fn bad_formats_refuse_alone_at_the_key_line() {
    let facts = facts();
    for (text, line) in [
        ("format = 0\nbogus = 1\n", 1),
        ("format = \"1\"\nbogus = 1\n", 1),
        ("format = 1.5\nbogus = 1\n", 1),
        ("bogus = 1\nformat = -1\n", 2),
        ("format = 0x1\nbogus = 1\n", 1),
        ("[format]\nunknown = 1\n", 1),
    ] {
        assert_eq!(
            parse(text.as_bytes(), Shape::Standalone, &facts).expect_err("bad format").problems,
            [format!("temper.toml:{line}: format: a whole number from 1")],
            "{text}"
        );
    }
    assert_eq!(
        parse(b"[programs]\nformat = 1\n", Shape::Standalone, &facts).expect_err("not a top-level format").problems,
        ["temper.toml:2: programs.format: unknown; temper-box help config lists the keys"]
    );
}

#[test]
fn moves_keep_file_order_and_the_values_sources() {
    let facts = facts();
    let shape = Shape::Standalone;
    let old = b"# the original file\nretired_flag = true\nseed_count = 11\nlisten_on = ['127.0.0.1:7100']\ngit_binary = '/tools/git'\n";
    let read = parse_at(old, shape, &facts, TWO).expect("moved format 1");
    let newest = b"format = 2\nlisten = '127.0.0.1:7100'\n[programs]\ngit = '/tools/git'\n";
    let expected = parse_at(newest, shape, &facts, TWO).expect("format 2 spelling");
    assert_eq!(read.format, 1);
    assert_eq!(read.file, expected.file);
    assert_eq!(read.document, expected.document);
    assert_eq!(
        read.notices,
        [
            "temper.toml:2: retired_flag: ignored; test release took it away",
            "temper.toml:3: seed_count: moved: now in the fixture settings",
            "temper.toml:4: listen_on: format 1; format 2 writes it as a socket string in listen",
            "temper.toml:5: git_binary: format 1; format 2 writes it as programs.git",
        ]
    );
    assert_eq!(read.keys.line("programs.git"), Some(5));
    assert_eq!(read.keys.line("listen"), Some(4));
    let [kept] = read.moved_out.as_slice() else { panic!("one value for seeding") };
    assert_eq!((kept.key, kept.line, &kept.value), ("seed_count", 3, &Value::Integer(11)));
    assert!(matches!(kept.movement.kind, MoveKind::MovedOut { home: "fixture settings", .. }));
    assert_eq!(kept.movement.format, 2);
}

#[test]
fn retired_and_unknown_keys_stay_unknown() {
    let facts = facts();
    let shape = Shape::Standalone;
    for movement in TWO.moves {
        let key = movement.key;
        let text = format!("format = 2\n{key} = 1\n");
        let why = parse_at(text.as_bytes(), shape, &facts, TWO).expect_err("retired name in format 2");
        assert_eq!(why.problems, [format!("temper.toml:2: {key}: unknown; temper-box help config lists the keys")]);
    }
    assert_eq!(
        parse_at(b"bogus = 1\n", shape, &facts, TWO).expect_err("unknown in format 1").problems,
        ["temper.toml:1: bogus: unknown; temper-box help config lists the keys"]
    );
    for text in [b"slots = 1\n".as_slice(), b"slots = 1\nformat = 2\n"] {
        assert_eq!(
            parse_at(text, shape, &facts, TWO).expect_err("a former JSON name, not an older TOML key").problems,
            ["temper.toml:1: slots: moved: a worker's places are its setting's; temper ctl worker places NAME N"]
        );
    }
}

#[test]
fn rendering_a_later_format_writes_it_as_the_first_key() {
    let facts = facts();
    let shape = Shape::Standalone;
    let old = parse_at(b"listen_on = ['127.0.0.1:7100']\n", shape, &facts, TWO).expect("format 1");
    let rendered = render_at(&old.file, shape, TWO);
    let text = std::str::from_utf8(&rendered).expect("rendered UTF-8");
    assert!(
        text.lines()
            .find(|line| !line.is_empty() && !line.starts_with('#'))
            .expect("first key")
            .starts_with("format = 2")
    );
    let newest = parse_at(&rendered, shape, &facts, TWO).expect("rendered format 2");
    assert_eq!(newest.format, 2);
    assert_eq!(newest.file, old.file);
    assert_eq!(newest.document, old.document);
    assert_eq!(newest.notices, Vec::<String>::new());
}

#[test]
fn format_pages_list_each_move() {
    let help = pages::help_at(Shape::Standalone, TWO);
    let reference = pages::reference_at(TWO);
    for (page, end) in [(&help, None), (&reference, Some("\n## The merge command"))] {
        let text = std::str::from_utf8(page).expect("page");
        let section = text.split_once("\n## Formats\n").expect("formats section").1;
        let section = end.map_or(section, |end| section.split_once(end).expect("following section").0);
        assert!(section.contains("Format 2 is the newest temper-box reads; a file without format is format 1."));
        assert_eq!(
            section.lines().filter(|line| line.starts_with("- format ")).collect::<Vec<_>>(),
            [
                "- format 2: git_binary: renamed programs.git",
                "- format 2: listen_on: converted to listen: a socket string in listen",
                "- format 2: seed_count: moved out: now in the fixture settings",
                "- format 2: retired_flag: gone in test release",
            ]
        );
    }
}
