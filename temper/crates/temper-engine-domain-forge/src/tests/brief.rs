use super::*;
use skein_forge::api::{Answer, Check, JobAttempt, Read, Status, Step};
use temper_engine_domain_forge_change as change;

// temper-service's shipped root.brief.budgets.ci; read_bytes is 4608 and
// forge.client.answer_bytes exceeds the five-check log share of 819 bytes.
const SHIPPED_CI: u32 = 4096;
const SECTION: Token = Token::new(99);
const STEP: &[u8] = b"Each pull request adds changes/<its number>.md";
#[expect(clippy::disallowed_macros, reason = "the regression embeds recorded runner fixture bytes")]
const RUNNER_LOG: &[u8] = include_bytes!("fixtures/ci/red-changes.log");
const HEAD: [u8; 32] = [
    0xc1, 0xfb, 0x35, 0x99, 0xfa, 0x12, 0xc9, 0x76, 0x6f, 0x9c, 0xc1, 0x62, 0x84, 0x44, 0x16, 0xe8, 0x7a, 0x92, 0x91,
    0x82, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

// Contexts, descriptions, links and attempt copied from g1's red.json.
fn red_checks() -> Box<[Status]> {
    Box::new([
        Status {
            at: env().now,
            context: b"/ check (pull_request)".as_slice().into(),
            check: Check::Passed,
            description: b"Successful in 0s".as_slice().into(),
            url: b"/playground/widget/actions/runs/4/jobs/0".as_slice().into(),
            job: None,
        },
        Status {
            at: env().now,
            context: b"/ changes (pull_request)".as_slice().into(),
            check: Check::Failed,
            description: b"Failing after 1s".as_slice().into(),
            url: b"/playground/widget/actions/runs/4/jobs/1".as_slice().into(),
            job: Some(JobAttempt { head: HEAD, run: 4, job: 6, attempt: 1 }),
        },
    ])
}

fn gather(checks: Box<[Status]>, log: &[u8], budget: u32) -> Domain {
    gather_run(checks, log, budget, None)
}

fn gather_run(checks: Box<[Status]>, log: &[u8], budget: u32, run: Option<skein_forge::api::Run>) -> Domain {
    let job_bound = (budget / 5)
        .min(
            budget
                .checked_div(u32::try_from(checks.len()).expect("bounded checks").max(1))
                .expect("positive check count")
                / 2,
        )
        .max(1);
    let mut d = domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    let mut row = fixtures::opening_row();
    row.pull = Some(1);
    row.change.last_head = Some(HEAD);
    row.change.state = temper_engine_domain_forge_change::State::Repairing {
        requested: true,
        why: temper_engine_domain_forge_change::Repair::Ci,
    };
    outputs(&mut d, Event::Change { row });
    outputs(
        &mut d,
        Event::PlanBrief {
            section: SECTION,
            partner: None,
            source: BriefSource::Ci {
                item: BriefItem { repository: REPO.repository, number: 1 },
                head: BriefCommit(HEAD),
            },
        },
    );
    let started = outputs(
        &mut d,
        Event::GatherPlanned {
            task: 9,
            section: SECTION,
            parts: 2,
            bytes: budget,
            ci_budget: budget,
            deadline: env().now.saturating_add(Duration::from_secs(30)),
        },
    );
    let [Request::BriefClient { event: skein_forge_client::Event::Read { owner, read, .. } }] = started.as_ref() else {
        panic!("one checks read: {started:?}")
    };
    assert_eq!(*read, Read::Checks { commit: HEAD });
    let owner = *owner;
    let mut out = Queue::uncapped();
    assert!(brief::brief_answer(&mut d, owner, Ok(Answer::Checks(checks)), LIMITS.client.rows, &mut out));
    for _ in 0..LIMITS.client.rows.saturating_mul(2).saturating_add(1) {
        let Some(request) = out.pop() else { break };
        if let Request::BriefClient {
            event: skein_forge_client::Event::Read { read: Read::Job { attempt, max_bytes, .. }, .. },
        } = request
        {
            assert_eq!(max_bytes, job_bound, "the shipped five-check allowance reaches the provider");
            let tail = log.len().saturating_sub(usize::try_from(max_bytes).expect("u32"));
            assert!(brief::brief_answer(
                &mut d,
                owner,
                Ok(Answer::Job {
                    attempt,
                    step: match &run {
                        Some(run) => run.jobs.first().expect("job").step.clone(),
                        None => Some(Step { number: 2, name: STEP.into() }),
                    },
                    log: log[tail..].into(),
                    truncated: tail > 0,
                }),
                LIMITS.client.rows,
                &mut out
            ));
        } else if let Request::BriefClient { event: skein_forge_client::Event::Read { read: Read::Run { .. }, .. } } =
            request
        {
            let answer = match &run {
                Some(run) => Ok(Answer::Run(run.clone())),
                None => Err(skein_forge::api::Error::Missing),
            };
            assert!(brief::brief_answer(&mut d, owner, answer, LIMITS.client.rows, &mut out));
        } else if let Request::BriefSized { section: SECTION, size: Ok(size) } = request {
            assert!(size <= budget);
        } else {
            panic!("unexpected gather output: {request:?}");
        }
    }
    d
}

fn take(d: &mut Domain) -> Box<[u8]> {
    d.take_brief(SECTION).expect("retained CI section")
}

#[test]
fn a_workflow_failure_reaches_the_repair_at_shipped_limits() {
    assert_eq!(RUNNER_LOG.len(), 553, "the runner's recorded bytes");
    let bytes = take(&mut gather(red_checks(), RUNNER_LOG, SHIPPED_CI));
    let words = text(&bytes);
    for expected in [
        "/ changes (pull_request)",
        "Description: Failing after 1s",
        "Link: /playground/widget/actions/runs/4/jobs/1",
        "Failed step: 2 Each pull request adds changes/<its number>.md",
        "Missing changes/1.md",
    ] {
        assert!(words.contains(expected), "{expected}: {words}");
    }
    assert!(!words.contains("cut]"));
    assert!(!words.contains("truncated"));
}

#[test]
fn a_long_log_is_cut_from_its_front_keeping_its_check() {
    let mut log = skein_lib::List::with_capacity(20_018);
    for _ in 0_u32..20_000 {
        log.push(b'x').expect("log room");
    }
    for byte in b"THE LAST LOG BYTES" {
        log.push(*byte).expect("log tail room");
    }
    let mut d = gather(red_checks(), log.as_slice(), SHIPPED_CI);
    let cut = outputs(&mut d, Event::CutBrief { section: SECTION, bytes: 512 });
    let [Request::BriefSized { size: Ok(size), .. }] = cut.as_ref() else { panic!("successful cut: {cut:?}") };
    assert!(*size <= 512);
    // Cutting again must use the same structured check, not a previous rendering.
    let cut = outputs(&mut d, Event::CutBrief { section: SECTION, bytes: 480 });
    let [Request::BriefSized { size: Ok(size), .. }] = cut.as_ref() else { panic!("successful cut: {cut:?}") };
    assert!(*size <= 480);
    let bytes = take(&mut d);
    let words = text(&bytes);
    assert!(words.contains("Failed check: / changes (pull_request)\nDescription: Failing after 1s\nLink: /playground/widget/actions/runs/4/jobs/1\nFailed step: 2 Each pull request adds changes/<its number>.md\nEnd of its log:\n[log bytes cut]\n"), "{words}");
    assert!(words.contains("THE LAST LOG BYTES\nRepair the failed check.\n"), "{words}");
    assert_eq!(words.matches("[log bytes cut]\n").count(), 1);
    assert!(!words.contains("omitted"));
}

#[test]
fn checks_past_the_budget_are_left_out_whole_and_counted() {
    let mut checks = skein_lib::List::with_capacity(8);
    for name in [b"check-0", b"check-1", b"check-2", b"check-3", b"check-4", b"check-5", b"check-6", b"check-7"] {
        checks
            .push(Status {
                at: env().now,
                context: name.as_slice().into(),
                check: Check::Failed,
                description: b"failed".as_slice().into(),
                url: b"/job".as_slice().into(),
                job: None,
            })
            .expect("eight checks");
    }
    let expected = b"Repair the failed check at change head c1fb3599fa12c976. Finish with the change: it becomes the pull request's new head.\nFailed check: check-0\nDescription: failed\nLink: /job\n[no pinned job log for this head]\nFailed check: check-1\nDescription: failed\nLink: /job\n[no pinned job log for this head]\n[6 failed checks omitted]\nRepair the failed check.\n";
    let budget = u32::try_from(expected.len()).expect("small budget");
    let checks = checks.into_boxed();
    let mut d = gather(checks.clone(), b"", SHIPPED_CI);
    outputs(&mut d, Event::CutBrief { section: SECTION, bytes: budget });
    assert_eq!(take(&mut d).as_ref(), expected);
    let mut short = gather(checks.clone(), b"", SHIPPED_CI);
    outputs(&mut short, Event::CutBrief { section: SECTION, bytes: budget.saturating_sub(1) });
    let bytes = take(&mut short);
    let words = text(&bytes);
    assert!(words.contains("Failed check: check-0\n"));
    assert!(!words.contains("Failed check: check-1\n"));
    assert!(words.contains("[7 failed checks omitted]\n"));
    let mut tiny = gather(checks.clone(), b"", SHIPPED_CI);
    let notice = b"[8 failed checks omitted]\n";
    outputs(&mut tiny, Event::CutBrief { section: SECTION, bytes: u32::try_from(notice.len()).expect("notice size") });
    assert_eq!(take(&mut tiny).as_ref(), notice);
    // The gather's initial allocation follows the same rule as a parent's cut.
    assert_eq!(take(&mut gather(checks, b"", budget)).as_ref(), expected);
}

#[test]
fn a_log_cut_keeps_utf8_and_marks_the_missing_prefix() {
    let mut ci = brief_ci::Section::new(BriefCommit(HEAD), 1, SHIPPED_CI);
    ci.push(
        b"Failed check: unicode\nEnd of its log:\n".as_slice().into(),
        "αβγδεζηθικλμνξοπρστυφχψω".as_bytes().into(),
        false,
    );
    // Both sides of every UTF-8 boundary, including budgets too small for a head.
    for budget in 0..256 {
        if let Some(bytes) = ci.render(budget) {
            assert!(bytes.len() <= usize::try_from(budget).expect("small budget"));
            let words = text(&bytes);
            if let Some((_, log)) = words.split_once("End of its log:\n") {
                let log = log.strip_suffix("\nRepair the failed check.\n").expect("log ending");
                if let Some(tail) = log.strip_prefix("[log bytes cut]\n") {
                    assert!("αβγδεζηθικλμνξοπρστυφχψω".ends_with(tail));
                } else {
                    assert_eq!(log, "αβγδεζηθικλμνξοπρστυφχψω");
                }
            }
        }
    }
}

#[test]
fn large_check_heads_take_room_from_logs_before_a_check_is_omitted() {
    let mut checks = skein_lib::List::with_capacity(3);
    for context in [b"one", b"two", b"tri"] {
        checks
            .push(Status {
                at: env().now,
                context: context.as_slice().into(),
                check: Check::Failed,
                description: Box::new([b'd'; 130]),
                url: b"/job".as_slice().into(),
                job: Some(JobAttempt { head: HEAD, run: 4, job: 6, attempt: 1 }),
            })
            .expect("three checks");
    }
    let bytes = take(&mut gather(checks.into_boxed(), &[b'x'; 8192], 1024));
    let words = text(&bytes);
    assert_eq!(words.matches("Failed check:").count(), 3);
    assert_eq!(words.matches("Failed step: 2 Each pull request adds changes/<its number>.md\n").count(), 3);
    assert!(!words.contains("omitted"), "all whole heads fit: {words}");
    assert!(words.contains("[log bytes cut]\n"), "only the log tails are reduced: {words}");
}

#[expect(clippy::disallowed_methods, reason = "the test reader validates the rendered section's UTF-8")]
fn text(bytes: &[u8]) -> &str {
    core::str::from_utf8(bytes).expect("UTF-8 CI")
}

fn planned_read(d: &mut Domain, source: BriefSource, budget: u32) -> Token {
    planned_read_partner(d, source, budget, None)
}

fn planned_read_partner(d: &mut Domain, source: BriefSource, budget: u32, partner: Option<Token>) -> Token {
    outputs(d, Event::PlanBrief { section: SECTION, source, partner });
    let mut env = env();
    env.limits.brief_bytes = 4608;
    let mut out = Queue::uncapped();
    step(
        d,
        &env,
        Event::GatherPlanned {
            task: 9,
            section: SECTION,
            parts: 2,
            bytes: budget,
            ci_budget: SHIPPED_CI,
            deadline: env.now.saturating_add(Duration::from_secs(30)),
        },
        &mut out,
    );
    let Some(Request::BriefClient { event: skein_forge_client::Event::Read { owner, .. } }) = out.pop() else {
        panic!("one planned forge read: {out:?}")
    };
    owner
}

fn pending_pull_item(body: &[u8], state: change::State, head: [u8; 32]) -> (Domain, Token) {
    pending_pull_item_partner(body, state, head, None)
}

fn pending_pull_item_partner(
    body: &[u8],
    state: change::State,
    head: [u8; 32],
    partner: Option<Token>,
) -> (Domain, Token) {
    let mut config = configured()[0].clone();
    config.git = b"https://forge.example".as_slice().into();
    let mut d = setup_domain(&[config], false);
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    let mut row = fixtures::opening_row();
    row.pull = Some(1);
    row.change.last_head = Some(head);
    row.change.clean = Box::new([head]);
    row.change.state = state;
    outputs(&mut d, Event::Change { row });
    if let Some(section) = partner {
        outputs(
            &mut d,
            Event::PlanBrief {
                section,
                source: BriefSource::Ci {
                    item: BriefItem { repository: REPO.repository, number: 1 },
                    head: BriefCommit(head),
                },
                partner: None,
            },
        );
    }
    let owner = planned_read_partner(
        &mut d,
        BriefSource::Pull { item: BriefItem { repository: REPO.repository, number: 1 }, head: BriefCommit(head) },
        4608,
        partner,
    );
    let item = skein_forge::api::Summary {
        labels: Box::default(),
        kind: skein_forge::api::Kind::Pull,
        author: 7,
        key: None,
        updated: env().now,
        number: 1,
        title: b"Small fix".as_slice().into(),
        body: body.into(),
        state: skein_forge::api::State::Open,
    };
    let mut out = Queue::uncapped();
    assert!(brief::brief_answer(
        &mut d,
        owner,
        Ok(Answer::Item { item, comments: Box::default(), more: false }),
        LIMITS.client.rows,
        &mut out
    ));
    (d, owner)
}

fn pending_pull(body: &[u8], state: change::State) -> (Domain, Token) {
    pending_pull_partner(body, state, None)
}

fn pending_pull_partner(body: &[u8], state: change::State, partner: Option<Token>) -> (Domain, Token) {
    let (mut d, owner) = pending_pull_item_partner(body, state, HEAD, partner);
    let mut out = Queue::uncapped();
    let mut pull = fixtures::open_pull();
    pull.commit = HEAD;
    pull.base_commit = None;
    assert!(brief::brief_answer(&mut d, owner, Ok(Answer::Pull(pull)), LIMITS.client.rows, &mut out));
    (d, owner)
}

#[test]
fn stale_files_pause_a_pull_section_and_wake_its_change_even_with_a_partner() {
    const PARTNER: Token = Token::new(100);
    for (state, partner) in [
        (change::State::Resolving { requested: true, base: fixtures::BASE }, None),
        (change::State::Repairing { requested: true, why: change::Repair::Ci }, Some(PARTNER)),
    ] {
        let (mut d, owner) = pending_pull_partner(b"Whole body", state, partner);
        outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
        let mut out = Queue::uncapped();
        assert!(brief::brief_answer(&mut d, owner, Err(skein_forge::api::Error::Stale), LIMITS.client.rows, &mut out));
        assert_eq!(
            out.pop(),
            Some(Request::BriefReadRefused {
                section: SECTION,
                stage: "files",
                reason: "the change's head moved",
                heads: None,
                error: Some(skein_forge::api::Error::Stale),
            })
        );
        assert_eq!(
            out.pop(),
            Some(Request::BriefSized {
                section: SECTION,
                size: Err(BriefUnread {
                    fate: BriefFate::Capacity,
                    words: Box::from(b"the change's head moved".as_slice())
                }),
            })
        );
        assert!(out.is_empty());
        assert!(d.take_brief(SECTION).is_none(), "capacity never becomes words");
        outputs(&mut d, Event::ReadAfresh);
        outputs(&mut d, Event::SettleOutbox);
        assert!(ready(&mut d).contains(&Request::WakeProcedure { task: fixtures::CHANGE }));
    }
}

#[test]
fn stale_at_other_pull_stages_stays_permanent() {
    for compared in [false, true] {
        let state = change::State::Resolving { requested: true, base: fixtures::BASE };
        let (mut d, owner) =
            if compared { pending_pull(b"Whole body", state) } else { pending_pull_item(b"Whole body", state, HEAD) };
        let mut out = Queue::uncapped();
        if compared {
            assert!(brief::brief_answer(
                &mut d,
                owner,
                Ok(Answer::PullFiles { head: HEAD, files: Box::new([]), more: false, cut: false }),
                LIMITS.client.rows,
                &mut out,
            ));
            out.clear();
        }
        assert!(brief::brief_answer(&mut d, owner, Err(skein_forge::api::Error::Stale), LIMITS.client.rows, &mut out));
        assert!(out.pop().is_some());
        assert_eq!(
            out.pop(),
            Some(Request::BriefSized {
                section: SECTION,
                size: Err(BriefUnread {
                    fate: BriefFate::Permanent,
                    words: Box::from(b"the forge refused this read".as_slice())
                }),
            })
        );
        assert!(out.is_empty());
        assert!(d.procedure_wakes.is_empty());
    }
}

#[test]
fn a_pull_section_reads_its_own_tasks_moved_head() {
    let pinned = [1; 32];
    let head = [99; 32];
    for (writer, state, before, after) in [
        (
            Writer::Run { task: 9, attempt: 3 },
            change::State::Gating { head: pinned, asked: Box::new([]) },
            fixtures::BASE,
            head,
        ),
        (
            Writer::Lost { task: 9, attempt: 1 },
            change::State::Resolving { requested: true, base: fixtures::BASE },
            head,
            fixtures::BASE,
        ),
        (
            Writer::Run { task: 9, attempt: 2 },
            change::State::Repairing { requested: true, why: change::Repair::Semantic },
            pinned,
            head,
        ),
    ] {
        let (mut d, owner) = pending_pull_item(b"Whole body", state, pinned);
        outputs(
            &mut d,
            Event::Restore {
                record: Stored::Hold(Hold {
                    name: run_branch(b"51"),
                    task: fixtures::CHANGE,
                    writer: Some(writer),
                    design_writers: Box::new([]),
                    drift: None,
                }),
            },
        );
        let mut pull = fixtures::open_pull();
        pull.commit = head;
        let mut out = Queue::uncapped();
        assert!(brief::brief_answer(&mut d, owner, Ok(Answer::Pull(pull)), LIMITS.client.rows, &mut out));
        assert_eq!(
            out.pop(),
            Some(Request::BriefClient {
                event: skein_forge_client::Event::Read {
                    owner,
                    repository: REPO,
                    read: Read::PullFiles { number: 1, head, page: 1, skip: 0, sides: true },
                }
            }),
            "{writer:?}"
        );
        assert!(out.is_empty());
        assert!(brief::brief_answer(
            &mut d,
            owner,
            Ok(Answer::PullFiles { head, files: Box::new([]), more: false, cut: false }),
            LIMITS.client.rows,
            &mut out
        ));
        assert_eq!(
            out.pop(),
            Some(Request::BriefClient {
                event: skein_forge_client::Event::Read {
                    owner,
                    repository: REPO,
                    read: Read::Compare { before, after },
                }
            })
        );
        assert!(brief::brief_answer(
            &mut d,
            owner,
            Ok(Answer::Compare { before, after, files: Box::new([]), ahead: 0, behind: 0, changed: 0 }),
            LIMITS.client.rows,
            &mut out
        ));
        let bytes = take(&mut d);
        let words = text(&bytes);
        let notice = "[the pull's head is 636363636363, past the change's recorded 010101010101: an earlier attempt of this task pushed it, and this section reads at 636363636363]\n";
        assert!(words.contains(notice), "{words}");
        assert!(
            words.find(notice).expect("notice") < words.find("Title:").expect("content"),
            "the notice follows the identity, before content: {words}"
        );
        assert_eq!(d.hold(&run_branch(b"51")).expect("hold").writer, Some(writer));
        assert_eq!(d.change(fixtures::CHANGE).expect("change").change.last_head, Some(pinned));
    }
}

#[test]
fn another_writers_pull_head_pauses_the_brief_and_wakes_the_change() {
    for (writer, state, fate) in [
        (None, change::State::Resolving { requested: true, base: fixtures::BASE }, BriefFate::Transient),
        (
            Some(Writer::Run { task: 8, attempt: 2 }),
            change::State::Resolving { requested: true, base: fixtures::BASE },
            BriefFate::Transient,
        ),
        (
            Some(Writer::Lost { task: 8, attempt: 1 }),
            change::State::Resolving { requested: true, base: fixtures::BASE },
            BriefFate::Transient,
        ),
        (
            Some(Writer::Entry(17)),
            change::State::Resolving { requested: true, base: fixtures::BASE },
            BriefFate::Transient,
        ),
        (None, change::State::Gating { head: [1; 32], asked: Box::new([]) }, BriefFate::Capacity),
        (
            Some(Writer::Run { task: 8, attempt: 2 }),
            change::State::Repairing { requested: true, why: change::Repair::Ci },
            BriefFate::Capacity,
        ),
    ] {
        let (mut d, owner) = pending_pull_item(b"Whole body", state, [1; 32]);
        outputs(
            &mut d,
            Event::Restore {
                record: Stored::Hold(Hold {
                    name: run_branch(b"51"),
                    task: fixtures::CHANGE,
                    writer,
                    design_writers: Box::new([]),
                    drift: None,
                }),
            },
        );
        outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
        let mut pull = fixtures::open_pull();
        pull.commit = [99; 32];
        let mut out = Queue::uncapped();
        assert!(brief::brief_answer(&mut d, owner, Ok(Answer::Pull(pull)), LIMITS.client.rows, &mut out));
        assert_eq!(
            out.pop(),
            Some(Request::BriefReadRefused {
                section: SECTION,
                stage: "pull",
                reason: "the change's head moved",
                heads: Some(([1; 32], [99; 32])),
                error: None,
            })
        );
        assert_eq!(
            out.pop(),
            Some(Request::BriefSized {
                section: SECTION,
                size: Err(BriefUnread { fate, words: Box::from(b"the change's head moved".as_slice()) })
            })
        );
        assert!(out.is_empty());
        outputs(&mut d, Event::ReadAfresh);
        outputs(&mut d, Event::SettleOutbox);
        let resumed = ready(&mut d);
        assert!(resumed.contains(&Request::WakeProcedure { task: fixtures::CHANGE }), "{writer:?}: {resumed:?}");
    }
}

#[test]
fn a_review_brief_reads_on_at_the_cut_page_and_skip() {
    let (mut d, owner) = pending_pull(b"Whole body", change::State::Gating { head: HEAD, asked: Box::new([]) });
    for (path, cut, page, skip) in [(b"a", true, 1, 1), (b"b", true, 1, 2), (b"c", false, 2, 0)] {
        let mut out = Queue::uncapped();
        assert!(brief::brief_answer(
            &mut d,
            owner,
            Ok(Answer::PullFiles {
                head: HEAD,
                files: Box::new([skein_forge::api::File {
                    path: Box::from(path.as_slice()),
                    before: None,
                    after: None
                }]),
                more: true,
                cut,
            }),
            LIMITS.client.rows,
            &mut out
        ));
        assert_eq!(
            out.pop(),
            Some(Request::BriefClient {
                event: skein_forge_client::Event::Read {
                    owner,
                    repository: REPO,
                    read: Read::PullFiles { number: 1, head: HEAD, page, skip, sides: true },
                }
            })
        );
    }
    let mut out = Queue::uncapped();
    assert!(brief::brief_answer(
        &mut d,
        owner,
        Ok(Answer::PullFiles { head: HEAD, files: Box::new([]), more: false, cut: false }),
        LIMITS.client.rows,
        &mut out
    ));
    let text = take(&mut d);
    for path in [b"File: a", b"File: b", b"File: c"] {
        assert!(skein_lib::bytes::find(&text, path).is_some());
    }
}

#[test]
fn a_late_brief_answer_for_completed_work_is_dropped() {
    for compared in [false, true] {
        let (mut d, owner) =
            pending_pull(b"Whole body", change::State::Resolving { requested: true, base: fixtures::BASE });
        let files = Answer::PullFiles { head: HEAD, files: Box::new([]), more: false, cut: false };
        if compared {
            assert!(brief::brief_answer(&mut d, owner, Ok(files.clone()), LIMITS.client.rows, &mut Queue::uncapped()));
        }
        let row = d.changes.get_mut(&fixtures::CHANGE).expect("live change");
        row.change.state = change::State::Landed { since: env().wall, merge: fixtures::BASE };
        let answer = if compared {
            Answer::Compare {
                before: HEAD,
                after: fixtures::BASE,
                files: Box::new([]),
                ahead: 0,
                behind: 1,
                changed: 0,
            }
        } else {
            files
        };
        let mut out = Queue::uncapped();
        assert!(brief::brief_answer(&mut d, owner, Ok(answer), LIMITS.client.rows, &mut out));
        assert!(out.is_empty(), "a completed change drops the late answer: {out:?}");
        assert!(d.brief_fetches.is_empty());
        assert!(d.brief_pending.is_empty());
        assert!(d.take_brief(SECTION).is_none());
    }
}

#[test]
fn a_planned_brief_does_not_start_after_its_change_completes() {
    let item = BriefItem { repository: REPO.repository, number: 1 };
    for source in [
        BriefSource::Pull { item, head: BriefCommit(HEAD) },
        BriefSource::Ci { item, head: BriefCommit(HEAD) },
        BriefSource::Reviews { item, head: BriefCommit(HEAD) },
        request_source(),
    ] {
        let mut d = request_domain();
        outputs(&mut d, Event::PlanBrief { section: SECTION, source, partner: None });
        d.changes.get_mut(&fixtures::CHANGE).expect("live change").change.state = change::State::Ended;
        let out = outputs(
            &mut d,
            Event::GatherPlanned {
                task: 9,
                section: SECTION,
                parts: 2,
                bytes: 2048,
                ci_budget: 2048,
                deadline: env().now.saturating_add(Duration::from_secs(30)),
            },
        );
        assert!(out.is_empty(), "completed work starts no brief read: {out:?}");
        assert!(d.brief_pending.is_empty());
        assert!(d.brief_fetches.is_empty());
    }
}

#[test]
fn a_completed_change_does_not_retry_a_busy_brief() {
    let (mut d, owner) =
        pending_pull(b"Whole body", change::State::Resolving { requested: true, base: fixtures::BASE });
    let mut out = Queue::uncapped();
    assert!(brief::brief_answer(&mut d, owner, Err(skein_forge::api::Error::Busy), LIMITS.client.rows, &mut out));
    d.changes.get_mut(&fixtures::CHANGE).expect("live change").change.state = change::State::Ended;
    d.reclaim();
    brief::retry(&mut d, &mut out);
    assert!(out.is_empty(), "a completed change never retries admission: {out:?}");
    assert!(d.brief_fetches.is_empty());
}

fn pull_section(body: &[u8]) -> Box<[u8]> {
    let (mut d, owner) = pending_pull(
        body,
        temper_engine_domain_forge_change::State::Repairing {
            requested: true,
            why: temper_engine_domain_forge_change::Repair::Ci,
        },
    );
    let mut out = Queue::uncapped();
    assert!(brief::brief_answer(
        &mut d,
        owner,
        Ok(Answer::PullFiles { head: HEAD, files: Box::default(), more: false, cut: false }),
        LIMITS.client.rows,
        &mut out
    ));
    take(&mut d)
}

fn resolution_section(files: Box<[skein_forge::api::File]>) -> Box<[u8]> {
    let (mut d, owner) = pending_pull(
        b"Whole body",
        temper_engine_domain_forge_change::State::Resolving { requested: true, base: fixtures::BASE },
    );
    let mut out = Queue::uncapped();
    assert!(brief::brief_answer(
        &mut d,
        owner,
        Ok(Answer::PullFiles { head: HEAD, files, more: false, cut: false }),
        LIMITS.client.rows,
        &mut out
    ));
    assert_eq!(
        out.pop(),
        Some(Request::BriefClient {
            event: skein_forge_client::Event::Read {
                owner,
                repository: REPO,
                read: Read::Compare { before: HEAD, after: fixtures::BASE },
            },
        })
    );
    assert!(brief::brief_answer(
        &mut d,
        owner,
        Ok(Answer::Compare {
            before: HEAD,
            after: fixtures::BASE,
            files: Box::default(),
            ahead: 0,
            behind: 1,
            changed: 0
        }),
        LIMITS.client.rows,
        &mut out
    ));
    take(&mut d)
}

#[test]
#[expect(clippy::disallowed_macros, reason = "the table formats literal expected file contents")]
fn a_resolutions_files_are_carried_whole_or_named() {
    use skein_forge::api::File;
    type Row = (&'static str, Box<[File]>, Box<str>);
    let a =
        File { path: b"a".as_slice().into(), before: Some(Box::new([b'x'; 100])), after: Some(Box::new([b'y'; 120])) };
    let b = File { path: b"b".as_slice().into(), before: None, after: Some(Box::new([b'z'; 50])) };
    let codec = File {
        path: b"codec".as_slice().into(),
        before: Some(Box::new([b'p'; 3000])),
        after: Some(Box::new([b'q'; 3000])),
    };
    let whole_a = format!("File: a\nBefore:\n{}\nAfter:\n{}\n", "x".repeat(100), "y".repeat(120));
    let whole_b = format!("File: b\nBefore:\n\nAfter:\n{}\n", "z".repeat(50));
    let named = "File: codec\n[not carried, past the brief's room: 3000 bytes before, 3000 bytes after; read it in your checkout]\n";
    let rows: [Row; 4] = [
        ("small_files_whole", Box::new([a.clone(), b]), format!("{whole_a}{whole_b}").into()),
        ("a_large_file_named", Box::new([codec.clone()]), named.into()),
        ("named_then_whole", Box::new([codec, a]), format!("{named}{whole_a}").into()),
        ("an_added_large_file", Box::new([File { path: b"codec".as_slice().into(), before: None, after: Some(Box::new([b'q'; 6000])) }]),
            "File: codec\n[not carried, past the brief's room: absent before, 6000 bytes after; read it in your checkout]\n".into()),
    ];
    let mut failed = skein_lib::List::with_capacity(4);
    for (name, files, expected) in rows {
        let bytes = resolution_section(files);
        let section = text(&bytes).split_once("Files and diff:\n").expect("files heading").1;
        let contents = match section.split_once("What landed in the base: 0 commits\n") {
            Some((files, _)) => Some(files),
            None => None,
        };
        if contents != Some(expected.as_ref()) {
            failed.push(name).expect("at most one failure per row");
        }
    }
    assert!(failed.is_empty(), "files must be whole or named; failed rows: {:?}", failed.as_slice());
}

#[test]
fn a_repairs_pull_section_names_its_pull_request_before_its_contents() {
    let bytes = pull_section(b"Whole body");
    let words = text(&bytes);
    assert!(words.starts_with("Pull request: #1\nBranch: temper/51\nBase: main\nHead: c1fb3599fa12c9766f9cc162844416e87a929182\nAddress: https://forge.example/org/repo/pulls/1\n"), "{words}");
    assert!(words.contains("Title: Small fix\nBody: Whole body\n"));
}

fn remarks_section(remarks: &[u8]) -> Box<[u8]> {
    let mut d = domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    let mut row = fixtures::opening_row();
    row.pull = Some(1);
    row.change.last_head = Some(HEAD);
    row.gate_remarks = Box::new([GateRemark { number: 1, head: HEAD, words: remarks.into() }]);
    outputs(&mut d, Event::Change { row });
    let owner = planned_read(
        &mut d,
        BriefSource::Reviews { item: BriefItem { repository: REPO.repository, number: 1 }, head: BriefCommit(HEAD) },
        4608,
    );
    assert!(brief::brief_answer(
        &mut d,
        owner,
        Ok(Answer::Reviews { reviews: Box::default(), more: false, cut: false }),
        LIMITS.client.rows,
        &mut Queue::uncapped()
    ));
    take(&mut d)
}

const REVIEWED: [u8; 32] = [7; 32];

fn request_source() -> BriefSource {
    BriefSource::Request {
        item: BriefItem { repository: REPO.repository, number: 1 },
        head: BriefCommit(HEAD),
        reviewed: BriefCommit(REVIEWED),
    }
}

fn request_domain() -> Domain {
    let mut d = domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    let mut row = fixtures::opening_row();
    row.pull = Some(1);
    row.change.last_head = Some(HEAD);
    row.gate_remarks = Box::new([GateRemark { number: 1, head: HEAD, words: b"not this gate".as_slice().into() }]);
    outputs(&mut d, Event::Change { row });
    d
}

fn request_review(id: u64, commit: [u8; 32], body: &[u8]) -> skein_forge::api::Review {
    skein_forge::api::Review {
        id,
        provenance: skein_forge::api::Provenance::Original,
        revision: 1,
        author: 42,
        verdict: skein_forge::api::Verdict::RequestChanges,
        commit,
        key: None,
        body: body.into(),
        at: env().now,
        official: false,
    }
}

#[test]
fn a_requests_section_quotes_only_the_reviewed_heads_body_and_line_remark() {
    let mut d = request_domain();
    let owner = planned_read(&mut d, request_source(), 4608);
    let mut out = Queue::uncapped();
    assert!(brief::brief_answer(
        &mut d,
        owner,
        Ok(Answer::Reviews {
            reviews: Box::new([
                request_review(11, REVIEWED, b"Please retain the blank line."),
                request_review(12, HEAD, b"Other head's review"),
            ]),
            more: false,
            cut: false
        }),
        LIMITS.client.rows,
        &mut out
    ));
    let Some(Request::BriefClient {
        event: skein_forge_client::Event::Read { owner: call, read: Read::Remarks { review: 11, .. }, .. },
    }) = out.pop()
    else {
        panic!("read the kept review's remarks: {out:?}")
    };
    assert_eq!(call, owner);
    assert!(out.is_empty());
    assert!(brief::brief_answer(
        &mut d,
        owner,
        Ok(Answer::Remarks {
            remarks: Box::new([skein_forge::api::Remark {
                id: 21,
                author: 42,
                path: b"src/lib.rs".as_slice().into(),
                line: 9,
                body: b"Keep both appended lines.".as_slice().into(),
            }]),
            more: false,
            cut: false
        }),
        LIMITS.client.rows,
        &mut out
    ));
    let bytes = take(&mut d);
    assert_eq!(
        bytes.as_ref(),
        b"Review: Please retain the blank line.\nRemark at src/lib.rs:9 Keep both appended lines.\n"
    );
}

#[test]
fn a_request_without_remarks_ends_with_an_explicit_marker() {
    for reviews in [
        [].as_slice(),
        [request_review(12, HEAD, b"Other head's review")].as_slice(),
        [request_review(11, REVIEWED, b"")].as_slice(),
    ] {
        let mut d = request_domain();
        let owner = planned_read(&mut d, request_source(), 4608);
        let mut out = Queue::uncapped();
        assert!(brief::brief_answer(
            &mut d,
            owner,
            Ok(Answer::Reviews { reviews: reviews.into(), more: false, cut: false }),
            LIMITS.client.rows,
            &mut out
        ));
        if let Some(Request::BriefClient {
            event: skein_forge_client::Event::Read { read: Read::Remarks { review: 11, .. }, .. },
        }) = out.pop()
        {
            assert!(brief::brief_answer(
                &mut d,
                owner,
                Ok(Answer::Remarks { remarks: Box::new([]), more: false, cut: false }),
                LIMITS.client.rows,
                &mut out
            ));
        }
        let bytes = take(&mut d);
        assert!(bytes.ends_with(b"[the request for changes gave no remarks]\n"), "{}", text(&bytes));
        assert!(!text(&bytes).contains("Gate remarks:"));
        assert!(bytes.len() <= 4608);
    }
}

#[test]
fn a_requests_section_refuses_a_moved_change_head() {
    let mut d = request_domain();
    let source = BriefSource::Request {
        item: BriefItem { repository: REPO.repository, number: 1 },
        head: BriefCommit([8; 32]),
        reviewed: BriefCommit(REVIEWED),
    };
    outputs(&mut d, Event::PlanBrief { section: SECTION, source, partner: None });
    let out = outputs(
        &mut d,
        Event::GatherPlanned {
            task: 9,
            section: SECTION,
            parts: 2,
            bytes: 512,
            ci_budget: SHIPPED_CI,
            deadline: env().now.saturating_add(Duration::from_secs(30)),
        },
    );
    let [
        Request::BriefReadRefused {
            section: SECTION,
            stage: "reviews",
            reason: "the change's head moved",
            heads: Some((pinned, head)),
            error: None,
        },
        Request::BriefSized { section: SECTION, size: Err(why) },
    ] = out.as_ref()
    else {
        panic!("moved: {out:?}")
    };
    assert_eq!(*pinned, [8; 32]);
    assert_eq!(*head, HEAD);
    assert_eq!(d.procedure_wakes.get(&fixtures::CHANGE), Some(&true));
    assert_eq!(why.fate, BriefFate::Capacity);
    assert_eq!(why.words.as_ref(), b"the change's head moved");
}

#[test]
fn four_thousand_bytes_of_gate_remarks_reach_the_repair_whole() {
    let remarks = [b'r'; 4000];
    let bytes = remarks_section(&remarks);
    let body = bytes.strip_prefix(b"Gate remarks: ").expect("remarks heading");
    assert_eq!(body.strip_suffix(b"\n"), Some(remarks.as_slice()));
}

#[test]
fn five_thousand_bytes_of_gate_remarks_keep_their_prefix_and_mark_the_cut() {
    let remarks = [b'r'; 5000];
    let bytes = remarks_section(&remarks);
    let words = text(&bytes);
    let (kept, marker) = words.rsplit_once("\n[").expect("marked cut");
    assert!(remarks.starts_with(kept.strip_prefix("Gate remarks: ").expect("remarks heading").as_bytes()));
    let count = skein_lib::Decimal::of(
        u64::try_from(b"Gate remarks: ".len() + remarks.len() + 1 - kept.len()).expect("cut bytes"),
    );
    assert_eq!(marker.strip_suffix(" bytes cut]\n").expect("cut marker").as_bytes(), count.as_bytes());
    assert!(bytes.len() <= 4608);
}

#[test]
fn a_maximal_task_pull_and_remarks_fit_the_assembled_brief() {
    let task = [b't'; 4608];
    let pull = pull_section(&[b'p'; 5000]);
    let remarks = remarks_section(&[b'r'; 5000]);
    assert!(text(&pull).contains("Title: Small fix\nBody: "));
    assert!(pull.ends_with(b" bytes cut]\n"));
    assert!(remarks.ends_with(b" bytes cut]\n"));
    assert!(task.len() + pull.len() + remarks.len() + 768 <= 14_592);
}

fn pending_checks() -> (Domain, Token) {
    let mut d = domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    let mut row = fixtures::opening_row();
    row.pull = Some(1);
    row.change.last_head = Some(HEAD);
    row.change.state = temper_engine_domain_forge_change::State::Repairing {
        requested: true,
        why: temper_engine_domain_forge_change::Repair::Ci,
    };
    outputs(&mut d, Event::Change { row });
    let owner = planned_read(
        &mut d,
        BriefSource::Ci { item: BriefItem { repository: REPO.repository, number: 1 }, head: BriefCommit(HEAD) },
        SHIPPED_CI,
    );
    (d, owner)
}

fn pending_job() -> (Domain, Token, Read) {
    let (mut d, owner) = pending_checks();
    let mut out = Queue::uncapped();
    assert!(brief::brief_answer(&mut d, owner, Ok(Answer::Checks(red_checks())), LIMITS.client.rows, &mut out));
    let Some(Request::BriefClient { event: skein_forge_client::Event::Read { read, .. } }) = out.pop() else {
        panic!("one pinned job read: {out:?}")
    };
    assert!(out.is_empty());
    let Read::Run { .. } = read else { panic!("classify the failed job's run first") };
    assert!(brief::brief_answer(&mut d, owner, Err(skein_forge::api::Error::Missing), LIMITS.client.rows, &mut out));
    let Some(Request::BriefClient { event: skein_forge_client::Event::Read { read, .. } }) = out.pop() else {
        panic!("one pinned job read after classification: {out:?}")
    };
    assert!(out.is_empty());
    (d, owner, read)
}

#[test]
fn a_busy_job_read_waits_for_reclaim_then_keeps_the_step_and_log_end() {
    let (mut d, owner, read) = pending_job();
    let mut out = Queue::uncapped();
    assert!(brief::brief_answer(&mut d, owner, Err(skein_forge::api::Error::Busy), LIMITS.client.rows, &mut out));
    assert!(out.is_empty(), "a refusal cannot retry in the same drain");
    assert!(!brief::waiting(&d));
    brief::retry(&mut d, &mut out);
    assert!(out.is_empty(), "even another drain must wait for reclaim");
    d.reclaim();
    assert!(brief::waiting(&d));
    brief::retry(&mut d, &mut out);
    assert_eq!(
        out.pop(),
        Some(Request::BriefClient {
            event: skein_forge_client::Event::Read { owner, repository: REPO, read: read.clone() },
        })
    );
    let Read::Job { attempt, .. } = read else { panic!("pinned job") };
    assert!(brief::brief_answer(
        &mut d,
        owner,
        Ok(Answer::Job {
            attempt,
            step: Some(Step { number: 2, name: STEP.into() }),
            log: RUNNER_LOG.into(),
            truncated: false,
        }),
        LIMITS.client.rows,
        &mut out
    ));
    let Some(Request::BriefSized { section: SECTION, size: Ok(size) }) = out.pop() else {
        panic!("completed CI section: {out:?}")
    };
    assert!(size <= SHIPPED_CI);
    let bytes = take(&mut d);
    let words = text(&bytes);
    assert_eq!(words.matches("Failed check:").count(), 1, "retry does not repeat the check head");
    assert!(words.contains("Failed step: 2 Each pull request adds changes/<its number>.md\nEnd of its log:\n"));
    assert!(words.contains(text(RUNNER_LOG)), "the complete log, including its end: {words}");
    assert!(words.ends_with("Repair the failed check.\n"));
    assert!(!words.contains("could not be read"));
    assert!(d.briefs_idle());
}

#[test]
fn a_job_read_busy_until_the_gather_deadline_keeps_the_marker() {
    let (mut d, owner, read) = pending_job();
    let deadline = env().now.saturating_add(Duration::from_secs(30));
    let mut out = Queue::uncapped();
    for _ in 0_u32..3 {
        assert!(brief::brief_answer(&mut d, owner, Err(skein_forge::api::Error::Busy), LIMITS.client.rows, &mut out));
        assert!(out.is_empty());
        d.reclaim();
        brief::retry(&mut d, &mut out);
        assert_eq!(
            out.pop(),
            Some(Request::BriefClient {
                event: skein_forge_client::Event::Read { owner, repository: REPO, read: read.clone() },
            })
        );
    }
    assert!(brief::brief_answer(&mut d, owner, Err(skein_forge::api::Error::Busy), LIMITS.client.rows, &mut out));
    brief::expire(&mut d, Time::from_nanos(deadline.as_nanos().checked_sub(1).unwrap()), &mut out);
    assert!(out.is_empty(), "the original deadline stays in force");
    assert_eq!(brief::next_deadline(&d), Some(deadline));
    brief::expire(&mut d, deadline, &mut out);
    assert_eq!(
        out.pop(),
        Some(Request::BriefReadRefused {
            section: SECTION,
            stage: "job",
            reason: "the forge client is busy",
            heads: None,
            error: Some(skein_forge::api::Error::Busy)
        })
    );
    let Some(Request::BriefSized { section: SECTION, size: Ok(size) }) = out.pop() else {
        panic!("completed CI section: {out:?}")
    };
    assert!(size <= SHIPPED_CI);
    let bytes = take(&mut d);
    let words = text(&bytes);
    assert!(words.contains("Failed check: / changes (pull_request)"));
    assert!(words.ends_with("[job log could not be read]\nRepair the failed check.\n"), "{words}");
    d.reclaim();
    brief::retry(&mut d, &mut out);
    assert!(out.is_empty());
    assert!(d.briefs_idle());
}

#[test]
fn an_inflight_job_at_the_gather_deadline_leaves_the_section_late() {
    for retried in [false, true] {
        let (mut d, owner, read) = pending_job();
        let deadline = env().now.saturating_add(Duration::from_secs(30));
        let mut out = Queue::uncapped();
        if retried {
            assert!(brief::brief_answer(
                &mut d,
                owner,
                Err(skein_forge::api::Error::Busy),
                LIMITS.client.rows,
                &mut out
            ));
            assert_eq!(brief::next_deadline(&d), Some(deadline));
            d.reclaim();
            brief::retry(&mut d, &mut out);
            assert_eq!(
                out.pop(),
                Some(Request::BriefClient { event: skein_forge_client::Event::Read { owner, repository: REPO, read } })
            );
        }
        assert_eq!(brief::next_deadline(&d), None, "the parent owns an in-flight read's deadline");
        brief::expire(&mut d, deadline, &mut out);
        assert!(out.is_empty(), "an unanswered read cannot complete the section with a marker");
        assert!(!d.briefs_idle(), "the section stays unfinished for the parent's late handling");
        assert!(outputs(&mut d, Event::DropBrief { section: SECTION }).is_empty());
        assert!(d.briefs_idle());
    }
}

#[test]
fn checks_busy_until_the_gather_deadline_fail_the_section() {
    let (mut d, owner) = pending_checks();
    let deadline = env().now.saturating_add(Duration::from_secs(30));
    let mut out = Queue::uncapped();
    assert!(brief::brief_answer(&mut d, owner, Err(skein_forge::api::Error::Busy), LIMITS.client.rows, &mut out));
    assert_eq!(brief::next_deadline(&d), Some(deadline));
    brief::expire(&mut d, Time::from_nanos(deadline.as_nanos().checked_sub(1).unwrap()), &mut out);
    assert!(out.is_empty());
    brief::expire(&mut d, deadline, &mut out);
    assert_eq!(
        out.pop(),
        Some(Request::BriefReadRefused {
            section: SECTION,
            stage: "checks",
            reason: "the forge client is busy",
            heads: None,
            error: Some(skein_forge::api::Error::Busy)
        })
    );
    assert_eq!(out.pop(), Some(Request::BriefSized { section: SECTION, size: Err(brief_words::busy()) }));
    assert!(out.is_empty());
    assert!(d.briefs_idle());
}

#[test]
fn dropping_a_busy_section_ends_its_retry() {
    let (mut d, owner, _) = pending_job();
    let mut out = Queue::uncapped();
    assert!(brief::brief_answer(&mut d, owner, Err(skein_forge::api::Error::Busy), LIMITS.client.rows, &mut out));
    assert!(outputs(&mut d, Event::DropBrief { section: SECTION }).is_empty());
    d.reclaim();
    brief::retry(&mut d, &mut out);
    assert!(out.is_empty());
    assert!(d.briefs_idle());
}

#[test]
fn github_gate_timestamp_survives_a_truncated_log_at_shipped_limits() {
    const LINE: &[u8] = b"Missing changes/31.md with the line: opened 2026-11-01T14:07:25Z";
    let mut log = skein_lib::List::with_capacity(9000);
    for _ in 0_u32..8192 {
        log.push(b'x').expect("log room");
    }
    for byte in b"\n".iter().chain(LINE).chain(b"\njob changes failed\n") {
        log.push(*byte).expect("log tail room");
    }
    for head in [HEAD, [0x31; 32]] {
        let mut checks = red_checks();
        checks[1].context = b"changes".as_slice().into();
        checks[1].url = b"https://github.com/acme/widgets/actions/runs/22/job/31".as_slice().into();
        checks[1].job = Some(JobAttempt { head, run: 22, job: 31, attempt: 2 });
        // gather's repair head is HEAD: a pin from another head must stay unread.
        let bytes = take(&mut gather(checks, log.as_slice(), SHIPPED_CI));
        let words = text(&bytes);
        if head == HEAD {
            assert!(words.contains(text(LINE)), "{words}");
            assert!(words.contains("[log bytes cut]"));
        } else {
            assert!(words.contains("[no pinned job log for this head]"));
            assert!(!words.contains(text(LINE)));
        }
    }
}

#[test]
fn each_unread_error_has_its_count_and_fixed_words() {
    use BriefFate::{Capacity, Permanent, Transient};
    use skein_forge::api::Error;
    let rows: &[(Error, BriefFate, &[u8])] = &[
        (Error::Busy, Capacity, b"the forge client is busy"),
        (Error::NotConnected, Capacity, b"the forge is not connected"),
        (Error::Unavailable, Capacity, b"the forge could not be reached"),
        (Error::Timeout, Capacity, b"the forge did not answer in time"),
        (Error::RateLimited { after: Duration::from_secs(1) }, Transient, b"the forge is limiting calls"),
        (Error::InvalidAnswer, Permanent, b"the forge's answer did not match its read"),
        (Error::Forbidden, Permanent, b"the forge forbids this read"),
        (Error::NotInstalled, Permanent, b"the forge App is not installed"),
        (Error::Missing, Permanent, b"the forge has no such object"),
        (Error::MissingJob, Permanent, b"the forge has no such object"),
        (
            Error::TooLarge(skein_forge::api::Excess::exact(skein_forge::api::Bound::Rows, 8, 9)),
            Permanent,
            b"the answer is past forge.client.rows (8): 9 rows",
        ),
        (Error::Full, Permanent, b"the forge refused this read"),
        (Error::Stale, Permanent, b"the forge refused this read"),
        (Error::Refused, Permanent, b"the forge refused this read"),
    ];
    for (error, fate, words) in rows {
        let unread = brief_words::unread_of(&Err(*error));
        assert_eq!(unread.fate, *fate, "{error:?}");
        assert_eq!(unread.words.as_ref(), *words, "{error:?}");
    }
    let mut pull = fixtures::open_pull();
    pull.commit = [99; 32];
    assert_eq!(
        brief_words::unread_of(&Ok(Answer::Pull(pull))),
        BriefUnread { fate: Capacity, words: Box::from(b"the change's head moved".as_slice()) }
    );
    assert_eq!(
        brief_words::unread_of(&Ok(Answer::Commit(HEAD))),
        BriefUnread { fate: Permanent, words: Box::from(b"the forge answered another read".as_slice()) }
    );
}

#[test]
fn local_section_failures_name_the_bound_or_room_that_failed() {
    let item = BriefItem { repository: REPO.repository, number: 1 };
    let mut d = domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    let mut out = Queue::uncapped();
    let unnamed = BriefUnread {
        fate: BriefFate::Permanent,
        words: Box::from(b"the change cannot be named in its section".as_slice()),
    };
    assert_eq!(
        brief::brief_pull(&mut d, SECTION, item, BriefCommit(HEAD), 1, 512, &mut out),
        Some(BriefRead::Failed(unnamed.clone()))
    );
    let mut row = fixtures::opening_row();
    row.pull = Some(1);
    row.change.last_head = Some(HEAD);
    outputs(&mut d, Event::Change { row });
    assert_eq!(
        brief::brief_pull(&mut d, SECTION, item, BriefCommit([99; 32]), 1, 512, &mut out),
        Some(BriefRead::Failed(BriefUnread {
            fate: BriefFate::Capacity,
            words: Box::from(b"the change's head moved".as_slice())
        }))
    );
    assert_eq!(
        brief::brief_pull(&mut d, SECTION, item, BriefCommit(HEAD), 1, 1, &mut out),
        Some(BriefRead::Failed(unnamed.clone()))
    );
    assert_eq!(
        brief::brief_pull(&mut d, SECTION, item, BriefCommit(HEAD), 0, 512, &mut out),
        Some(BriefRead::Failed(BriefUnread {
            fate: BriefFate::Permanent,
            words: Box::from(b"the connector cannot hold this section".as_slice())
        }))
    );
    d.brief_fetches = skein_lib::Map::with_capacity(0);
    assert_eq!(
        brief::brief_pull(&mut d, SECTION, item, BriefCommit(HEAD), 1, 512, &mut out),
        Some(BriefRead::Failed(BriefUnread {
            fate: BriefFate::Capacity,
            words: Box::from(b"the connector has no room for another read".as_slice())
        }))
    );
    assert_eq!(
        out.pop(),
        Some(Request::BriefReadRefused {
            section: SECTION,
            stage: "pull",
            reason: "the change's head moved",
            heads: Some(([99; 32], HEAD)),
            error: None
        })
    );
    assert_eq!(d.procedure_wakes.get(&fixtures::CHANGE), Some(&true));
    assert!(out.is_empty(), "no local refusal makes a forge read");
    held::ready_ci(&mut d, SECTION, brief_ci::Section::new(BriefCommit(HEAD), 0, 0), 0, &mut out);
    assert_eq!(
        out.pop(),
        Some(Request::BriefReady {
            owner: SECTION,
            read: BriefRead::Failed(BriefUnread {
                fate: BriefFate::Permanent,
                words: Box::from(b"the failed checks do not fit their section".as_slice())
            })
        })
    );
}

#[test]
fn a_runners_failure_reaches_the_repair_at_shipped_limits() {
    let first = JobAttempt { head: HEAD, run: 4, job: 6, attempt: 1 };
    let mut failed = change_ci::Failed::new(Box::new([Status {
        at: Time::ZERO,
        context: b"ci/test".as_slice().into(),
        check: Check::Failed,
        description: Box::new([]),
        url: Box::new([]),
        job: Some(first),
    }]));
    assert_eq!(failed.next(HEAD), Some(Read::Run { run: 4 }));
    failed.read(4, Ok(Answer::Run(fixtures::failed_run(first, Some(0)))));
    assert_eq!(failed.fault(HEAD), change::Fault::Runner { reruns: 0 });
    assert_eq!(LIMITS.change_policy.reruns, 1);
    let second = JobAttempt { attempt: 2, ..first };
    let checks = Box::new([Status {
        at: Time::ZERO,
        context: b"ci/test".as_slice().into(),
        check: Check::Failed,
        description: Box::new([]),
        url: Box::new([]),
        job: Some(second),
    }]);
    let bytes = take(&mut gather_run(
        checks,
        b"could not prepare job",
        SHIPPED_CI,
        Some(fixtures::failed_run(second, Some(0))),
    ));
    let words = text(&bytes);
    assert!(words.contains("the runner's: set-up failed; run again 1 time at this head, failed again"), "{words}");
    assert!(words.contains("could not prepare job"));
}
