use super::super::fixtures as f;
use crate::jig::Printed;
use jig_client::contract::{self as c, AtBound, EscalationHolder as Holder, WaitKind, WaitObject, waits::Bound};
use skein_lib::{List, Wall, Writer};

// The three rows in ctl/results.md, 1's waits example.
const ROWS: u32 = 3;
const PROJECT_NUMBER: u32 = 2;
const PROPOSER: u64 = 812;
const QUESTIONER: u64 = 820;
const DEPENDENT: u64 = 833;
const PROJECT: &[&str] = &["waits", "--project", "2"];
pub(in crate::tests) const FILTERED: &[&str] =
    &["waits", "--project", "2", "--kind", "party", "--party", "73", "--due", "2h"];
const MORE: &[u8] = b"read on with: temper ctl waits --project 2 --after 1122\n";
const FILTERED_MORE: &[u8] =
    b"read on with: temper ctl waits --project 2 --kind party --party 73 --due 7200 --after 1122\n";
const HEADING: &[u8] = b"waits of project 2 \xc2\xb7 now 2026-11-26 09:40 UTC\n";
const NOW: Wall = Wall::from_nanos(1_795_686_000_000_000_000);
const OVERDUE: Wall = Wall::from_nanos(1_795_604_520_000_000_000);

#[test]
fn project_waits_print_each_row_and_continue_with_all_filters() {
    let first = print(PROJECT, example_rows(), Some(f::cursor()));
    f::check_golden("project_waits", &first.stdout);
    let continued = f::extended(PROJECT, &["--after", "1122"]);
    let last = print(continued.as_slice(), example_rows(), None);
    assert_eq!(last.stdout.as_ref(), first.stdout.strip_suffix(MORE).expect("first page's continuation"));

    // The filter only admits the bounded wait at a role the party holds.
    let mut filtered = example_rows();
    filtered.truncate(1);
    let continued = f::extended(FILTERED, &["--after", "1111"]);
    let page = print(continued.as_slice(), filtered, Some(f::cursor()));
    assert!(page.stdout.ends_with(FILTERED_MORE), "{page:?}");
}

#[test]
fn empty_project_waits_distinguish_filters_from_continuation() {
    for (extra, words) in [
        (&[][..], b"no waits\n".as_slice()),
        (&["--after", "1122"], b"no waits\n"),
        (&["--kind", "sign-off"], b"no waits match\n"),
        (&["--party", "73"], b"no waits match\n"),
        (&["--due", "1s"], b"no waits match\n"),
    ] {
        let argv = f::extended(PROJECT, extra);
        let answer = print(argv.as_slice(), List::with_capacity(0), None);
        assert_eq!(answer.stdout.strip_prefix(HEADING), Some(words));
    }
}

#[test]
fn project_waits_send_the_resolved_project_and_each_filter() {
    let mut root = f::signed_in(f::PARTY);
    for (word, kind) in WAIT_KINDS {
        let call = submit(
            &mut root,
            &["waits", "--project", "shop", "--kind", word, "--party", "73", "--due", "1s", "--after", "1122"],
        );
        assert_eq!(
            root.reads[&call].1,
            c::Read::Waits {
                project: f::PROJECT,
                kind: Some(*kind),
                party: Some(73),
                due: Some(1_000_000_000),
                after: Some(f::cursor()),
            }
        );
        assert_eq!(answer(&mut root, call, List::with_capacity(0), None).status, 0);
    }
    let call = submit(&mut root, &["waits", "--project", "2", "--due", "720h"]);
    assert_eq!(
        root.reads[&call].1,
        c::Read::Waits {
            project: PROJECT_NUMBER,
            kind: None,
            party: None,
            due: Some(2_592_000_000_000_000),
            after: None,
        }
    );
}

#[test]
fn project_waits_refuse_invalid_kinds_and_due_durations_locally() {
    let choices = refusal(&["waits", "--project", "2", "--kind", "sign_off"]);
    assert_eq!(
        choices.as_ref(),
        b"waits: --kind takes one of party, question, dependency, resource, account, host, delegate, news, step, \
          room, turn or sign-off; temper ctl help waits"
    );
    for invalid in ["0s", "2592001", "-1", "1.5h"] {
        let error = refusal(&["waits", "--project", "2", "--due", invalid]);
        assert_eq!(error.as_ref(), b"waits: --due DURATION takes a duration: seconds, or a number with s, m or h, from 1 to 2592000 seconds; temper ctl help waits");
    }
    for input in [
        br#"{"tool":"waits","arguments":{"project":2,"due":0}}"#.as_slice(),
        br#"{"tool":"waits","arguments":{"project":2,"due":2592001}}"#,
        br#"{"tool":"waits","arguments":{"project":2,"due":"1s"}}"#,
    ] {
        f::json(input).expect_err("invalid JSON call");
    }
}

#[test]
fn project_waits_json_uses_page_time_and_has_no_continuation_after_the_last_row() {
    let argv = f::extended(PROJECT, &["--json"]);
    let first = print(argv.as_slice(), example_rows(), Some(f::cursor()));
    f::unique_keys(&first.stdout).expect("waits JSON keys are unique");
    f::check_golden("project_waits_json", &first.stdout);
    let mut root = f::signed_in(f::PARTY);
    let next = f::extended(PROJECT, &["--after", "1122", "--json"]);
    let call = submit(&mut root, next.as_slice());
    let mut rows = example_rows();
    rows.truncate(1);
    let row = rows.get_mut(0).expect("overdue row");
    row.wait.bound = Bound::At { at: NOW, then: AtBound::Overdue };
    let last = answer(&mut root, call, rows, None);
    assert!(last.stdout.ends_with(br#""overdue":true}],"read_on":null}"#), "{last:?}");
}

#[test]
fn unknown_project_waits_use_task_lists_refusal() {
    let mut root = f::signed_in(f::PARTY);
    let missing_name = root.submit(&["waits", "--project", "missing"]);
    let missing_number = submit(&mut root, PROJECT);
    root.serve(missing_number, c::ReadResult::None(c::ReadNone::Unknown));
    for (call, status, stderr) in
        [(missing_name, 1, b"unknown project\n".as_slice()), (missing_number, 2, b"unknown\n")]
    {
        let printed = root.answer(call).expect("unknown project answered");
        assert_eq!(printed.status, status);
        assert_eq!(printed.stderr.as_ref(), stderr);
    }
}

fn refusal(argv: &[&str]) -> Box<[u8]> {
    let why = f::parse(argv).expect_err("invalid waits argument is refused locally");
    let mut words = List::with_capacity(u32::try_from(argv.len()).expect("bounded argv fixture"));
    for word in argv {
        words.push(word.as_bytes()).expect("one slot per word");
    }
    let mut measured = Writer::measure(227);
    crate::table::refusal(why, words.as_slice(), &mut measured).expect("argument refusal fits");
    let mut out = Writer::new(measured.written());
    crate::table::refusal(why, words.as_slice(), &mut out).expect("argument refusal fits exactly");
    out.finish()
}

fn submit(root: &mut f::Harness, argv: &[&str]) -> u32 {
    // One read is expected. The common harness counts only its pre-existing tools.
    root.pending_reads.push(root.next.checked_add(1).expect("call number"));
    root.submit(argv)
}

fn print(argv: &[&str], rows: List<c::WaitRow>, after: Option<c::Cursor>) -> Printed {
    let mut root = f::signed_in(f::PARTY);
    let call = submit(&mut root, argv);
    let printed = answer(&mut root, call, rows, after);
    assert_eq!(printed.status, 0, "{printed:?}");
    printed
}

fn answer(root: &mut f::Harness, call: u32, rows: List<c::WaitRow>, after: Option<c::Cursor>) -> Printed {
    root.serve(call, c::ReadResult::Value(c::ReadValue::Waits(Box::new(c::WaitPage { now: NOW, rows, after }))));
    root.answer(call).expect("one finished waits read")
}

fn example_rows() -> List<c::WaitRow> {
    let mut rows = List::with_capacity(ROWS);
    for (task, title, kind, object, since, bound, hears) in [
        (
            PROPOSER,
            b"Store shape".as_slice(),
            WaitKind::Party,
            WaitObject::Entry {
                kind: c::WaitedFor::Proposal,
                number: 3,
                at: Holder::Role { project: PROJECT_NUMBER, role: 1 },
            },
            Wall::from_nanos(1_795_600_920_000_000_000),
            Bound::At { at: OVERDUE, then: AtBound::Overdue },
            None,
        ),
        (
            QUESTIONER,
            b"S7 history in task logs",
            WaitKind::Question,
            WaitObject::Question { number: 4, answerer: PROPOSER },
            Wall::from_nanos(1_795_680_060_000_000_000),
            Bound::At { at: Wall::from_nanos(1_795_687_260_000_000_000), then: AtBound::PassUp },
            Some(Holder::Task(PROPOSER)),
        ),
        (
            DEPENDENT,
            b"Read the layout",
            WaitKind::Dependency,
            WaitObject::Task(QUESTIONER),
            Wall::from_nanos(1_795_677_120_000_000_000),
            Bound::None(c::Unbounded::NoDeadline),
            Some(Holder::Task(QUESTIONER)),
        ),
    ] {
        let mut chip = f::goal(task, c::WaitsOn::Nothing).chip;
        chip.title = f::document_text(title);
        chip.details.details.project = PROJECT_NUMBER;
        rows.push(c::WaitRow { chip, wait: c::Wait { task, kind, object, since, bound, hears } }).expect("example row");
    }
    rows
}

const WAIT_KINDS: &[(&str, WaitKind)] = &[
    ("party", WaitKind::Party),
    ("question", WaitKind::Question),
    ("dependency", WaitKind::Dependency),
    ("resource", WaitKind::Resource),
    ("account", WaitKind::Account),
    ("host", WaitKind::Host),
    ("delegate", WaitKind::Delegate),
    ("news", WaitKind::News),
    ("step", WaitKind::Step),
    ("room", WaitKind::Room),
    ("turn", WaitKind::Turn),
    ("sign-off", WaitKind::SignOff),
];
