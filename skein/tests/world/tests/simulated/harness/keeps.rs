use super::{run_story, setup};
use skein_fake_person_browser::{
    Harness, Kept, Outcome, Script, Step,
    harness::{Element, Origin, Scripted, Shown},
    person,
};
use skein_lib::{Duration, Time};

const SECOND: Duration = Duration::from_secs(1);

fn keep_harness() -> Harness {
    setup(vec![Origin { host: b"localhost".as_slice().into(), pages: vec![] }])
}

fn keep_page() -> Scripted {
    Scripted::new().page(
        b"/",
        Shown {
            nodes: vec![],
            elements: [b"input[type=hidden]".as_slice(), b"input"]
                .into_iter()
                .flat_map(|selector| {
                    [
                        Element {
                            selector: selector.into(),
                            attributes: vec![
                                (b"value".as_slice().into(), b"first".as_slice().into()),
                                (b"data-note".as_slice().into(), b"named".as_slice().into()),
                                (b"data-empty".as_slice().into(), Box::new([])),
                            ],
                        },
                        Element {
                            selector: selector.into(),
                            attributes: vec![(b"value".as_slice().into(), b"second".as_slice().into())],
                        },
                    ]
                })
                .collect(),
        },
    )
}

fn keep(selector: &[u8], attribute: Option<&[u8]>, name: &[u8]) -> Step {
    Step::Keep { selector: selector.into(), attribute: attribute.map(Box::from), name: name.into() }
}

fn keep_script<const N: usize>(steps: [Step; N]) -> Script {
    Script::new(std::iter::once(Step::Visit { origin: 0, path: b"/".as_slice().into() }).chain(steps).collect())
}

fn kept(person: u32, name: &[u8], value: &[u8]) -> Kept {
    Kept { person, name: name.into(), value: value.into() }
}

#[test]
fn keeps_read_values_named_and_empty_attributes_in_one_run() {
    let mut harness = keep_harness();
    let mut person = keep_script([
        keep(b"input[type=hidden]", None, b"read"),
        keep(b"input", Some(b"data-note"), b"read"),
        keep(b"input", Some(b"data-empty"), b"read"),
        keep(b"input", None, b"read"),
    ]);
    assert_eq!(run_story(&mut harness, &mut [&mut person], keep_page()).outcome, Outcome::Done);
    assert_eq!(
        harness.kept(),
        &[kept(0, b"read", b"first"), kept(0, b"read", b"named"), kept(0, b"read", b""), kept(0, b"read", b"first"),]
    );
}

#[test]
fn a_keep_without_an_element_fails_its_step() {
    let mut person = keep_script([keep(b"#missing", None, b"read")]);
    let run = run_story(&mut keep_harness(), &mut [&mut person], keep_page());
    let Outcome::Failed { person: 0, step: 1, report } = run.outcome else {
        panic!("missing selector fails keep: {:?}", run.outcome)
    };
    assert!(report.contains("keep selector #missing attribute value: no element"), "{report}");
}

#[test]
fn a_keep_without_an_attribute_fails_its_step() {
    let mut person = keep_script([keep(b"input", Some(b"absent"), b"read")]);
    let run = run_story(&mut keep_harness(), &mut [&mut person], keep_page());
    let Outcome::Failed { person: 0, step: 1, report } = run.outcome else {
        panic!("missing attribute fails keep: {:?}", run.outcome)
    };
    assert!(report.contains("keep selector input attribute absent: no attribute"), "{report}");
}

#[test]
fn keeps_append_duplicate_names_in_completion_order_with_their_person() {
    let mut harness = keep_harness();
    let mut first = keep_script([keep(b"input", None, b"same"), keep(b"input", Some(b"data-note"), b"same")]);
    let mut second = keep_script([keep(b"input", Some(b"data-empty"), b"other")]);
    assert_eq!(run_story(&mut harness, &mut [&mut first, &mut second], keep_page()).outcome, Outcome::Done);
    assert_eq!(harness.kept(), &[kept(0, b"same", b"first"), kept(0, b"same", b"named"), kept(1, b"other", b"")]);
}

#[test]
fn a_new_run_clears_the_preceding_keeps() {
    let mut harness = keep_harness();
    let mut first = keep_script([keep(b"input", None, b"old")]);
    assert_eq!(run_story(&mut harness, &mut [&mut first], keep_page()).outcome, Outcome::Done);
    let mut second = keep_script([keep(b"input", Some(b"data-empty"), b"new")]);
    assert_eq!(run_story(&mut harness, &mut [&mut second], keep_page()).outcome, Outcome::Done);
    assert_eq!(harness.kept(), &[kept(0, b"new", b"")]);
}

#[test]
fn a_failed_run_retains_completed_keeps() {
    let mut harness = keep_harness();
    let mut person = keep_script([keep(b"input", None, b"read"), keep(b"#missing", None, b"absent")]);
    assert!(matches!(
        run_story(&mut harness, &mut [&mut person], keep_page()).outcome,
        Outcome::Failed { step: 2, .. }
    ));
    assert_eq!(harness.kept(), &[kept(0, b"read", b"first")]);
}

#[test]
fn a_request_with_another_host_has_its_status_awaited() {
    let mut person = Script::new(Box::new([Step::Request {
        method: skein_http::Method::Get,
        path: b"/".as_slice().into(),
        host: b"other.test".as_slice().into(),
        headers: vec![],
        body: Box::new([]),
        status: 200,
        within: SECOND,
    }]));
    let run = run_story(&mut setup(vec![]), &mut [&mut person], ready_page());
    assert_eq!(run.outcome, Outcome::Done);
    let [server] = run.children.as_slice() else { panic!("only the scripted server was spawned") };
    let [head] = server.request_heads.as_slice() else { panic!("one request, got {:?}", server.request_heads) };
    assert!(head.starts_with(b"GET / HTTP/1.1\r\n"), "{}", String::from_utf8_lossy(head));
    assert!(head.split(|b| *b == b'\n').any(|line| line == b"Host: other.test\r"), "{}", String::from_utf8_lossy(head));
}

fn ready_page() -> Scripted {
    Scripted::new()
        .page(b"/", Shown { nodes: vec![(b"heading".as_slice().into(), b"Ready".as_slice().into())], elements: vec![] })
}

#[test]
fn an_unmet_see_fails_at_its_bound_in_simulated_time() {
    let see = |name| {
        Step::Person(person::Step::See {
            find: person::Find::exactly_named(skein_ui::Role::Heading { level: 1 }, name),
            within: SECOND,
        })
    };
    let mut person = Script::new(Box::new([see(b"Ready"), see(b"Never shown")]));
    let run = run_story(&mut setup(vec![]), &mut [&mut person], ready_page());
    let Outcome::Failed { person: 0, step: 1, report } = run.outcome else {
        panic!("only the unmet See fails: {:?}", run.outcome)
    };
    assert!(
        report.contains("Missing")
            && report.contains("the browser step: bound 1000000000 ns, waited 1000000000 ns in simulated time"),
        "{report}"
    );
    assert!(report.contains("Snapshot:\nheading Ready\n") && report.contains("Screenshot: unavailable"), "{report}");
    assert!(run.end >= Time::ZERO.saturating_add(SECOND), "the step did not expire before its bound");
    assert!(
        run.end <= Time::ZERO.saturating_add(Duration::from_secs(1 + 5 + 4)),
        "the bound plus capture and close: {:?}",
        run.end
    );
}
