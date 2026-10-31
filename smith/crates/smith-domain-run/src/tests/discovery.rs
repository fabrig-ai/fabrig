use super::*;

#[test]
fn an_admitted_run_reads_its_checkout_then_opens_main_with_the_whole_budget() {
    let mut harness = Harness::new(LIMITS);
    let requests = harness.start(7, charter());
    let [Request::Admitted { host_run, run }, Request::Read { owner, at, max, deadline }] = &*requests else {
        panic!("expected an admitted run, got {requests:?}");
    };
    assert_eq!((*host_run, owner), (Token::new(7), run));
    assert_eq!(at, &Place { root: Token::new(900), path: bytes(b"AGENTS.md") });
    assert_eq!((*max, *deadline), (LIMITS.guide_bytes, Time::ZERO.saturating_add(LIMITS.io_timeout)));
    assert_eq!((harness.domain.runs(), harness.domain.conversations()), (1, 1), "main has its slot from the start");

    let read = Read::Text { text: bytes(b"Run the tests."), whole: true };
    let requests = harness.step(Event::Read { owner: *run, read });
    let [Request::Open { conversation: _, opening }] = &*requests else {
        panic!("expected main to open, got {requests:?}");
    };
    let mut found = Found::with_capacity(1);
    found.guides.push(Guide { repository: 0, text: bytes(b"Run the tests."), whole: true }).expect("room");
    let expected = Opening {
        final_turn: None,
        generated_schema_bytes: LIMITS.schema_costs.finish(&charter().outcome).expect("schema count"),
        identity: opening.identity,
        ordinal: 0,
        activation: 1,
        host_tools: charter().grants.host_tools,
        deliver: false,
        llm: charter().llm,
        line: 7168,
        system: crate::prompt::system(&charter(), Some(&workspace()), &found, BUDGET, false),
        prompt: bytes(crate::prompt::BEGIN),
        summary: bytes(crate::prompt::SUMMARY),
        preamble: bytes(crate::prompt::PREAMBLE),
        tools: charter().grants.tools,
        workspace: Some(workspace()),
        budget: BUDGET,
        finish: true,
        families: crate::charter::Families { tools: charter().grants.tools, agents: false },
        transcript: None,
        wait: true,
    };
    assert_eq!(opening, &expected);
    assert_eq!(harness.domain.next_deadline(), Some(Time::ZERO.saturating_add(BUDGET.time)));
}

#[test]
fn a_run_looks_for_checks_in_writable_repositories_when_a_change_must_pass_them() {
    let mut harness = Harness::new(LIMITS);
    let checkout = Workspace {
        directories: Box::new([
            Directory {
                host: Box::from(b"/host".as_slice()),
                name: bytes(b"temper"),
                root: Token::new(900),
                writable: true,
                git: true,
                conflicts: Box::new([]),
            },
            Directory {
                host: Box::from(b"/host".as_slice()),
                name: bytes(b"docs"),
                root: Token::new(901),
                writable: false,
                git: true,
                conflicts: Box::new([]),
            },
        ]),
    };
    let outcome = OutcomeSpec {
        change: Some(ChangeSpec {
            checks_must_pass: true,
            fields: Box::new([
                FieldRule { name: b"title".as_slice().into(), max: 1024 },
                FieldRule { name: b"body".as_slice().into(), max: 1024 },
            ]),
        }),
        verdicts: Box::new([]),
        report: None,
        failure: None,
    };
    let requests = harness.start_workspace(
        1,
        Charter { outcome, reserve: crate::charter::Reserve { turns: 0, time: LIMITS.check_timeout }, ..charter() },
        Some(checkout),
    );
    let [Request::Admitted { run, .. }, Request::Read { at, .. }] = &*requests else {
        panic!("expected a read, got {requests:?}");
    };
    let run = *run;
    assert_eq!(at.root, Token::new(900));
    let requests = harness.step(Event::Read { owner: run, read: Read::Failed });
    let [Request::Probe { owner, at, deadline }] = &*requests else {
        panic!("expected a probe, got {requests:?}");
    };
    assert_eq!((owner, at), (&run, &Place { root: Token::new(900), path: bytes(b".temper/pre-pr") }));
    assert_eq!(*deadline, Time::ZERO.saturating_add(LIMITS.io_timeout));
    let requests = harness.step(Event::Probed { owner: run, executable: true });
    let [Request::Read { at, .. }] = &*requests else {
        panic!("expected a read, got {requests:?}");
    };
    assert_eq!(at.root, Token::new(901), "a read-only repository has no checks to look for");
    let requests = harness.step(Event::Read { owner: run, read: Read::Missing });
    let [Request::Open { opening, .. }] = &*requests else {
        panic!("expected main to open, got {requests:?}");
    };
    let system = &opening.system;
    let marked = b"- `temper`, which you may change, a git working tree, with checks (`.temper/pre-pr`)\n";
    assert!(skein_lib::bytes::find(system, marked).is_some(), "the checkout says which has checks");
}

#[test]
fn a_report_only_run_probes_checks_in_its_writable_directory() {
    let mut harness = Harness::new(LIMITS);
    let mut mounted = workspace();
    mounted.directories[0].writable = true;
    let requests = harness.start_workspace(1, text_charter(false), Some(mounted));
    let [Request::Admitted { run, .. }, Request::Read { .. }] = &*requests else {
        panic!("report run begins discovery: {requests:?}");
    };
    let run = *run;
    let requests = harness.step(Event::Read { owner: run, read: Read::Missing });
    let [Request::Probe { at, .. }] = &*requests else {
        panic!("writable report directory probes checks: {requests:?}");
    };
    assert_eq!(at.root, Token::new(900));
    let requests = harness.step(Event::Probed { owner: run, executable: true });
    let [Request::Open { opening, .. }] = &*requests else {
        panic!("report opens after the probe: {requests:?}");
    };
    assert!(skein_lib::bytes::find(&opening.system, b"with checks (`.temper/pre-pr`)").is_some());
}

#[test]
fn a_run_with_nothing_to_look_for_opens_main_at_once() {
    let mut harness = Harness::new(LIMITS);
    let requests = harness.start_workspace(1, charter(), None);
    let [Request::Admitted { .. }, Request::Open { .. }] = &*requests else {
        panic!("expected an admitted run and main, got {requests:?}");
    };
}

#[test]
fn a_guide_that_is_not_text_is_not_there() {
    let mut harness = Harness::new(LIMITS);
    let run = harness.prepare(1);
    let requests = harness.step(Event::Read { owner: run, read: Read::NotText });
    let [Request::Open { opening, .. }] = &*requests else { panic!("expected main to open, got {requests:?}") };
    assert_eq!(
        opening.system,
        crate::prompt::system(&charter(), Some(&workspace()), &Found::with_capacity(1), opening.budget, false)
    );
}
