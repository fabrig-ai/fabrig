use crate::tests::fixtures as f;
use jig_client::contract::{
    self as c, AtBound, EscalationHolder as Holder, Unbounded, WaitKind as Kind, WaitObject as Object, waits::Bound,
};
use skein_lib::{List, Wall};
use temper_client_protocol_wire::waits::{CHANGE_CHECKING, LANDING_ANCESTRY, THING_ENTRY, THING_TASK};

pub(super) const DESCENDANT_JSON: &[u8] = br#""wait":{"task":44,"kind":"party","object":{"entry":{"kind":"question","number":3,"at":{"person":73}}},"since":1792854120000000000,"bound":{"at":1792882920000000000,"then":"pass_up"},"hears":{"person":73}}"#;
const SOONER: Wall = Wall::from_nanos(1_792_868_520_000_000_000);

// The sample in client/ctl/results.md, 1 supplies the first three numbers.
const OWN_QUESTION: u64 = 228_316;
const ANSWERER: u64 = 34;
const DEPENDENCY: u64 = 17;
const RESOURCE_HOLDER: u64 = 7;
const DELEGATE: u64 = 22;

#[test]
fn task_summary_uses_the_supplied_nearest_even_if_its_wait_is_not_listed() {
    let mut task = f::waiting_goal();
    task.wait_count = 1; // The goal's one own wait is outside the supplied lines.
    task.nearest = Some(c::Nearest { kind: Kind::News, at: SOONER, then: AtBound::Hold });
    let text = read(task.clone(), false);
    assert!(f::has(&text, "\nwaiting on: you: question 3 of task 44 · 1 waiting · since 2026-10-24 15:02 UTC · news holds at 2026-10-24 19:02 UTC\nwaits: 1\n  … 1 more\n".as_bytes()));
    let json = read(task, true);
    f::unique_keys(&json.stdout).expect("valid task JSON");
    for field in [
        DESCENDANT_JSON,
        br#""nearest":{"at":1792868520000000000,"then":"hold","kind":"news"}"#,
        br#""waits":[],"wait_count":1"#,
    ] {
        assert!(f::has(&json, field), "{json:?}");
    }
}

#[test]
fn a_task_waits_block_prints_waits_in_document_order() {
    let mut task = f::goal(f::TASK, c::WaitsOn::Question { number: OWN_QUESTION });
    let at = Wall::from_nanos(1_795_507_620_000_000_000);
    let mut question = wait(
        Kind::Question,
        Object::Question { number: OWN_QUESTION, answerer: ANSWERER },
        Bound::At { at, then: AtBound::PassUp },
        Some(Holder::Task(ANSWERER)),
    );
    question.since = Wall::from_nanos(1_795_500_420_000_000_000);
    let mut dependency = wait(
        Kind::Dependency,
        Object::Task(DEPENDENCY),
        Bound::None(Unbounded::NoDeadline),
        Some(Holder::Task(ANSWERER)),
    );
    dependency.since = Wall::from_nanos(1_795_499_400_000_000_000);
    task.wait = Some(question.clone());
    task.nearest = Some(c::Nearest { at, then: AtBound::PassUp, kind: Kind::Question });
    let room = wait(
        Kind::Room,
        Object::Procedure {
            step: CHANGE_CHECKING,
            things: list([c::Thing { what: THING_ENTRY, number: 7, owner: Some(51) }]),
        },
        Bound::None(Unbounded::Behind(51)),
        None,
    );
    let step = wait(
        Kind::Step,
        Object::Procedure {
            step: LANDING_ANCESTRY,
            things: list([c::Thing { what: THING_TASK, number: 15090, owner: None }]),
        },
        Bound::At { at, then: AtBound::Hold },
        None,
    );
    task.waits = list([question, dependency, room, step]);
    task.wait_count = task.waits.len();
    f::check_golden("task_own_waits", &read(task.clone(), false).stdout);
    let json = read(task, true);
    let expected = br#""waits":[{"task":41,"kind":"question","object":{"question":{"number":228316,"answerer":34}},"since":1795500420000000000,"bound":{"at":1795507620000000000,"then":"pass_up"},"hears":{"task":34}},{"task":41,"kind":"dependency","object":{"task":17},"since":1795499400000000000,"bound":{"none":"no_deadline"},"hears":{"task":34}},{"task":41,"kind":"room","object":{"procedure":{"step":106,"things":[{"what":1,"number":7,"owner":51}]}},"since":1792854120000000000,"bound":{"none":{"behind":51}},"hears":null},{"task":41,"kind":"step","object":{"procedure":{"step":210,"things":[{"what":2,"number":15090,"owner":null}]}},"since":1792854120000000000,"bound":{"at":1795507620000000000,"then":"hold"},"hears":null}],"wait_count":4"#;
    assert!(f::has(&json, expected), "{json:?}");
}

#[test]
fn absent_waits_leave_the_text_alone_and_give_empty_json_fields() {
    let task = f::goal(f::TASK, c::WaitsOn::Nothing);
    let text = read(task.clone(), false);
    assert!(f::has(&text, b"\nwaiting on: Nothing awaited\n"));
    assert!(!f::has(&text, b"\nwaits:"));
    let json = read(task, true);
    assert!(f::has(&json, br#""waits_on":"nothing","wait":null,"nearest":null,"waits":[],"wait_count":0"#));
}

#[test]
fn held_and_account_summaries_keep_their_words_before_the_wait() {
    let run = c::TaskRunWait::Account { account: None, source: None, why: c::TaskWaitWhy::Empty };
    let first = wait(Kind::Account, Object::Run(run), Bound::At { at: SOONER, then: AtBound::Hold }, None);
    let mut task = f::goal(f::TASK, c::WaitsOn::Nothing);
    task.run_wait = Some(run);
    task.wait = Some(first.clone());
    task.nearest = Some(c::Nearest { kind: Kind::Account, at: SOONER, then: AtBound::Hold });
    task.waits = list([first]);
    task.wait_count = 1;
    for (task, expected) in [
        (task.clone(), "\nwaiting on: an account · since 2026-10-24 15:02 UTC · holds at 2026-10-24 19:02 UTC\n"),
        (
            f::held_task(task, c::HoldCategory::Effects),
            "\nwaiting on: held · since 2026-10-24 15:02 UTC · holds at 2026-10-24 19:02 UTC\n",
        ),
    ] {
        let text = read(task, false);
        assert!(f::has(&text, expected.as_bytes()), "{text:?}");
    }
}

type HolderCase = (Option<Holder>, &'static str, &'static str);
type ObjectCase = (Kind, Object, &'static str, &'static str, HolderCase);
const UNHEARD: HolderCase = (None, "", "null");
const YOU: HolderCase = (Some(Holder::Person(f::PARTY)), " · hears you", r#"{"person":73}"#);
const PROJECT_ROLE: HolderCase =
    (Some(Holder::Role { project: 2, role: 7 }), " · hears role 7 in project 2", r#"{"role":{"project":2,"role":7}}"#);
const KINDS: usize = 12;
const BOUND_FORMS: usize = 8;
const OMITTED: u32 = 2;
fn object_cases() -> [ObjectCase; KINDS] {
    [
        (
            Kind::Party,
            Object::Entry { kind: c::WaitedFor::Proposal, number: 28, at: Holder::Person(f::PARTY) },
            "party proposal 28 at you",
            r#""entry":{"kind":"proposal","number":28,"at":{"person":73}}"#,
            YOU,
        ),
        (
            Kind::Question,
            Object::Question { number: 3, answerer: ANSWERER },
            "question 3 to task 34",
            r#""question":{"number":3,"answerer":34}"#,
            UNHEARD,
        ),
        (Kind::Dependency, Object::Task(DEPENDENCY), "dependency task 17", r#""task":17"#, UNHEARD),
        (
            Kind::Resource,
            Object::Resource {
                name: c::ResourceName {
                    connector: temper_client_protocol_wire::FORGE,
                    path: list([Box::from(&b"slot"[..]), Box::from(&b"one"[..])]),
                },
                holder: Some(RESOURCE_HOLDER),
            },
            "resource slot/one held by task 7",
            r#""resource":{"name":{"connector":1,"path":["736c6f74","6f6e65"]},"holder":7}"#,
            UNHEARD,
        ),
        (
            Kind::Account,
            Object::Run(c::TaskRunWait::Account {
                account: Some(12),
                source: Some(4),
                why: c::TaskWaitWhy::Refreshing,
            }),
            "account 12 source 4: refreshing",
            r#""run":{"account":{"account":12,"source":4,"why":"refreshing"}}"#,
            UNHEARD,
        ),
        (
            Kind::Host,
            Object::Run(c::TaskRunWait::Worker { worker: 2, until: SOONER }),
            "host waiting for worker 2 to return, until 1792868520000000000",
            r#""run":{"worker":{"worker":2,"until":1792868520000000000}}"#,
            UNHEARD,
        ),
        (Kind::Delegate, Object::Task(DELEGATE), "delegate task 22", r#""task":22"#, UNHEARD),
        (
            Kind::News,
            Object::News { subscriptions: list([5, 6]), timer: Some(SOONER) },
            "news subscriptions 5, 6 and timer at 2026-10-24 19:02 UTC",
            r#""news":{"subscriptions":[5,6],"timer":1792868520000000000}"#,
            UNHEARD,
        ),
        (
            Kind::Step,
            Object::Procedure {
                step: 5,
                things: list([
                    c::Thing { what: 77, number: 51, owner: None },
                    c::Thing { what: 8, number: 52, owner: Some(ANSWERER) },
                ]),
            },
            "step 5: thing 77 51, forge 52 of task 34",
            r#""procedure":{"step":5,"things":[{"what":77,"number":51,"owner":null},{"what":8,"number":52,"owner":34}]}"#,
            UNHEARD,
        ),
        (
            Kind::Room,
            thing(Some(ANSWERER)),
            "room 5: rule 51 of task 34",
            r#""procedure":{"step":5,"things":[{"what":7,"number":51,"owner":34}]}"#,
            UNHEARD,
        ),
        (Kind::Turn, Object::Task(ANSWERER), "turn task 34", r#""task":34"#, UNHEARD),
        (
            Kind::SignOff,
            thing(None),
            "sign-off 5: rule 51",
            r#""procedure":{"step":5,"things":[{"what":7,"number":51,"owner":null}]}"#,
            PROJECT_ROLE,
        ),
    ]
}
fn bound_cases() -> [(Bound, &'static str, &'static str); BOUND_FORMS] {
    [
        (
            Bound::At { at: SOONER, then: AtBound::Overdue },
            "is overdue at 2026-10-24 19:02 UTC",
            r#""at":1792868520000000000,"then":"overdue""#,
        ),
        (
            Bound::At { at: SOONER, then: AtBound::PassUp },
            "passes up at 2026-10-24 19:02 UTC",
            r#""at":1792868520000000000,"then":"pass_up""#,
        ),
        (Bound::None(Unbounded::NoDeadline), "no bound: no deadline", r#""none":"no_deadline""#),
        (
            Bound::At { at: SOONER, then: AtBound::Hold },
            "holds at 2026-10-24 19:02 UTC",
            r#""at":1792868520000000000,"then":"hold""#,
        ),
        (Bound::None(Unbounded::OperatorHold), "no bound: an operator's hold", r#""none":"operator_hold""#),
        (Bound::None(Unbounded::Behind(ANSWERER)), "no bound: behind task 34", r#""none":{"behind":34}"#),
        (Bound::None(Unbounded::Standing), "no bound: standing", r#""none":"standing""#),
        (
            Bound::At { at: SOONER, then: AtBound::GiveUp },
            "gives up at 2026-10-24 19:02 UTC",
            r#""at":1792868520000000000,"then":"give_up""#,
        ),
    ]
}

#[test]
#[expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    reason = "ordinary test assembles expected wait lines and JSON"
)]
fn wait_rows_name_every_kind_and_bound() {
    use core::fmt::Write;
    let cases = object_cases();
    let mut task = f::goal(f::TASK, c::WaitsOn::Nothing);
    let row_count = u32::try_from(cases.len()).expect("fixture rows");
    task.waits = List::with_capacity(row_count);
    task.wait_count = row_count.checked_add(OMITTED).expect("two omitted rows");
    let mut text_block = format!("\nwaits: {}\n", task.wait_count);
    let mut json_rows = String::from(r#""waits":["#);
    let mut separator = "";
    for ((kind, object, words, json, (holder, holder_words, holder_json)), (bound, because, bound_json)) in
        cases.into_iter().zip(bound_cases().into_iter().cycle())
    {
        task.waits.push(wait(kind, object, bound, holder)).expect("one case per row");
        writeln!(text_block, "  {words} · since 2026-10-24 15:02 UTC · {because}{holder_words}").expect("text row");
        let kind = words.split_whitespace().next().expect("row kind");
        write!(json_rows, r#"{separator}{{"task":41,"kind":"{kind}","object":{{{json}}},"since":1792854120000000000,"bound":{{{bound_json}}},"hears":{holder_json}}}"#).expect("JSON row");
        separator = ",";
    }
    text_block.push_str("  … 2 more\n");
    let text = read(task.clone(), false);
    assert!(f::has(&text, text_block.as_bytes()), "{text:?}");
    let json = read(task, true);
    json_rows.push_str(r#"],"wait_count":14"#);
    assert!(f::has(&json, json_rows.as_bytes()), "{json:?}");
}

fn thing(owner: Option<u64>) -> Object {
    Object::Procedure { step: 5, things: list([c::Thing { what: 7, number: 51, owner }]) }
}
fn wait(kind: Kind, object: Object, bound: Bound, hears: Option<Holder>) -> c::Wait {
    c::Wait { task: f::TASK, kind, object, since: f::QUESTION_SINCE, bound, hears }
}
fn list<T, const N: usize>(rows: [T; N]) -> List<T> {
    let mut list = List::with_capacity(u32::try_from(N).expect("fixture size"));
    for row in rows {
        assert!(list.push(row).is_ok());
    }
    list
}
fn read(task: c::Task, json: bool) -> crate::jig::Printed {
    let answer = f::read_answer(if json { f::TASK_JSON_CALL } else { f::TASK_CALL }, f::task_result(task));
    assert_eq!(answer.status, 0, "{answer:?}");
    answer
}
