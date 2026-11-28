use super::*;
use crate::jig::{
    argv::{self, Usage},
    call::{Parsed, Source},
    json,
    render::document,
    table::Tool,
};
use jig_client::Asker;
use skein_lib::{Duration, List, Wall};

const LIMITS: Limits = Limits {
    rows: 32,
    settings: jig_client_protocol_wire::CEILINGS,
    program_login: b"test",
    application: b"test",
    ask_bytes: 4_096,
    result_bytes: 4_096,
    text: 1_024,
    words: 1_024,
    part: 1_024,
    wait: Duration::from_nanos(0),
    wait_most: Duration::from_nanos(0),
};

#[derive(Debug)]
struct Parts;

impl render::Parts for Parts {
    fn diagnostic_prefix(&self) -> &'static [u8] {
        b"ctl"
    }
    fn command_prefix(&self) -> &'static [u8] {
        b"ctl"
    }
    fn project_charter_directory(&self) -> &'static [u8] {
        b".project/charters/"
    }
    fn failures_recovery(&self) -> &'static [u8] {
        b"read again"
    }
    fn phrasebook(&self, text: u32) -> skein_ui::phrases::Phrasebook {
        document::phrasebook(text)
    }
    fn prepare_setting(&self, _call: &mut Call, _me: Option<&c::Me>) -> bool {
        true
    }
    fn setting_selector(&self, _call: &Call) -> Option<c::Setting> {
        None
    }
    fn setting_name_kind(&self, _call: &Call) -> Option<names::Kind> {
        None
    }
    fn application_read(&self, _query: super::super::application::Query) -> Readable {
        unreachable!("no application setting")
    }
    fn application_answer(
        &self,
        _query: super::super::application::Query,
        _page: &Readout,
    ) -> Option<super::super::application::Answer> {
        None
    }
    fn setting_name(&self, _setting: c::Setting, _out: &mut render::Text) -> bool {
        false
    }
    fn setting_value(&self, _kind: u32, _bytes: &[u8], _out: &mut skein_json::writer::Encoder) -> bool {
        false
    }
    fn task_next(&self, _task: &c::Task, _out: &mut render::Text) {}
    fn owner_part(&self, _part: &c::Part, _out: &mut render::Text) {}
    fn part(&self, _part: &c::ApplicationPart, _out: &mut render::Text) {}
    fn change_kind(&self, _connector: u16, _kind: u16, _out: &mut render::Text) {}
    fn change(&self, _kind: u16, _part: &c::Part, _words: &c::DocumentText, _out: &mut render::Text) {}
}

fn tools() -> [&'static Tool; 45] {
    let mut tools = [&rows::HELP; 45];
    *tools.get_mut(usize::from(rows::SETTINGS_AT)).expect("inventory row") = &rows::SETTINGS;
    *tools.get_mut(usize::from(rows::SETTING_AT)).expect("setting row") = &rows::SETTING;
    tools
}

fn reading(words: &[&[u8]]) -> Reading {
    let Parsed::Call { call, json } = argv::parse(&tools(), words, &LIMITS).expect("setting call") else {
        unreachable!("argv is not a call file")
    };
    Reading::new(Called { number: 1, call_number: 1, call, json, value: None }, &Parts)
}

fn pop_read(out: &mut Queue<Out>) -> (u32, c::Read) {
    let Some(Out::Client { event: Event::Read { handle, read: Readable::Jig(read) } }) = out.pop() else {
        unreachable!("expected a jig read")
    };
    assert!(out.is_empty());
    (u32::try_from(handle.number & u64::from(u32::MAX)).expect("local number"), read)
}

#[test]
fn charter_preview_reads_the_project_at_newest_by_number_or_name() {
    let runner = Runner::new(Asker(1), b"preview", 0);
    let mut out = Queue::with_capacity(2);
    let setting = c::Setting::Charter(c::SettingNumber { number: 5 });
    let expected = (1, c::Read::Setting(c::ReadSetting { setting, project: Some(3), revision: 0, proposal: None }));
    for selector in [b"5".as_slice(), b"implementer"] {
        let mut reading = reading(&[b"settings", b"show", b"charter", selector, b"--project", b"3"]);
        setting_start(&runner, &mut reading, &mut out);
        let request = pop_read(&mut out);
        if selector == b"5" {
            assert_eq!(request, expected);
            continue;
        }
        assert_eq!(request, (6, c::Read::Settings));
        let inventory = Readout::Jig(c::ReadResult::Value(c::ReadValue::Settings(Box::new(c::SettingsDocument {
            newest: 9,
            settings: list([c::SettingLine {
                name: Some(b"implementer".as_slice().into()),
                setting,
                shown: None,
                follows: c::SettingFollows::Default,
                last: None,
                application: None,
            }]),
            held: list([]),
            pending: list([]),
        }))));
        assert!(setting_page(&runner, &mut reading, 6, &inventory, &mut out).expect("resolve the name"));
        assert_eq!(pop_read(&mut out), expected);
    }
}

#[test]
fn charter_preview_refuses_unknown_reasons_and_revisions() {
    let tools = tools();
    let words: &[&[u8]] = &[b"settings", b"show", b"charter", b"3", b"--project", b"1", b"--reason", b"never"];
    let expected = Usage::Kind { tool: rows::SETTING_AT, form: 3, argument: rows::CHARTER_PREVIEW_REASON };
    assert_eq!(argv::parse(&tools, words, &LIMITS).expect_err("unknown reason"), expected);
    let bad_reason =
        br#"{"tool":"settings show","arguments":{"kind":"charter","number":3,"project":1,"reason":"never"}}"#;
    assert_eq!(json::call(&tools, bad_reason, Source::Stdin, &LIMITS).expect_err("unknown JSON reason"), expected);
    let together: &[&[u8]] = &[b"settings", b"show", b"charter", b"3", b"--project", b"1", b"--revision", b"2"];
    let _why = argv::parse(&tools, together, &LIMITS).expect_err("revision beside project");
    let _why = json::call(
        &tools,
        br#"{"tool":"settings show","arguments":{"kind":"charter","number":3,"revision":2,"project":1}}"#,
        Source::Stdin,
        &LIMITS,
    )
    .expect_err("JSON revision beside project");
}

fn list<T, const N: usize>(items: [T; N]) -> List<T> {
    let mut out = List::with_capacity(u32::try_from(N).expect("fixture count"));
    for item in items {
        assert!(out.push(item).is_ok());
    }
    out
}

fn words(text: &str) -> c::Text {
    c::Text::new(text.as_bytes().into(), 1_024).expect("fixture words")
}

fn part(on: u32, instructions: &str) -> c::CharterPart {
    c::CharterPart { on, instructions: words(instructions) }
}

fn charter(bound: bool, with_overlay: bool, number: u32) -> c::ReadValue {
    let zero = Duration::from_nanos(0);
    let (instructions, parts) = if with_overlay {
        (words("Stock.\n\nUse the box.\n"), list([part(1, "Begin."), part(2, "Ask."), part(1, "Boot.\n")]))
    } else {
        (words("Stock."), list([part(1, "Begin.")]))
    };
    let charter = c::Charter {
        name: Some(b"builder".as_slice().into()),
        number,
        instructions,
        parts,
        rest: false,
        documents: list([]),
        waiting: zero,
        resume: false,
        turns: 0,
        time: zero,
        reserve_turns: 0,
        reserve_time: zero,
        alternatives: list([]),
        model: c::SettingModel {
            provider: 1,
            dialect: 1,
            endpoint: 1,
            program: None,
            name: words("small"),
            window: 1_024,
            output: 32,
            input_price: 0,
            cached_price: 0,
            cache_write_price: None,
            output_price: 0,
            price_unit: 1,
            efforts: list([]),
            effort: None,
            speeds: list([]),
            speed: None,
        },
        inspect: false,
        modify: false,
        shell: false,
        agents: false,
        call_timeout: zero,
        tools: 0,
        hosts: c::SettingHosts::Workers,
        spend: 0,
        wake: c::WakePolicy {
            words: c::WakeRule::Never,
            notices: c::WakeRule::Never,
            news: c::WakeRule::Never,
            results: c::ResultsWake::Never,
            questions: false,
            answers: false,
            timers: false,
        },
    };
    let overlay = with_overlay.then_some(Box::new(c::Overlay {
        rest: None,
        model: None,
        alternatives: None,
        inspect: None,
        modify: None,
        shell: None,
        agents: None,
        tools: None,
        hosts: None,
        waiting: None,
        resume: None,
        turns: None,
        time: None,
        reserve_turns: None,
        reserve_time: None,
        call_timeout: None,
        spend: None,
        documents: None,
        instructions: words("Use the box.\n"),
        parts: list([part(2, "Ask."), part(1, "Boot.\n")]),
    }));
    let project = bound.then_some(Box::new(c::ProjectCharterDocument {
        name: b"implementer".as_slice().into(),
        revision: 4,
        source: b"org/shop at 1a2b3c4d5e6f".as_slice().into(),
        settings: b"builder".as_slice().into(),
        instructions: words("Café.\n"),
        parts: list([part(2, "Ask first."), part(3, "Read the spec.")]),
    }));
    c::ReadValue::Setting(Box::new(c::SettingDocument {
        overlay,
        setting: c::Setting::Charter(c::SettingNumber { number }),
        revision: 12,
        value: Some(c::SettingValue::Charter(Box::new(charter))),
        application: None,
        proposed: None,
        project,
    }))
}

fn shown(words: &[&[u8]], value: c::ReadValue) -> Box<[u8]> {
    let mut reading = reading(words);
    reading.pages.insert(1, Readout::Jig(c::ReadResult::Value(value)));
    let printed = document::print(
        document::Document::ReadSet(&reading, None, Wall::from_nanos(0)),
        reading.called.json,
        &LIMITS,
        &Parts,
    );
    assert_eq!(printed.status, 0);
    assert!(printed.stderr.is_empty());
    printed.stdout
}

const BOUND_HEAD: &str = "charter implementer for project 3, as a new task would run by it\n\
-- the project's .project/charters/implementer.md, revision 4, org/shop at 1a2b3c4d5e6f, run by charter 2's settings: 7 bytes\n\
Café.\n\
-- the deployment's lines, charter 2's overlay: 13 bytes\n\
Use the box.\n";

#[test]
fn charter_preview_prints_sources_and_filters_only_parts() {
    let cases: [(&[&[u8]], &str); 2] = [
        (
            &[b"settings", b"show", b"charter", b"5", b"--project", b"3"],
            "-- part on question; the project's: 10 bytes\nAsk first.\n\
          -- part on first, question; the project's: 14 bytes\nRead the spec.\n\
          -- part on question; the deployment's overlay: 4 bytes\nAsk.\n\
          -- part on first; the deployment's overlay: 6 bytes\nBoot.\n",
        ),
        (
            &[b"settings", b"show", b"charter", b"5", b"--project", b"3", b"--reason", b"first"],
            "-- part on first, question; the project's: 14 bytes\nRead the spec.\n\
          -- part on first; the deployment's overlay: 6 bytes\nBoot.\n",
        ),
    ];
    for (argv, expected) in cases {
        let printed = shown(argv, charter(true, true, 2));
        assert_eq!(printed.strip_prefix(BOUND_HEAD.as_bytes()), Some(expected.as_bytes()));
    }
}

#[test]
fn unbound_charter_preview_splits_the_overlay_and_preserves_plain_show() {
    let argv: &[&[u8]] = &[b"settings", b"show", b"charter", b"2", b"--project", b"3"];
    let value = charter(false, true, 2);
    assert_eq!(
        shown(argv, value.clone()).as_ref(),
        b"charter 2 builder for project 3, as a new task would run by it\n\
          -- charter 2 builder, the stock's: 6 bytes\nStock.\n\
          -- the deployment's lines, charter 2's overlay: 13 bytes\nUse the box.\n\
          -- part on first; the stock's: 6 bytes\nBegin.\n\
          -- part on question; the deployment's overlay: 4 bytes\nAsk.\n\
          -- part on first; the deployment's overlay: 6 bytes\nBoot.\n"
    );
    for (number, revision, origin) in [
        (2, 1, b"-- charter 2 builder, the stock's: 6 bytes".as_slice()),
        (2, 12, b"-- charter 2 builder: 6 bytes"),
        (33, 12, b"-- charter 33 builder, set whole: 6 bytes"),
    ] {
        let number_word = skein_lib::Decimal::of(u64::from(number));
        let argv: &[&[u8]] = &[b"settings", b"show", b"charter", number_word.as_bytes(), b"--project", b"3"];
        let c::ReadValue::Setting(mut value) = charter(false, false, number) else { unreachable!("charter fixture") };
        value.revision = revision;
        let printed = shown(argv, c::ReadValue::Setting(value));
        assert!(skein_lib::bytes::find(&printed, origin).is_some());
    }
    assert_eq!(
        shown(&[b"settings", b"show", b"charter", b"2"], value.clone()),
        render::settings_text(&value, None, &LIMITS)
    );
}

#[test]
fn charter_preview_json_keeps_the_whole_read_despite_the_reason() {
    let plain = render::settings_json(&charter(false, true, 2), &LIMITS);
    let printed = shown(
        &[b"settings", b"show", b"charter", b"5", b"--project", b"3", b"--reason", b"first", b"--json"],
        charter(true, true, 2),
    );
    assert_eq!(printed.strip_prefix(plain.strip_suffix(b"}").expect("object end")), Some(
        r#","project":{"name":"implementer","revision":4,"source":"org/shop at 1a2b3c4d5e6f","settings":"builder","instructions":"Café.\n","parts":[{"on":["question"],"instructions":"Ask first."},{"on":["first","question"],"instructions":"Read the spec."}]}}"#.as_bytes()
    ));
}
