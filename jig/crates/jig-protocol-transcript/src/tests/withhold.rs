use super::*;

const TITLE: &[u8] = b"Kept about this person";
const SECTION: &[u8] = b"## Kept about this person\n\n1. First\n2. Second\n3. Third\n\n## Shared\nKept words.\n";
const PROJECTED: &[u8] =
    b"## Kept about this person\n\nWithheld: shown to its person alone.\n\n## Shared\nKept words.\n";

// Thirteen section/configuration rows, seven returns, one archive, four
// admission errors, and four output-bound rows.
const ROWS: u32 = 29;

#[derive(Debug)]
struct Row {
    name: &'static str,
    limits: Limits,
    bytes: Box<[u8]>,
    private: Private,
    expected: Result<smith::Turn, Undecodable>,
}

fn private() -> Private {
    let mut tools = List::with_capacity(1);
    tools.push(Box::from(b"read".as_slice())).unwrap();
    Private { title: Box::from(TITLE), tools }
}

fn row(name: &'static str, before: smith::Turn, private: Private, after: smith::Turn) -> Row {
    Row { name, limits: limits(), bytes: encode(&before), private, expected: Ok(after) }
}

fn system(value: &[u8]) -> smith::Turn {
    let mut parts = source(&[]).into_parts();
    parts.system = Some(Box::from(value));
    smith::Turn::new(&CEILINGS, parts).unwrap()
}

fn texts(role: smith::Role, values: &[SBlock]) -> smith::Turn {
    source(&[message(role, values)])
}

fn replay() -> smith::Replay {
    smith::Replay::new(
        &CEILINGS,
        smith::ReplayParts { dialect: DIALECT, bytes: Box::from(b"provider bytes".as_slice()) },
    )
    .unwrap()
}

fn replayed(value: &[u8]) -> SBlock {
    let mut parts = text(value).into_parts();
    parts.replay = Some(replay());
    SBlock::Text(smith::Text::new(&CEILINGS, parts).unwrap())
}

fn named_call(id: &[u8], name: &[u8], input: &[u8], replay: Option<smith::Replay>) -> SBlock {
    let SBlock::Call(call) = call(id) else { panic!("call builder returns a call") };
    let mut parts = call.into_parts();
    parts.name = Box::from(name);
    parts.input = Box::from(input);
    parts.replay = replay;
    SBlock::Call(smith::Call::new(&CEILINGS, parts).unwrap())
}

fn answer(value: &[u8], error: bool) -> smith::Returned {
    smith::Returned::Said(smith::Said::new(&CEILINGS, smith::SaidParts { text: Box::from(value), error }).unwrap())
}

fn calls(returned: smith::Returned, projected: bool) -> smith::Turn {
    let input = if projected { b"{}".as_slice() } else { b"{\"private\":true}" };
    let replay = if projected { None } else { Some(replay()) };
    source(&[
        message(smith::Role::User, &[result(b"private", returned), result(b"public", said(b"public result"))]),
        message(
            smith::Role::Assistant,
            &[
                named_call(b"private", b"read", input, replay),
                named_call(b"public", b"other", b"{\"kept\":true}", Some(self::replay())),
            ],
        ),
    ])
}

// Version 6 shares the turn body, without version 7's final system option.
fn archived(turn: &smith::Turn) -> Box<[u8]> {
    let bytes = encode(turn);
    let end = bytes.len().checked_sub(1).unwrap();
    let mut writer = skein_lib::Writer::new(end);
    writer.put(&6_u16.to_be_bytes()).unwrap();
    writer.put(bytes.get(2..end).unwrap()).unwrap();
    writer.finish()
}

#[test]
#[expect(clippy::too_many_lines, reason = "one table compares the projection and admission rules")]
fn private_parts_are_withheld_with_the_same_admission() {
    let opening = b"## Kept about this person\nNone kept.\nPlease explain this.\n";
    let opening_after = b"## Kept about this person\n\nWithheld: shown to its person alone.\n\nPlease explain this.\n";
    let immediate = b"## Kept about this person\nPlease keep these words.";
    let immediate_after =
        b"## Kept about this person\n\nWithheld: shown to its person alone.\n\nPlease keep these words.";
    let unchanged =
        texts(smith::Role::Assistant, &[replayed(SECTION), call(b"private"), result(b"private", said(b"answer"))]);
    let mut rows = List::with_capacity(ROWS);
    for row in [
        row("system numbered section", system(SECTION), private(), system(PROJECTED)),
        row("opening user section", texts(smith::Role::User, &[SBlock::Text(text(opening))]), private(), texts(smith::Role::User, &[SBlock::Text(text(opening_after))])),
        row("immediate prose", texts(smith::Role::Assistant, &[SBlock::Text(text(immediate))]), private(), texts(smith::Role::Assistant, &[SBlock::Text(text(immediate_after))])),
        row("changed replay dropped, unchanged kept", texts(smith::Role::Assistant, &[replayed(SECTION), replayed(b"Kept text")]), private(), texts(smith::Role::Assistant, &[SBlock::Text(text(PROJECTED)), replayed(b"Kept text")])),
        row("empty title still withholds calls", calls(said(b"private words"), false), Private { title: Box::new([]), ..private() }, calls(answer(WITHHELD.as_bytes(), false), true)),
        row("empty title keeps sections", system(SECTION), Private { title: Box::new([]), ..private() }, system(SECTION)),
        row("none keeps all blocks", unchanged.clone(), Private::none(), unchanged),
        row("refusal text is unchanged", texts(smith::Role::Assistant, &[SBlock::Refusal(text(SECTION))]), private(), texts(smith::Role::Assistant, &[SBlock::Refusal(text(SECTION))])),
        row("heading at end", system(b"## Kept about this person"), private(), system(b"## Kept about this person\n\nWithheld: shown to its person alone.\n\n")),
        row("section ends at text end", system(b"## Kept about this person\n0001. Secret"), private(), system(b"## Kept about this person\n\nWithheld: shown to its person alone.\n\n")),
        row("exact heading only", system(b"## Kept about this person extra\n1. Public\n"), private(), system(b"## Kept about this person extra\n1. Public\n")),
        row("number syntax ends section", system(b"## Kept about this person\n. Kept\n1. Also kept\n"), private(), system(b"## Kept about this person\n\nWithheld: shown to its person alone.\n\n. Kept\n1. Also kept\n")),
        row("repeated sections", system(b"## Kept about this person\n1. First\n## Kept about this person\nNone kept.\n"), private(), system(b"## Kept about this person\n\nWithheld: shown to its person alone.\n\n## Kept about this person\n\nWithheld: shown to its person alone.\n\n")),
    ] {
        rows.push(row).unwrap();
    }
    for (name, returned, error) in [
        ("said success", said(b"private result"), false),
        ("said error", answer(b"private error", true), true),
        ("outcome success", owned(smith::Outcome::Removed), false),
        ("outcome refused", owned(smith::Outcome::NotGranted), true),
        (
            "invalid",
            smith::Returned::Invalid(
                smith::Invalid::new(&CEILINGS, smith::InvalidParts { problem: smith::CallProblem::UnknownTool })
                    .unwrap(),
            ),
            true,
        ),
        (
            "not run",
            smith::Returned::NotRun(
                smith::NotRun::new(&CEILINGS, smith::NotRunParts { why: smith::NotRunWhy::Stopped }).unwrap(),
            ),
            true,
        ),
        ("withdrawn", smith::Returned::Withdrawn, true),
    ] {
        rows.push(row(name, calls(returned, false), private(), calls(answer(WITHHELD.as_bytes(), error), true)))
            .unwrap();
    }
    let turn = texts(smith::Role::User, &[SBlock::Text(text(opening))]);
    rows.push(Row {
        name: "version 6 upgrades",
        limits: limits(),
        bytes: archived(&turn),
        private: private(),
        expected: Ok(texts(smith::Role::User, &[SBlock::Text(text(opening_after))])),
    })
    .unwrap();
    for (name, bytes, error) in [
        ("malformed", Box::from(b"\0".as_slice()), Undecodable::Malformed),
        ("version", Box::from([0, 4].as_slice()), Undecodable::Version { found: 4 }),
        ("smith bound", Box::from([0, 6, 0, 0, 0, 0, 0, 65].as_slice()), Undecodable::TooLarge),
    ] {
        rows.push(Row { name, limits: limits(), bytes, private: private(), expected: Err(error) }).unwrap();
    }
    let mut byte_bound = limits();
    byte_bound.bytes = source(&[]).measure();
    rows.push(Row {
        name: "input byte bound",
        limits: byte_bound,
        bytes: encode(&system(SECTION)),
        private: private(),
        expected: Err(Undecodable::TooLarge),
    })
    .unwrap();
    let short = system(b"## Kept about this person\n");
    let expanded = system(b"## Kept about this person\n\nWithheld: shown to its person alone.\n\n");
    let mut output_bound = limits();
    output_bound.bytes = expanded.measure();
    rows.push(Row {
        name: "output at byte bound",
        limits: output_bound,
        bytes: encode(&short),
        private: private(),
        expected: Ok(expanded),
    })
    .unwrap();
    output_bound.bytes -= 1;
    rows.push(Row {
        name: "output past byte bound",
        limits: output_bound,
        bytes: encode(&short),
        private: private(),
        expected: Err(Undecodable::TooLarge),
    })
    .unwrap();
    let mut field_bound = limits();
    field_bound.smith.turn_system_some =
        u32::try_from(b"## Kept about this person\n\nWithheld: shown to its person alone.\n\n".len()).unwrap();
    rows.push(Row {
        name: "output at field bound",
        limits: field_bound,
        bytes: encode(&short),
        private: private(),
        expected: Ok(system(b"## Kept about this person\n\nWithheld: shown to its person alone.\n\n")),
    })
    .unwrap();
    field_bound.smith.turn_system_some -= 1;
    rows.push(Row {
        name: "output past field bound",
        limits: field_bound,
        bytes: encode(&short),
        private: private(),
        expected: Err(Undecodable::TooLarge),
    })
    .unwrap();

    for Row { name, limits, bytes, private, expected } in &rows {
        let admission = decode(limits, bytes);
        let projected = withhold(limits, bytes, private);
        if let Err(ref error) = admission {
            assert_eq!(projected, Err(*error), "{name}: admission");
        }
        let decoded = match projected {
            Ok(bytes) => {
                assert_eq!(Reader::new(&bytes).u16(), Some(7), "{name}: current version");
                Ok(smith_transcript::decode_archived(&limits.smith, &bytes).unwrap())
            }
            Err(error) => Err(error),
        };
        assert_eq!(&decoded, expected, "{name}");
    }
}
