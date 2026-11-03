use crate::tests::fixtures as f;
use jig_client::{Watchable, contract as c};
use skein_lib::Time;

fn tier_run_words() -> Box<[u8]> {
    let prefix = b"Judged run: commit bc9128bd..., branch temper/tier/sl";
    let identity = b"Judged run: commit bc9128bd..., branch temper/tier/slow.1, forge run 91 http://forge/org/repo/actions/runs/91";
    let padding = 128usize.checked_sub(prefix.len()).unwrap();
    let mut words = skein_lib::Writer::new(padding.checked_add(identity.len()).unwrap());
    words.put(b"Tier slow run 1\n").unwrap();
    for _ in b"Tier slow run 1\n".len()..padding {
        words.put(b" ").unwrap();
    }
    words.put(identity).unwrap();
    words.finish()
}

fn serve_specification(root: &mut f::Harness, call: u32, words: &[u8]) {
    let (core, limits, row) = f::source_task(words);
    for _ in 0..words.len() {
        let Some((_, c::Read::TaskText { task: 1, field: c::TaskTextField::Specification, from })) =
            root.reads.get(&call)
        else {
            break;
        };
        let chunk = f::source_chunk(&core, &limits, &row, jig_domain_core::reads::TaskTextField::Specification, *from);
        root.serve(call, c::ReadResult::Value(c::ReadValue::TaskText(Box::new(chunk))));
    }
    assert!(root.reads.is_empty(), "sequential source reads finish within their byte bound");
}

fn source_wait(
    words: &[u8],
    tools: &crate::jig::Limits,
    end: Option<c::WatchEnd>,
    json: bool,
) -> (f::Harness, u32, Box<[u8]>) {
    let (core, limits, row) = f::source_task(words);
    let mut task = f::source_summary(&core, &limits, &row);
    task.chip.phase = c::Phase::Done;
    let digest = f::task_digest(&c::Delivery::Task(Box::new(task.clone()))).hex();
    let mut root = f::signed_in(f::PARTY);
    root.limits.tools = *tools;
    let argv = f::extended(
        &["task", "show", "1", "--until", "ended", "--timeout", "1s"],
        if json { &["--json"] } else { &[] },
    );
    let call = root.submit(argv.as_slice());
    let watch = Watchable::Jig(c::Subject::Task { task: 1 });
    if let Some(end) = end {
        if end == c::WatchEnd::SubjectEnded {
            let mut live = task.clone();
            live.chip.phase = c::Phase::Running;
            root.deliver_subject(&watch, c::Delivery::Task(Box::new(live)));
        }
        root.end(call, &watch, end);
    } else {
        root.serving = Some(call);
        root.deliver_subject(&watch, c::Delivery::Task(Box::new(task.clone())));
        root.serving = None;
    }
    root.serve(call, f::ended_task_result(task, c::Ending::Done));
    assert_eq!(crate::root::next_deadline(&root.root), None);
    root.now = Time::ZERO.checked_add(f::WAIT_SPAN).unwrap();
    root.fire();
    (root, call, Box::from(digest.as_slice()))
}

fn settled_source_wait(root: &mut f::Harness, call: u32) {
    assert!(root.reads.is_empty());
    assert_eq!(root.watches.len(), f::SHARED_WATCHES);
    assert_eq!(crate::root::next_deadline(&root.root), None);
    root.now = Time::ZERO.checked_add(f::WAIT_SPAN).unwrap();
    root.fire();
    assert!(root.answer(call).is_none(), "one answer despite the expired wait deadline");
}

// The client-tools root is the lowest world owning waits and their printed document.
#[test]
fn task_wait_fills_the_tier_run_identity_after_the_terminal_read() {
    let words = tier_run_words();
    let (core, limits, row) = f::source_task(&words);
    let task = f::source_summary(&core, &limits, &row);
    assert!(task.spec.specification.words.cut);
    assert!(task.spec.specification.words.bytes().ends_with(b"branch temper/tier/sl"));
    let (plain, _) = f::source_show(&words, &crate::limits::tools(), false);
    assert!(f::has(&plain, &words), "plain task show prints the whole run identity");
    for end in [Some(c::WatchEnd::SubjectEnded), Some(c::WatchEnd::Unknown), None] {
        let (mut root, call, digest) = source_wait(&words, &crate::limits::tools(), end, false);
        assert!(root.answer(call).is_none(), "source reads precede the answer");
        serve_specification(&mut root, call, &words);
        let answer = root.answer(call).expect("terminal wait document");
        assert_eq!(answer.status, 0);
        assert!(f::has(&answer, &words), "wait must retain the branch, forge run and URL");
        assert!(f::has(&answer, b"Final task report"));
        assert!(f::has(&answer, &digest));
        assert!(answer.stdout.ends_with(b"met\n"));
        settled_source_wait(&mut root, call);
    }
}

#[test]
fn task_wait_fills_a_meeting_delivery_without_changing_its_digest() {
    let words = tier_run_words();
    let (core, limits, row) = f::source_task(&words);
    let mut task = f::source_summary(&core, &limits, &row);
    task.chip.phase = c::Phase::Running;
    let delivery = c::Delivery::Task(Box::new(task));
    let digest = f::task_digest(&delivery).hex();
    let mut root = f::signed_in(f::PARTY);
    let call = root.submit(&["task", "show", "1", "--until", "phase:running", "--timeout", "1s"]);
    root.serving = Some(call);
    root.deliver_subject(&Watchable::Jig(c::Subject::Task { task: 1 }), delivery);
    root.serving = None;
    assert!(root.answer(call).is_none());
    assert_eq!(crate::root::next_deadline(&root.root), None);
    serve_specification(&mut root, call, &words);
    let answer = root.answer(call).unwrap();
    assert_eq!(answer.status, 0);
    assert!(f::has(&answer, &words));
    assert!(f::has(&answer, &digest));
    assert!(answer.stdout.ends_with(b"met\n"));
    settled_source_wait(&mut root, call);
}

#[test]
fn task_wait_json_keeps_the_projected_document_without_supplemental_reads() {
    for end in [Some(c::WatchEnd::SubjectEnded), Some(c::WatchEnd::Unknown), None] {
        let (mut root, call, digest) = source_wait(&tier_run_words(), &crate::limits::tools(), end, true);
        let answer = root.answer(call).unwrap();
        assert_eq!(answer.status, 0);
        assert!(f::has(&answer, &digest));
        assert!(f::has(&answer, b"Final task report"));
        assert!(f::has(&answer, b"branch temper/tier/sl"));
        assert!(!f::has(&answer, b"forge run 91"));
        assert!(answer.stdout.ends_with(b",\"met\":true}"));
        settled_source_wait(&mut root, call);
    }
}

fn check_source_continuation(answer: &crate::jig::Printed, words: &[u8]) -> usize {
    let start = skein_lib::bytes::find(&answer.stdout, b"asked: ").unwrap().checked_add(b"asked: ".len()).unwrap();
    let shown = skein_lib::bytes::find(&answer.stdout[start..], b"\n").unwrap();
    assert_eq!(&answer.stdout[start..start.checked_add(shown).unwrap()], &words[..shown]);
    let mut measure = crate::jig::render::Text::measure();
    measure.number(u64::try_from(shown).unwrap());
    let mut number = crate::jig::render::Text::writing_exact(measure);
    number.number(u64::try_from(shown).unwrap());
    let next = f::appended(b"read on with: task text 1 --field specification --from ", &number.finish());
    assert!(f::has(answer, &f::appended(&next, b"\n")));
    assert!(skein_lib::bytes::is_utf8(&answer.stdout));
    shown
}

#[test]
fn task_wait_continues_at_the_first_undisplayed_utf8_byte_under_each_ceiling() {
    let mut words = skein_lib::Writer::new(2001);
    words.put(&[b'x'; 511]).unwrap();
    words.put("é".as_bytes()).unwrap();
    words.put(&[b'y'; 1488]).unwrap();
    let words = words.finish();
    let (mut whole, call, _) = source_wait(&words, &crate::limits::tools(), Some(c::WatchEnd::Unknown), false);
    serve_specification(&mut whole, call, &words);
    let text_bound = u32::try_from(whole.answer(call).unwrap().stdout.len().checked_sub(128).unwrap()).unwrap();
    for ceiling in [0u8, 1, 2] {
        let mut limits = crate::limits::tools();
        match ceiling {
            0 => limits.result_bytes = 1024 + 512 * 6,
            1 => limits.part = 512,
            2 => limits.text = text_bound,
            _ => unreachable!(),
        }
        let (mut root, call, digest) = source_wait(&words, &limits, Some(c::WatchEnd::SubjectEnded), false);
        serve_specification(&mut root, call, &words);
        let answer = root.answer(call).unwrap();
        assert_eq!(answer.status, 0);
        let shown = check_source_continuation(&answer, &words);
        if ceiling == 2 {
            assert!(shown < words.len(), "exercise the renderer's text budget");
        } else {
            assert_eq!(shown, 511, "the source bound preserves the UTF-8 boundary");
        }
        assert!(u64::try_from(answer.stdout.len()).unwrap() <= limits.result_bytes);
        assert!(answer.stdout.len() <= usize::try_from(limits.text).unwrap());
        assert!(f::has(&answer, b"result: Completed"));
        if ceiling != 2 {
            assert!(f::has(&answer, b"Final task report"));
        }
        assert!(f::has(&answer, &digest));
        assert!(f::has(&answer, b"\nmet\n"));
        settled_source_wait(&mut root, call);
    }
}

#[test]
fn task_wait_retains_its_terminal_result_when_source_reads_cannot_advance() {
    for result in [
        c::ReadResult::None(c::ReadNone::Unknown),
        c::ReadResult::Value(c::ReadValue::TaskText(Box::new(c::TaskText {
            task: 1,
            field: c::TaskTextField::Specification,
            from: 128,
            total: 300,
            next: Some(128),
            text: c::Text::new(Box::new([]), 128).unwrap(),
        }))),
    ] {
        let words = [b'x'; 300];
        let (mut root, call, digest) = source_wait(&words, &crate::limits::tools(), Some(c::WatchEnd::Unknown), false);
        assert!(root.answer(call).is_none());
        let Some((_, c::Read::TaskText { from: 128, .. })) = root.reads.get(&call) else {
            panic!("supplemental read starts after the overview");
        };
        root.serve(call, result);
        let answer = root.answer(call).unwrap();
        assert_eq!(answer.status, 0);
        assert_eq!(check_source_continuation(&answer, &words), 128);
        assert!(f::has(&answer, b"Final task report"));
        assert!(f::has(&answer, &digest));
        assert!(answer.stdout.ends_with(b"met\n"));
        settled_source_wait(&mut root, call);
    }
}

#[test]
fn a_wait_until_replied_answers_at_the_parked_delivery_with_its_digest() {
    let (mut root, call) = f::waiting_call(f::WAIT_TASK, f::UNTIL_REPLIED);
    let parked = f::parked();
    let expected = f::subject_json(&crate::jig::render::Part::delivery(&parked).expect("task form"));
    root.deliver_subject(&f::task_watch(), parked);
    let answer = root.answer(call).expect("met at the parked delivery");
    assert_eq!(answer.status, 0);
    assert!(f::json_has(&answer, &expected));
    assert!(answer.stdout.ends_with(b",\"met\":true}"));
}
#[test]
fn a_read_matches_a_wait_digest() {
    let mut root = f::signed_in(f::PARTY);
    for row in f::wait_cases(&mut root) {
        let digest = row.read_digest;
        let call = root.wait_since(row.argv, &[], &digest);
        root.deliver_subject(&row.watch, row.delivery.clone());
        assert!(root.answer(call).is_none(), "unchanged {:?}", row.argv);
        let changed = f::changed_delivery(row.delivery);
        root.deliver_subject(&row.watch, changed);
        let answer = root.answer(call).expect("changed subject answers");
        assert_eq!(answer.status, 0, "{:?}", row.argv);
        assert!(!f::json_has(&answer, &digest), "{:?}", row.argv);
    }
}
#[test]
fn since_compares_the_first_delivery() {
    let mut root = f::signed_in(f::PARTY);
    let parked = f::parked();
    let different = f::task_digest(&f::writing_chat()).hex();
    let call = root.wait_since(f::WAIT_TASK, &[], &different);
    root.deliver_subject(&f::task_watch(), parked);
    assert_eq!(root.answer(call).expect("first differs").status, 0);
}
#[test]
fn until_changed_answers_the_first_change() {
    let (mut root, call) = f::waiting_call(f::WAIT_TASK, &[]);
    root.deliver_subject(&f::task_watch(), f::parked());
    assert!(root.answer(call).is_none());
    root.deliver_subject(&f::task_watch(), f::writing_chat());
    assert_eq!(root.answer(call).expect("first change").status, 0);
}
#[test]
fn both_requirements_must_hold_on_the_same_delivery() {
    let parked = f::parked();
    let mut root = f::signed_in(f::PARTY);
    let call = root.wait_since(f::WAIT_TASK, f::UNTIL_REPLIED, &f::task_digest(&parked).hex());
    for delivery in [parked.clone(), f::writing_chat(), parked.clone()] {
        root.deliver_subject(&f::task_watch(), delivery);
        assert!(root.answer(call).is_none());
    }
    root.deliver_subject(&f::task_watch(), f::changed_delivery(parked));
    assert_eq!(root.answer(call).expect("both hold now").status, 0);
}
#[test]
fn timeout_exits_3_with_the_last_subject() {
    let (mut root, call) = f::waiting_call(f::WAIT_TASK, f::SHORT_TIMEOUT);
    let parked = f::parked();
    let expected = f::subject_json(&crate::jig::render::Part::delivery(&parked).expect("task form"));
    root.deliver_subject(&f::task_watch(), parked);
    root.now = Time::ZERO.checked_add(f::WAIT_SPAN).expect("deadline");
    root.fire();
    let answer = root.answer(call).expect("timed out");
    assert_eq!(answer.status, 3);
    assert!(f::json_has(&answer, &expected));
    assert!(f::json_has(&answer, b"\"why\":\"timeout\",\"met\":false"));
}
#[test]
fn a_timeout_does_not_fire_early() {
    let (mut root, call) = f::waiting_call(f::WAIT_TASK, f::SHORT_TIMEOUT);
    assert_eq!(crate::root::next_deadline(&root.root), Time::ZERO.checked_add(f::WAIT_SPAN));
    root.fire();
    assert!(root.answer(call).is_none());
}
#[test]
fn a_me_change_rechecks_the_task_without_changing_its_digest() {
    let (mut root, call) = f::waiting_call(f::WAIT_TASK, f::UNTIL_WAITING_ON_ME);
    let task = f::goal(
        f::TASK,
        f::party_wait(c::Whom::Role { project: f::PROJECT, role: f::POLICY_ROLE }, c::WaitedFor::Question),
    );
    let delivery = c::Delivery::Task(Box::new(task));
    let digest = f::task_digest(&delivery).hex();
    root.deliver_subject(&f::task_watch(), delivery);
    assert!(root.answer(call).is_none());
    let mut me = f::me(f::PARTY);
    me.projects.get_mut(0).expect("project").role = c::HeldRole::Policy { role: f::POLICY_ROLE };
    root.deliver(c::Delivery::Me(Box::new(me)));
    let answer = root.answer(call).expect("role now holds");
    assert_eq!(answer.status, 0);
    assert!(f::json_has(&answer, &digest));
}
#[test]
fn a_meeting_delivery_wins_over_timeout_in_the_same_iteration() {
    let flags = f::extended(f::SHORT_TIMEOUT, f::UNTIL_REPLIED);
    let (mut root, call) = f::waiting_call(f::WAIT_TASK, flags.as_slice());
    root.now = Time::ZERO.checked_add(f::WAIT_SPAN).expect("deadline");
    root.deliver_subject(&f::task_watch(), f::parked());
    root.fire();
    assert_eq!(root.answer(call).expect("delivery won").status, 0);
}
#[test]
fn a_wait_keeps_its_baseline_while_behind() {
    let (mut root, call) = f::waiting_call(f::WAIT_TASK, &[]);
    root.deliver_subject(&f::task_watch(), f::parked());
    root.watch_event(&f::task_watch(), c::WatchEvent::Behind);
    root.deliver(c::Delivery::Me(Box::new(f::me(f::PARTY))));
    assert!(root.answer(call).is_none());
    root.deliver_subject(&f::task_watch(), f::writing_chat());
    assert_eq!(root.answer(call).expect("new live delivery").status, 0);
}
#[test]
fn coalesced_deliveries_answer_from_the_current_document() {
    let (mut root, call) = f::waiting_call(f::WAIT_TASK, f::UNTIL_REPLIED);
    root.deliver_subject(&f::task_watch(), f::writing_chat());
    assert!(root.answer(call).is_none());
    let changed = f::changed_delivery(f::parked());
    let expected = f::subject_json(&crate::jig::render::Part::delivery(&changed).expect("task"));
    root.deliver_subject(&f::task_watch(), changed);
    assert!(f::json_has(&root.answer(call).expect("current parked document"), &expected));
}
#[test]
fn run_text_changes_are_deliveries() {
    let (mut root, call) = f::waiting_call(f::WAIT_RUN, &[]);
    root.deliver_run(f::no_provisional());
    root.deliver_run(f::provisional());
    let answer = root.answer(call).expect("text changed");
    assert_eq!(answer.status, 0);
    assert!(f::json_has(&answer, b"draft"));
    assert!(root.answer(call).is_none());
}
#[test]
fn an_answer_releases_only_the_calls_handle() {
    let (mut root, call) = f::waiting_call(f::WAIT_TASK, f::UNTIL_REPLIED);
    let flags = f::extended(f::WAIT_TASK, &["--until", "phase:running", "--json"]);
    let other = root.submit(flags.as_slice());
    root.deliver_subject(&f::task_watch(), f::parked());
    assert_eq!(root.answer(call).expect("first call met").status, 0);
    assert!(root.answer(other).is_none());
    assert_eq!(root.watches.len(), f::TWO_CALLERS);
    root.deliver_subject(&f::task_watch(), f::writing_chat());
    assert_eq!(root.answer(other).expect("second call met").status, 0);
    assert_eq!(root.watches.len(), f::SHARED_WATCHES);
    assert_eq!(crate::root::next_deadline(&root.root), None);
    for subject in root.watches.values() {
        assert_eq!(*subject, Watchable::Jig(c::Subject::Me));
    }
}

#[test]
fn a_wait_text_uses_the_subjects_read_form() {
    let expected = f::read_answer(f::TASK_CALL, f::task_result(f::parked_task()));
    let (mut root, call) = f::plain_wait(f::UNTIL_REPLIED);
    root.deliver_subject(&f::task_watch(), f::parked());
    assert_eq!(root.answer(call).expect("text answer").stdout, f::appended(&expected.stdout, b"met\n"));
}
#[test]
fn a_timeout_text_names_why_it_was_not_met() {
    let expected = f::read_answer(f::TASK_CALL, f::task_result(f::parked_task()));
    let (mut root, call) = f::plain_wait(f::SHORT_TIMEOUT);
    root.deliver_subject(&f::task_watch(), f::parked());
    root.now = Time::ZERO.checked_add(f::WAIT_SPAN).expect("deadline");
    root.fire();
    assert_eq!(root.answer(call).expect("timeout text").stdout, f::appended(&expected.stdout, b"not met: timeout\n"));
}

#[test]
fn a_task_watch_ending_reads_how_the_task_ended_before_answering() {
    let mut root = f::signed_in(f::PARTY);
    for phase in f::ENDED_PHASES {
        let argv = f::extended(f::WAIT_TASK, f::UNTIL_ENDED);
        let argv = f::extended(
            argv.as_slice(),
            if argv.as_slice().contains(&"--until") || argv.as_slice().contains(&"--since") {
                &["--json"]
            } else {
                &["--until", "changed", "--json"]
            },
        );
        let call = root.submit(argv.as_slice());
        root.deliver_subject(&f::task_watch(), f::writing_chat());
        root.end(call, &f::task_watch(), c::WatchEnd::SubjectEnded);
        assert!(root.answer(call).is_none(), "{phase:?}: final read is pending");
        assert_eq!(root.reads.get(&call).expect("final read").1, c::Read::Task { task: f::TASK });
        let task = f::ended_task(*phase);
        let read = root.submit(f::TASK_JSON_CALL);
        root.serve(read, f::task_result(task.clone()));
        let expected = root.answer(read).expect("ended task read");
        root.serve(call, f::task_result(task));
        let answer = root.answer(call).expect("final task read");
        assert_eq!(answer.status, 0, "{phase:?}");
        assert_eq!(answer.stdout, f::met_json(&expected.stdout), "{phase:?}");
    }
}
// Both a refused opening and an admitted watch's end use the same final read.
// The task read has a live form and a ended document/result form.
const FINAL_TASK_FORMS: usize = 2;
// Any application family is unexpected on the jig task read.
const UNEXPECTED_FAMILY: u16 = 1;
const TASK_ENDS: &[c::WatchEnd] = &[c::WatchEnd::SubjectEnded, c::WatchEnd::Unknown];

fn final_task_reads(task: c::Task, ending: c::Ending) -> [c::ReadResult; FINAL_TASK_FORMS] {
    [f::task_result(task.clone()), f::ended_task_result(task, ending)]
}

fn final_read(end: c::WatchEnd, json: bool, until: &[&str]) -> (f::Harness, u32, skein_lib::Token, skein_lib::Token) {
    let flags = f::extended(until, f::SHORT_TIMEOUT);
    let (mut root, call) =
        if json { f::waiting_call(f::WAIT_TASK, flags.as_slice()) } else { f::plain_wait(flags.as_slice()) };
    let watch = root.end(call, &f::task_watch(), end);
    assert!(root.answer(call).is_none(), "{end:?}: final read pending without any delivery");
    let (read, what) = root.reads.get(&call).expect("final read");
    assert_eq!(*what, c::Read::Task { task: f::TASK });
    let read = *read;
    assert_eq!(crate::root::next_deadline(&root.root), None);
    assert_eq!(root.watches.len(), f::SHARED_WATCHES);
    (root, call, watch, read)
}

fn settled_final_read(root: &mut f::Harness, call: u32, watch: skein_lib::Token, read: skein_lib::Token) {
    assert!(root.reads.is_empty());
    assert_eq!(root.watches.len(), f::SHARED_WATCHES);
    let handle = jig_client::Handle { asker: crate::root::RUNNER.asker(), number: u64::from(call) << 32 };
    assert!(root.root.client().document(handle).is_none(), "call's watch handle released");
    assert_eq!(crate::root::next_deadline(&root.root), None);
    root.client(jig_client::Event::Up { up: jig_client::Up::ReadDone { read, result: jig_client::Readout::Limit } });
    root.client(jig_client::Event::Up { up: jig_client::Up::WatchEnded { watch, end: c::WatchEnd::SubjectEnded } });
    root.now = Time::ZERO.checked_add(f::WAIT_SPAN).expect("old wait deadline");
    root.fire();
    root.client(jig_client::Event::Up { up: jig_client::Up::SignedOut });
    assert!(root.answer(call).is_none(), "no second answer after late terminals, timer or shared sign-out");
    assert_eq!(crate::root::next_deadline(&root.root), None);
}

#[test]
fn a_replied_wait_meets_the_task_ending_before_delivery() {
    for result in final_task_reads(f::ended_task(c::Phase::Done), c::Ending::Done) {
        let expected = f::read_answer(f::TASK_JSON_CALL, result.clone());
        let (mut root, call) = f::waiting_call(f::WAIT_TASK, f::UNTIL_REPLIED);
        root.deliver_subject(&f::task_watch(), f::writing_chat());
        assert!(root.answer(call).is_none(), "running has not replied");
        let watch = root.end(call, &f::task_watch(), c::WatchEnd::SubjectEnded);
        assert!(root.answer(call).is_none(), "terminal read pending");
        let (read, what) = root.reads.get(&call).expect("final read");
        assert_eq!(*what, c::Read::Task { task: f::TASK });
        let read = *read;
        root.serve(call, result);
        let answer = root.answer(call).expect("terminal task has replied");
        assert_eq!(answer.status, 0);
        assert_eq!(answer.stdout, f::met_json(&expected.stdout));
        settled_final_read(&mut root, call, watch, read);
    }
}

#[test]
fn a_replied_wait_meets_an_initially_unknown_terminal_task() {
    for result in final_task_reads(f::ended_task(c::Phase::Done), c::Ending::Done) {
        let expected = f::read_answer(f::TASK_JSON_CALL, result.clone());
        let (mut root, call, watch, read) = final_read(c::WatchEnd::Unknown, true, f::UNTIL_REPLIED);
        root.serve(call, result);
        let answer = root.answer(call).expect("already ended task has replied");
        assert_eq!(answer.status, 0);
        assert_eq!(answer.stdout, f::met_json(&expected.stdout));
        settled_final_read(&mut root, call, watch, read);
    }
}

#[test]
fn a_task_ending_before_delivery_reads_its_terminal_result() {
    for end in TASK_ENDS {
        for (phase, ending) in [
            (c::Phase::Ended, c::Ending::Deleted),
            (c::Phase::Done, c::Ending::Done),
            (c::Phase::Failed, c::Ending::Failed { reason: None }),
            (c::Phase::Cancelled, c::Ending::Cancelled { by: c::Actor::Task(f::OTHER_TASK), reason: None, at: None }),
        ] {
            for result in final_task_reads(f::ended_task(phase), ending) {
                let expected = f::read_answer(f::TASK_JSON_CALL, result.clone());
                let (mut root, call, watch, read) = final_read(*end, true, f::UNTIL_ENDED);
                root.serve(call, result);
                let answer = root.answer(call).expect("matching terminal read");
                assert_eq!(answer.status, 0, "{end:?}, {phase:?}");
                assert_eq!(answer.stdout, f::met_json(&expected.stdout), "{end:?}, {phase:?}");
                settled_final_read(&mut root, call, watch, read);
            }
        }
    }
}

#[test]
fn a_final_task_read_must_match_the_number_and_terminal_phase() {
    let phases = f::LIVE_PHASES.len().checked_add(f::ENDED_PHASES.len()).expect("fixture phase count");
    let mut tasks = skein_lib::List::with_capacity(u32::try_from(phases).expect("fixture phases fit"));
    for phase in f::LIVE_PHASES {
        tasks.push(f::ended_task(*phase)).expect("one row per live phase");
    }
    for phase in f::ENDED_PHASES {
        let mut wrong = f::ended_task(*phase);
        wrong.chip.task = f::OTHER_TASK;
        tasks.push(wrong).expect("one wrong task per terminal phase");
    }
    for (end, until) in [
        (c::WatchEnd::SubjectEnded, f::UNTIL_ENDED),
        (c::WatchEnd::Unknown, f::UNTIL_ENDED),
        (c::WatchEnd::SubjectEnded, f::UNTIL_REPLIED),
        (c::WatchEnd::Unknown, f::UNTIL_REPLIED),
    ] {
        for task in &tasks {
            // Ended keys are self-consistent even for the wrong task number.
            for result in final_task_reads(task.clone(), c::Ending::Done) {
                let (mut root, call, watch, read) = final_read(end, true, until);
                root.serve(call, result);
                let answer = root.answer(call).expect("invalid final task fails visibly");
                assert_eq!(answer.status, 1, "{end:?}: {task:?}");
                assert!(answer.stdout.is_empty());
                assert!(!answer.stderr.is_empty());
                settled_final_read(&mut root, call, watch, read);
            }
        }
    }
}

#[test]
fn a_final_task_read_rejects_an_unexpected_successful_document() {
    for end in TASK_ENDS {
        for result in [
            jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Me(Box::new(f::me(f::PARTY))))),
            jig_client::Readout::Family {
                family: UNEXPECTED_FAMILY,
                record: Some(Box::from(b"unexpected".as_slice())),
            },
        ] {
            let (mut root, call, watch, read) = final_read(*end, true, f::UNTIL_ENDED);
            root.serve_readout(call, result);
            let answer = root.answer(call).expect("unexpected document fails visibly");
            assert_eq!(answer.status, 1);
            assert!(answer.stdout.is_empty());
            assert!(!answer.stderr.is_empty());
            settled_final_read(&mut root, call, watch, read);
        }
    }
}
#[test]
fn an_already_ended_delivery_reads_the_final_task() {
    let (mut root, call) = f::waiting_call(f::WAIT_TASK, f::UNTIL_ENDED);
    let task = f::ended_task(c::Phase::Done);
    root.final_delivery(call, task.clone());
    assert!(root.answer(call).is_none());
    assert_eq!(crate::root::next_deadline(&root.root), None);
    root.serve(call, f::task_result(task));
    assert_eq!(root.answer(call).expect("final read").status, 0);
}
#[test]
fn a_task_ended_text_uses_the_final_reads_part() {
    for end in TASK_ENDS {
        for result in final_task_reads(f::ended_task(c::Phase::Done), c::Ending::Done) {
            let expected = f::read_answer(f::TASK_CALL, result.clone());
            let (mut root, call, watch, read) = final_read(*end, false, f::UNTIL_ENDED);
            root.serve(call, result);
            let answer = root.answer(call).expect("final text");
            assert_eq!(answer.status, 0);
            assert_eq!(answer.stdout, f::appended(&expected.stdout, b"met\n"));
            settled_final_read(&mut root, call, watch, read);
        }
    }
}
#[test]
fn a_run_ending_meets_ended() {
    let mut root = f::signed_in(f::PARTY);
    let call = root.wait_since(f::WAIT_RUN, f::UNTIL_ENDED, &f::task_digest(&f::parked()).hex());
    root.deliver_subject(&f::run_watch(), f::no_provisional());
    assert!(root.answer(call).is_none());
    root.end(call, &f::run_watch(), c::WatchEnd::SubjectEnded);
    let answer = root.answer(call).expect("ended meets despite since");
    assert_eq!(answer.status, 0);
    assert!(answer.stdout.ends_with(b",\"met\":true}"));
    assert!(root.reads.is_empty());
    assert_eq!(root.watches.len(), f::SHARED_WATCHES);
}
// The tools step fixture supplies the cached document and watch end directly.
#[test]
fn a_tree_watch_ending_labels_live_rows_as_the_last_observed_snapshot() {
    for json in [false, true] {
        for since in [false, true] {
            let mut root = f::signed_in(f::PARTY);
            let mut chip = f::parked_task().chip;
            chip.title = f::document_text(b"Tier slow run 1");
            let tree = f::delivery(f::tree_at(chip));
            let digest = f::task_digest(&tree).hex();
            let expected = f::subject_json(&crate::jig::render::Part::delivery(&tree).expect("tree"));
            let flags = f::extended(f::UNTIL_ENDED, f::SHORT_TIMEOUT);
            let words = f::extended(f::WAIT_TREE, flags.as_slice());
            let words = f::extended(words.as_slice(), if json { &["--json"] } else { &[] });
            #[expect(clippy::disallowed_methods, reason = "the test passes a rendered hex digest back through argv")]
            let baseline = ["--since", core::str::from_utf8(&digest).expect("digest")];
            let words = f::extended(words.as_slice(), if since { &baseline } else { &[] });
            let call = root.submit(words.as_slice());
            root.deliver_subject(&f::tree_watch(), tree);
            assert!(root.answer(call).is_none(), "Idle root has not ended");
            let watch = root.end(call, &f::tree_watch(), c::WatchEnd::SubjectEnded);
            let answer = root.answer(call).expect("watch establishes tree completion");
            assert_eq!(answer.status, 0);
            if json {
                assert!(f::has(&answer, &expected), "cached chips and digest stay unchanged");
                assert!(f::has(&answer, b"\"completion\":{\"source\":\"watch\",\"ended\":true}"));
                assert!(f::has(&answer, b"\"snapshot\":\"last_observed\""));
            } else {
                assert!(answer.stdout.starts_with(b"tree ended (watch)\nlast observed snapshot:\n"));
                assert!(f::has(&answer, "Tier slow run 1 · Waiting for news".as_bytes()));
                assert!(f::has(&answer, &digest));
            }
            assert!(root.reads.is_empty(), "watch completion needs no final task read");
            assert_eq!(root.watches.len(), f::SHARED_WATCHES);
            let handle = jig_client::Handle { asker: crate::root::RUNNER.asker(), number: u64::from(call) << 32 };
            assert!(root.root.client().document(handle).is_none());
            assert_eq!(crate::root::next_deadline(&root.root), None);
            root.client(jig_client::Event::Up {
                up: jig_client::Up::WatchEnded { watch, end: c::WatchEnd::SubjectEnded },
            });
            root.now = Time::ZERO.checked_add(f::WAIT_SPAN).expect("old deadline");
            root.fire();
            assert!(root.answer(call).is_none(), "completion is emitted once");
            assert_eq!(crate::root::next_deadline(&root.root), None);
        }
    }
}
#[test]
fn a_terminal_tree_delivery_meets_ended_without_watch_evidence() {
    for phase in f::ENDED_PHASES {
        let (mut root, call) = f::waiting_call(f::WAIT_TREE, f::UNTIL_ENDED);
        let tree = f::delivery(f::tree_at(f::ended_task(*phase).chip));
        let expected = f::subject_json(&crate::jig::render::Part::delivery(&tree).expect("tree"));
        root.deliver_subject(&f::tree_watch(), tree);
        let answer = root.answer(call).expect("terminal delivery");
        assert_eq!(answer.status, 0);
        assert!(f::has(&answer, &expected));
        assert!(!f::has(&answer, b"\"completion\""));
        assert!(!f::has(&answer, b"\"snapshot\""));
        assert_eq!(root.watches.len(), f::SHARED_WATCHES);
        assert_eq!(crate::root::next_deadline(&root.root), None);
    }
}
#[test]
fn a_tree_timeout_retains_its_snapshot_without_claiming_completion() {
    let flags = f::extended(f::UNTIL_ENDED, f::SHORT_TIMEOUT);
    let (mut root, call) = f::waiting_call(f::WAIT_TREE, flags.as_slice());
    let tree = f::delivery(f::tree_at(f::parked_task().chip));
    let expected = f::subject_json(&crate::jig::render::Part::delivery(&tree).expect("tree"));
    root.deliver_subject(&f::tree_watch(), tree);
    assert!(root.answer(call).is_none());
    root.now = Time::ZERO.checked_add(f::WAIT_SPAN).expect("deadline");
    root.fire();
    let answer = root.answer(call).expect("timeout");
    assert_eq!(answer.status, 3);
    assert!(f::has(&answer, &expected));
    assert!(f::has(&answer, b"\"why\":\"timeout\",\"met\":false"));
    assert!(!f::has(&answer, b"\"completion\""));
    assert!(!f::has(&answer, b"\"snapshot\""));
    assert_eq!(root.watches.len(), f::SHARED_WATCHES);
    assert_eq!(crate::root::next_deadline(&root.root), None);
}
#[test]
fn a_subject_ending_unmet_keeps_its_last_digest() {
    let mut root = f::signed_in(f::PARTY);
    for flags in [&[][..], f::UNTIL_RUNNING, &["--until", "held"][..]] {
        let argv = f::extended(f::WAIT_TASK, flags);
        let argv = f::extended(
            argv.as_slice(),
            if argv.as_slice().contains(&"--until") || argv.as_slice().contains(&"--since") {
                &["--json"]
            } else {
                &["--until", "changed", "--json"]
            },
        );
        let call = root.submit(argv.as_slice());
        let parked = f::parked();
        let expected = f::subject_json(&crate::jig::render::Part::delivery(&parked).expect("task"));
        root.deliver_subject(&f::task_watch(), parked);
        root.end(call, &f::task_watch(), c::WatchEnd::SubjectEnded);
        let answer = root.answer(call).expect("ended unmet");
        assert_eq!(answer.status, 3);
        assert!(f::json_has(&answer, &expected));
        assert!(answer.stdout.ends_with(b",\"why\":\"ended\",\"met\":false}"));
    }
}
#[test]
fn a_held_wait_ending_before_delivery_invents_no_document() {
    let (mut root, call) = f::waiting_call(f::WAIT_TASK, &["--until", "held"]);
    root.end(call, &f::task_watch(), c::WatchEnd::SubjectEnded);
    let answer = root.answer(call).expect("ended without delivery");
    assert_eq!(answer.status, 3);
    assert_eq!(answer.stdout.as_ref(), b"{\"task\":null,\"why\":\"ended\",\"met\":false}");
}
#[test]
fn a_subject_no_longer_visible_exits_2() {
    let mut root = f::signed_in(f::PARTY);
    for (end, reason) in [(c::WatchEnd::Unknown, b"unknown".as_slice()), (c::WatchEnd::NotAllowed, b"not_allowed")] {
        let argv = f::extended(f::WAIT_TASK, &["--until", "changed", "--json"]);
        let call = root.submit(argv.as_slice());
        root.end(call, &f::task_watch(), end);
        let answer = root.answer(call).expect("refused");
        assert_eq!(answer.status, 2);
        assert!(skein_lib::bytes::find(&answer.stderr, reason).is_some());
        assert_eq!(crate::root::next_deadline(&root.root), None);
    }
}
#[test]
fn client_watch_failures_exit_1() {
    let mut root = f::signed_in(f::PARTY);
    for end in [c::WatchEnd::SignedOut, c::WatchEnd::NotServed] {
        let argv = f::extended(f::WAIT_TASK, &["--until", "changed", "--json"]);
        let call = root.submit(argv.as_slice());
        root.end(call, &f::task_watch(), end);
        assert_eq!(root.answer(call).expect("client failure").status, 1);
    }
}
#[test]
fn final_read_failures_keep_the_reads_status() {
    for end in TASK_ENDS {
        for (result, status) in [
            (jig_client::Readout::Limit, 1),
            (jig_client::Readout::NotServed { family: None, served: None }, 1),
            (jig_client::Readout::Jig(c::ReadResult::None(c::ReadNone::Busy)), 1),
            (jig_client::Readout::Jig(c::ReadResult::None(c::ReadNone::SignedOut)), 1),
            (jig_client::Readout::Jig(c::ReadResult::None(c::ReadNone::Unreachable)), 1),
            (jig_client::Readout::Jig(c::ReadResult::None(c::ReadNone::Unknown)), 2),
            (jig_client::Readout::Jig(c::ReadResult::None(c::ReadNone::Refused)), 2),
        ] {
            let (mut root, call, watch, read) = final_read(*end, true, f::UNTIL_ENDED);
            root.serve_readout(call, result);
            let answer = root.answer(call).expect("final read failed");
            assert_eq!(answer.status, status);
            assert!(answer.stdout.is_empty());
            assert!(!answer.stderr.is_empty());
            settled_final_read(&mut root, call, watch, read);
        }
    }
}
#[test]
fn sign_out_of_required_me_ends_the_wait() {
    let (mut root, call) = f::waiting_call(f::WAIT_TASK, f::UNTIL_WAITING_ON_ME);
    root.end(call, &Watchable::Jig(c::Subject::Me), c::WatchEnd::SignedOut);
    assert_eq!(root.answer(call).expect("me signed out").status, 1);
    assert_eq!(crate::root::next_deadline(&root.root), None);
}
#[test]
fn a_stopping_engine_reopens_before_answering() {
    let mut root = f::signed_in(f::PARTY);
    for end in [c::WatchEnd::Stopping, c::WatchEnd::Dropped, c::WatchEnd::Busy] {
        let argv = f::extended(f::WAIT_TASK, f::UNTIL_REPLIED);
        let call = root.submit(argv.as_slice());
        root.deliver_subject(&f::task_watch(), f::writing_chat());
        root.end(call, &f::task_watch(), end);
        assert!(root.answer(call).is_none());
        root.now = crate::root::next_deadline(&root.root).expect("reopen deadline");
        root.fire();
        root.deliver_subject(&f::task_watch(), f::parked());
        assert_eq!(root.answer(call).expect("reopened parked task").status, 0);
    }
}
#[test]
fn a_finishing_read_releases_the_waits_timer() {
    let flags = f::extended(f::UNTIL_ENDED, f::SHORT_TIMEOUT);
    let (mut root, call) = f::waiting_call(f::WAIT_TASK, flags.as_slice());
    root.end(call, &f::task_watch(), c::WatchEnd::SubjectEnded);
    root.now = Time::ZERO.checked_add(f::WAIT_SPAN).expect("old deadline");
    root.fire();
    assert!(root.answer(call).is_none());
    assert_eq!(crate::root::next_deadline(&root.root), None);
    root.serve(call, f::task_result(f::ended_task(c::Phase::Done)));
    assert_eq!(root.answer(call).expect("read after old timeout").status, 0);
}
#[test]
fn an_answered_wait_ignores_late_terminals() {
    let (mut root, call) = f::finishing_call();
    let token = root.reads.get(&call).expect("final read").0;
    root.serve(call, f::task_result(f::ended_task(c::Phase::Done)));
    assert_eq!(root.answer(call).expect("single answer").status, 0);
    root.client(jig_client::Event::Up {
        up: jig_client::Up::ReadDone { read: token, result: jig_client::Readout::Limit },
    });
    root.fire();
    assert!(root.answer(call).is_none());
    assert_eq!(root.watches.len(), f::SHARED_WATCHES);
    assert_eq!(crate::root::next_deadline(&root.root), None);
}

#[test]
fn an_end_without_delivery_refuses_the_run() {
    let mut root = f::signed_in(f::PARTY);
    let argv = f::extended(f::WAIT_RUN, f::UNTIL_ENDED);
    let call = root.submit(argv.as_slice());
    root.end(call, &f::run_watch(), c::WatchEnd::SubjectEnded);
    let answer = root.answer(call).expect("refused run text");
    assert_eq!(answer.status, 2);
    assert!(answer.stdout.is_empty());
    assert_eq!(answer.stderr.as_ref(), b"gone\n");
}
#[test]
fn run_and_tree_endings_without_delivery_refuse_and_release_the_wait() {
    const BASELINE: &str = "0123456789abcdef";
    for (argv, subject) in [(f::WAIT_RUN, f::run_watch()), (f::WAIT_TREE, f::tree_watch())] {
        for (end, text, json) in [
            (c::WatchEnd::SubjectEnded, b"gone\n".as_slice(), b"{\"error\":\"gone\"}".as_slice()),
            (c::WatchEnd::Unknown, b"unknown\n".as_slice(), b"{\"error\":\"unknown\"}".as_slice()),
        ] {
            for since in [false, true] {
                for as_json in [false, true] {
                    let mut root = f::signed_in(f::PARTY);
                    let flags = f::extended(f::UNTIL_ENDED, f::SHORT_TIMEOUT);
                    let words = f::extended(argv, flags.as_slice());
                    let since_flags = ["--since", BASELINE];
                    let words = f::extended(words.as_slice(), if since { &since_flags } else { &[] });
                    let words = f::extended(words.as_slice(), if as_json { &["--json"] } else { &[] });
                    let call = root.submit(words.as_slice());
                    assert!(crate::root::next_deadline(&root.root).is_some());
                    assert!(root.answer(call).is_none());
                    let watch = root.end(call, &subject, end);
                    let answer = root.answer(call).expect("ending without a delivery is refused");
                    assert_eq!(answer.status, 2, "{argv:?}, {end:?}, since={since}, json={as_json}");
                    assert!(answer.stdout.is_empty());
                    assert_eq!(answer.stderr.as_ref(), if as_json { json } else { text });
                    assert!(root.reads.is_empty());
                    assert_eq!(root.watches.len(), f::SHARED_WATCHES);
                    let handle =
                        jig_client::Handle { asker: crate::root::RUNNER.asker(), number: u64::from(call) << 32 };
                    assert!(root.root.client().document(handle).is_none(), "call's watch handle released");
                    assert_eq!(crate::root::next_deadline(&root.root), None);
                    for end in TASK_ENDS {
                        root.client(jig_client::Event::Up { up: jig_client::Up::WatchEnded { watch, end: *end } });
                    }
                    root.now = Time::ZERO.checked_add(f::WAIT_SPAN).expect("old wait deadline");
                    root.fire();
                    assert!(root.answer(call).is_none(), "one answer despite late endings and timeout");
                    assert!(root.reads.is_empty());
                    assert_eq!(root.watches.len(), f::SHARED_WATCHES);
                    assert_eq!(crate::root::next_deadline(&root.root), None);
                }
            }
        }
    }
}
#[test]
fn an_unchanged_ended_delivery_still_waits_for_the_terminal() {
    let task = f::ended_task(c::Phase::Done);
    let delivery = c::Delivery::Task(Box::new(task.clone()));
    let mut root = f::signed_in(f::PARTY);
    let call = root.wait_since(f::WAIT_TASK, f::UNTIL_ENDED, &f::task_digest(&delivery).hex());
    root.deliver_subject(&f::task_watch(), delivery);
    assert!(root.answer(call).is_none());
    assert!(root.reads.is_empty());
    root.end(call, &f::task_watch(), c::WatchEnd::SubjectEnded);
    root.serve(call, f::task_result(task));
    assert_eq!(root.answer(call).expect("terminal overrides since").status, 0);
}
#[test]
fn a_client_handle_limit_exits_1() {
    let mut root = f::signed_in_with_handle_limit();
    let call = root.submit(f::extended(f::WAIT_TASK, &["--until", "changed"]).as_slice());
    assert_eq!(root.answer(call).expect("client handle limit").status, 1);
    assert_eq!(crate::root::next_deadline(&root.root), None);
}
#[test]
fn sign_out_during_the_final_read_exits_1() {
    for end in TASK_ENDS {
        let (mut root, call, watch, read) = final_read(*end, true, f::UNTIL_ENDED);
        root.client(jig_client::Event::Up { up: jig_client::Up::SignedOut });
        assert_eq!(root.answer(call).expect("signed out final read").status, 1);
        root.serve(call, f::ended_task_result(f::ended_task(c::Phase::Done), c::Ending::Done));
        settled_final_read(&mut root, call, watch, read);
    }
}

#[test]
fn an_unknown_watch_without_a_terminal_condition_keeps_status_2() {
    for (argv, flags, watch) in [
        (f::WAIT_TASK, &["--until", "held"][..], f::task_watch()),
        (f::WAIT_TASK, f::UNTIL_RUNNING, f::task_watch()),
        (f::WAIT_TASK, &[][..], f::task_watch()),
        (f::WAIT_RUN, f::UNTIL_ENDED, f::run_watch()),
        (f::WAIT_TREE, f::UNTIL_ENDED, f::tree_watch()),
    ] {
        let (mut root, call) = f::waiting_call(argv, flags);
        root.end(call, &watch, c::WatchEnd::Unknown);
        let answer = root.answer(call).expect("unknown subject");
        assert_eq!(answer.status, 2);
        assert!(answer.stdout.is_empty());
        assert!(skein_lib::bytes::find(&answer.stderr, b"unknown").is_some());
        assert!(root.reads.is_empty());
        assert_eq!(root.watches.len(), f::SHARED_WATCHES);
        assert_eq!(crate::root::next_deadline(&root.root), None);
    }
}

#[test]
fn changed_with_since_needs_a_change_after_the_baseline() {
    let mut root = f::signed_in(f::PARTY);
    let first = f::writing_chat();
    let different = f::task_digest(&f::parked()).hex();
    let call = root.wait_since(f::WAIT_TASK, &["--until", "changed"], &different);
    root.deliver_subject(&f::task_watch(), first.clone());
    assert!(root.answer(call).is_none());
    root.deliver_subject(&f::task_watch(), f::parked());
    assert!(root.answer(call).is_none(), "changed but equals since");
    root.deliver_subject(&f::task_watch(), first);
    assert_eq!(root.answer(call).expect("later delivery differs from since").status, 0);
}
#[test]
fn task_tree_reads_the_tree_alone() {
    let jig_client::Latest::Jig(c::Delivery::Tree(mut owner)) = f::owner_tree_at(f::parked_task().chip) else {
        panic!("owner tree")
    };
    owner.folded = skein_lib::List::with_capacity(1);
    owner
        .folded
        .push(c::TreeFold {
            parent: f::TASK,
            phases: c::PhaseCounts { waiting: 1, ..c::PhaseCounts::ZERO },
            held_below: 0,
        })
        .expect("fold");
    for tree in [c::Delivery::Tree(Box::new(f::composite_tree())), c::Delivery::Tree(owner)] {
        for json in [false, true] {
            let mut root = f::signed_in(f::PARTY);
            let flags = if json { &["--tree", "--json"][..] } else { &["--tree"][..] };
            let words = f::extended(f::TASK_CALL, flags);
            let compound = root.submit(words.as_slice());
            root.serve(compound, f::task_result(f::goal(f::TASK, f::waiting_on_me())));
            root.deliver_subject(&f::tree_watch(), tree.clone());
            let compound = root.answer(compound).expect("task and tree");
            let words = f::extended(f::WAIT_TREE, if json { &["--json"] } else { &[] });
            let single = root.submit(words.as_slice());
            assert!(root.reads.is_empty());
            root.deliver_subject(&f::tree_watch(), tree.clone());
            let single = root.answer(single).expect("tree alone");
            assert_eq!(single.status, 0);
            if json {
                assert_eq!(single.stdout, f::subject_json(&crate::jig::render::Part::delivery(&tree).expect("tree")));
                assert!(f::has(&compound, &single.stdout));
            } else {
                assert!(single.stdout.starts_with(b"tree:"));
                assert!(f::has(&single, b"digest: tree "));
                assert!(compound.stdout.ends_with(&single.stdout));
                assert!(!f::has(&single, b"met\n"));
            }
            assert_eq!(crate::root::next_deadline(&root.root), None);
        }
    }
}

// Supply the engine's pages and window separately: a group cursor can leave
// all updates for the read-on call while the watch already holds them.
fn inbox_read(
    needs: u32,
    updates: u32,
    needs_more: bool,
    owner: bool,
    json: bool,
) -> (f::Harness, crate::jig::Printed, c::Delivery) {
    let mut root = f::signed_in(f::PARTY);
    let argv = f::extended(f::INBOX_CALL, if json { &["--json"] } else { &[] });
    let call = root.submit(argv.as_slice());
    let delivery;
    if owner {
        let (mut needs_page, mut window) = f::owner_first_window(needs, false);
        let (mut updates_page, _) = f::owner_first_window(updates, false);
        for index in 0..updates_page.rows.len() {
            let row = updates_page.rows.get_mut(index).expect("update row");
            row.object =
                if index == 0 { c::InboxObject::Result { position: 1 } } else { c::InboxObject::Reply { message: 2 } };
        }
        window.cached_needs = needs;
        window.updates = updates_page.rows.clone();
        window.cached_updates.value = updates;
        if needs_more {
            needs_page.after = Some(f::cursor());
        }
        root.serve(call, c::ReadResult::Value(c::ReadValue::Inbox(Box::new(needs_page))));
        if !needs_more {
            root.serve(call, c::ReadResult::Value(c::ReadValue::Inbox(Box::new(updates_page))));
        }
        delivery = c::Delivery::Inbox(Box::new(window));
    } else {
        root.serve(call, f::inbox_rows(c::InboxReadGroup::Needs, needs, needs_more));
        if !needs_more {
            root.serve(call, f::inbox_rows(c::InboxReadGroup::Updates, updates, false));
        }
        delivery = c::Delivery::Inbox(Box::new(f::inbox_window(needs, updates, false)));
    }
    assert!(root.answer(call).is_none(), "read needs the watch's form too");
    root.deliver_subject(&f::inbox_watch(), delivery.clone());
    let read = root.answer(call).expect("read and first window delivered");
    assert_eq!(read.status, 0);
    assert_eq!(root.watches.len(), f::SHARED_WATCHES, "read closes its watch");
    (root, read, delivery)
}

fn inbox_text_digest(bytes: &[u8]) -> &[u8] {
    let marker = b"digest: inbox ";
    let start =
        skein_lib::bytes::find(bytes, marker).expect("inbox digest").checked_add(marker.len()).expect("digest offset");
    bytes.get(start..start.checked_add(16).expect("digest end")).expect("sixteen digest digits")
}

#[test]
fn an_inbox_wait_since_its_reads_digest_waits_while_nothing_moves() {
    // The closed chat left a result and reply, no needs, and a needs cursor.
    let (mut root, read, delivery) = inbox_read(0, f::WINDOW_UPDATES, true, true, false);
    let digest = inbox_text_digest(&read.stdout);
    assert!(read.stdout.starts_with(b"digest: inbox "));
    assert!(f::has(&read, b"read on with: inbox --after 001122"));
    let mut argv = skein_lib::List::with_capacity(5);
    for word in [b"inbox".as_slice(), b"--since", digest, b"--timeout", b"2s"] {
        argv.push(word).expect("five arguments");
    }
    let crate::jig::call::Parsed::Call { call, json: false } = crate::table::parse(argv.as_slice()).expect("wait argv")
    else {
        panic!("plain wait")
    };
    let call = root.submit_call(call, false);
    root.deliver_subject(&f::inbox_watch(), delivery);
    assert!(root.answer(call).is_none(), "same first window must wait");
    root.now = Time::ZERO.checked_add(skein_lib::Duration::from_secs(1)).expect("before deadline");
    root.fire();
    assert!(root.answer(call).is_none(), "timeout must not fire early");
    root.now = Time::ZERO.checked_add(skein_lib::Duration::from_secs(2)).expect("deadline");
    root.fire();
    let answer = root.answer(call).expect("timeout");
    assert_eq!(answer.status, 3);
    assert!(answer.stdout.ends_with(b"not met: timeout\n"));
    assert_eq!(inbox_text_digest(&answer.stdout), digest);
    assert_eq!(root.watches.len(), f::SHARED_WATCHES);
    assert_eq!(crate::root::next_deadline(&root.root), None);
}

#[test]
fn the_inbox_reads_digest_is_its_waits_first() {
    for owner in [false, true] {
        for (needs, updates, needs_more) in [
            (f::PAGE_ROWS, 0, false),
            (0, f::WINDOW_UPDATES, false),
            (f::PAGE_ROWS, f::WINDOW_UPDATES, false),
            (f::SHORT_WINDOW, f::WINDOW_UPDATES, false),
            (f::FULL_WINDOW, f::WINDOW_UPDATES, false),
            (f::FULL_WINDOW, f::WINDOW_UPDATES, true),
            (0, f::WINDOW_UPDATES, true),
        ] {
            let (mut root, read, delivery) = inbox_read(needs, updates, needs_more, owner, true);
            let digest = f::last_digest(&read.stdout);
            // An arbitrary different digest makes the wait print its first delivery.
            let call = root.wait_since(f::INBOX_CALL, &[], b"0000000000000000");
            root.deliver_subject(&f::inbox_watch(), delivery);
            let answer = root.answer(call).expect("first delivery differs");
            assert_eq!(answer.status, 0);
            assert_eq!(
                f::last_digest(&answer.stdout),
                digest,
                "owner={owner}, needs={needs}, updates={updates}, needs_more={needs_more}"
            );
        }
    }
}

#[test]
fn an_inbox_wait_since_its_reads_digest_answers_at_a_new_update() {
    let (mut root, read, mut delivery) = inbox_read(0, f::PAGE_ROWS, true, true, true);
    let digest = f::last_digest(&read.stdout);
    let call = root.wait_since(f::INBOX_CALL, &[], &digest);
    root.deliver_subject(&f::inbox_watch(), delivery.clone());
    assert!(root.answer(call).is_none());
    let c::Delivery::Inbox(window) = &mut delivery else { panic!("owner inbox") };
    let mut updates = skein_lib::List::with_capacity(f::WINDOW_UPDATES);
    let first = window.updates.get(0).expect("existing update").clone();
    updates.push(first.clone()).expect("existing update fits");
    let mut next = first;
    next.object = c::InboxObject::Reply { message: 2 };
    updates.push(next).expect("new update fits");
    window.updates = updates;
    window.cached_updates.value = f::WINDOW_UPDATES;
    root.deliver_subject(&f::inbox_watch(), delivery);
    let answer = root.answer(call).expect("new update answers");
    assert_eq!(answer.status, 0);
    assert_ne!(f::last_digest(&answer.stdout), digest);
    assert_eq!(root.watches.len(), f::SHARED_WATCHES);
    assert_eq!(crate::root::next_deadline(&root.root), None);
}

#[test]
fn run_ended_uses_the_current_attempt_even_without_a_terminal_event() {
    for initially_live in [false, true] {
        for json in [false, true] {
            let mut root = f::signed_in(f::PARTY);
            let flags = f::extended(f::UNTIL_ENDED, if json { &["--json"] } else { &[] });
            let argv = f::extended(f::WAIT_RUN, flags.as_slice());
            let call = root.submit(argv.as_slice());
            let c::Delivery::RunState(mut run) = f::no_provisional() else { panic!("run state") };
            if initially_live {
                // No call or provisional text does not mean the run ended.
                root.deliver_subject(&f::run_watch(), c::Delivery::RunState(run.clone()));
                assert!(root.answer(call).is_none());
            }
            run.attempt = None;
            root.deliver_subject(&f::run_watch(), c::Delivery::RunState(run));
            let answer = root.answer(call).expect("no live attempt meets ended immediately");
            assert_eq!(answer.status, 0, "{answer:?}");
            if json {
                assert!(f::has(&answer, b"\"attempt\":null"));
                assert!(answer.stdout.ends_with(b",\"met\":true}"));
            } else {
                assert!(answer.stdout.starts_with(b"run: none\n"), "{answer:?}");
                assert!(answer.stdout.ends_with(b"met\n"), "{answer:?}");
            }
            assert_eq!(crate::root::next_deadline(&root.root), None);
            assert_eq!(root.watches.len(), f::SHARED_WATCHES);
        }
    }
}

#[test]
fn run_ended_with_since_needs_both_a_changed_digest_and_no_attempt() {
    let c::Delivery::RunState(mut run) = f::no_provisional() else { panic!("run state") };
    run.attempt = None;
    let ended = c::Delivery::RunState(run.clone());
    let mut root = f::signed_in(f::PARTY);
    let call = root.wait_since(f::WAIT_RUN, f::UNTIL_ENDED, &f::task_digest(&ended).hex());
    root.deliver_subject(&f::run_watch(), ended.clone());
    assert!(root.answer(call).is_none(), "ended but unchanged");
    root.deliver_subject(&f::run_watch(), f::no_provisional());
    assert!(root.answer(call).is_none(), "changed but still running");
    root.deliver_subject(&f::run_watch(), ended);
    assert!(root.answer(call).is_none(), "ended but back at the baseline");
    run.committed_turn = run.committed_turn.checked_add(1).expect("next committed turn");
    root.deliver_subject(&f::run_watch(), c::Delivery::RunState(run));
    assert_eq!(root.answer(call).expect("both hold now").status, 0);
}

const INBOX_QUESTION: &[u8] = "Plan: I’ll add a `--csv` output-mode option to the Rust CLI in `src/main.rs`, reuse the existing `TABLE` data, print a header ".as_bytes();

fn owner_question_inbox(more: bool, cut: bool) -> (c::InboxPage, c::Delivery, c::Question) {
    let (mut page, mut window) = f::owner_first_window(1, more);
    let row = page.rows.get_mut(0).expect("question row");
    row.task = c::TaskLink::Stored { number: 1, status: None };
    row.object = c::InboxObject::Question { message: 2 };
    row.cursor = c::InboxCursor::Inbox { task: 1, object: row.object, at: Some(skein_lib::Wall::EPOCH) };
    window.needs = page.rows.clone();
    page.after = None;
    window.updates = question_inbox_updates(&page, more).rows;
    window.cached_updates.value = 1;
    let mut task = f::parked_task().chip;
    task.task = 1;
    let mut words = c::DocumentText::complete(c::Text::new(INBOX_QUESTION.into(), u32::MAX).expect("question words"));
    words.cut = cut;
    let question = c::Question {
        task,
        message: 2,
        from: Some(c::Requester::Task(2)),
        words,
        since: skein_lib::Wall::EPOCH,
        acts: skein_lib::List::with_capacity(0),
        ending: None,
    };
    (page, c::Delivery::Inbox(Box::new(window)), question)
}

fn question_inbox_updates(needs: &c::InboxPage, more: bool) -> c::InboxPage {
    let mut page = needs.clone();
    let row = page.rows.get_mut(0).expect("reply row");
    row.object = c::InboxObject::Reply { message: 3 };
    row.cursor = c::InboxCursor::Inbox { task: 1, object: row.object, at: Some(skein_lib::Wall::EPOCH) };
    page.after = if more { Some(f::cursor()) } else { None };
    page
}

fn serve_question_inbox(
    root: &mut f::Harness,
    call: u32,
    page: c::InboxPage,
    delivery: c::Delivery,
    question: c::Question,
) {
    let c::Delivery::Inbox(window) = &delivery else { panic!("owner inbox") };
    let updates = question_inbox_updates(&page, window.next.is_some());
    root.serve(call, c::ReadResult::Value(c::ReadValue::Inbox(Box::new(page))));
    root.serve(call, c::ReadResult::Value(c::ReadValue::Inbox(Box::new(updates))));
    root.serve(call, c::ReadResult::Value(c::ReadValue::Question(Box::new(question))));
    root.deliver_subject(&f::inbox_watch(), delivery);
}

#[test]
fn a_met_owner_inbox_wait_prints_the_plain_reads_question_and_calls() {
    for (more, engine_cut, client_cut) in [(true, true, false), (false, false, false), (true, false, true)] {
        let cut = engine_cut || client_cut;
        let (page, delivery, mut question) = owner_question_inbox(more, engine_cut);
        if client_cut {
            question.words = c::DocumentText::complete(
                c::Text::new(
                    "é".repeat(usize::try_from(crate::limits::TEXT).expect("text bound"))
                        .into_bytes()
                        .into_boxed_slice(),
                    u32::MAX,
                )
                .expect("long question words"),
            );
        }
        let mut root = f::signed_in(f::PARTY);
        let read = root.submit(&["inbox"]);
        serve_question_inbox(&mut root, read, page.clone(), delivery.clone(), question.clone());
        let plain = root.answer(read).expect("plain inbox");
        assert_eq!(plain.status, 0);
        if client_cut {
            #[expect(clippy::disallowed_methods, reason = "the test checks a UTF-8 boundary in bounded output")]
            let shown = core::str::from_utf8(&plain.stdout).expect("cut at a character");
            let words = shown.lines().nth(1).expect("words line").strip_suffix(" (cut)").expect("cut marker");
            assert!(!words.is_empty());
            assert!(words.len() < question.words.bytes().len());
        } else {
            assert!(f::has(&plain, INBOX_QUESTION));
        }
        assert!(f::has(&plain, b"1 \xc2\xb7 task\n"));
        assert_eq!(f::has(&plain, b" (cut)\n"), cut);
        assert_eq!(f::has(&plain, b"read whole: temper ctl DIR task activity 1\n"), cut);
        assert!(f::has(&plain, b"answer: temper ctl DIR task answer 1 2 TEXT\n"));
        assert_eq!(f::has(&plain, b"read on with: inbox --after 011122\n"), more);
        assert_eq!(plain.stdout.ends_with(b"no more\n"), !more);
        let wait = root.submit(&["inbox", "--until", "waiting", "--timeout", "2s"]);
        root.pending_reads.push(wait);
        root.deliver_subject(&f::inbox_watch(), delivery.clone());
        assert_eq!(crate::root::next_deadline(&root.root), None, "met wait cancels its timer before reading");
        assert!(root.answer(wait).is_none(), "follow-up read pending");
        serve_question_inbox(&mut root, wait, page, delivery, question);
        let answer = root.answer(wait).expect("met inbox");
        assert_eq!(answer.status, 0);
        assert_eq!(answer.stdout, f::appended(&plain.stdout, b"met\n"));
        if more && !client_cut {
            assert_eq!(answer.stdout, f::golden("inbox_wait_question"));
        }
        assert!(root.reads.is_empty());
        assert_eq!(root.watches.len(), f::SHARED_WATCHES);
    }
}

#[test]
fn a_met_owner_inbox_wait_prints_the_follow_up_reads_current_digest() {
    let (page, mut delivery, question) = owner_question_inbox(true, true);
    let mut root = f::signed_in(f::PARTY);
    let wait = root.submit(&["inbox", "--until", "waiting"]);
    root.pending_reads.push(wait);
    root.deliver_subject(&f::inbox_watch(), delivery.clone());
    let first = f::task_digest(&delivery).hex();
    let c::Delivery::Inbox(window) = &mut delivery else { panic!("owner inbox") };
    window.needs.get_mut(0).expect("question row").activity = Some(7);
    let current = f::task_digest(&delivery).hex();
    assert_ne!(first, current);
    serve_question_inbox(&mut root, wait, page, delivery, question);
    let answer = root.answer(wait).expect("met inbox");
    assert_eq!(inbox_text_digest(&answer.stdout), current);
    let since = root.wait_since(&["inbox"], &[], &current);
    // Re-delivering the same watch window leaves the since wait pending.
    let (_, mut delivery, _) = owner_question_inbox(true, true);
    let c::Delivery::Inbox(window) = &mut delivery else { panic!("owner inbox") };
    window.needs.get_mut(0).expect("question row").activity = Some(7);
    root.deliver_subject(&f::inbox_watch(), delivery);
    assert!(root.answer(since).is_none());
}

#[test]
fn a_failed_met_inbox_read_keeps_the_wait_delivery_and_status() {
    let (_, delivery, _) = owner_question_inbox(true, true);
    let mut root = f::signed_in(f::PARTY);
    let wait = root.submit(&["inbox", "--until", "waiting"]);
    root.pending_reads.push(wait);
    root.deliver_subject(&f::inbox_watch(), delivery.clone());
    root.serve(wait, c::ReadResult::None(c::ReadNone::Unknown));
    // The plain inbox read still needs its digest watch.
    root.deliver_subject(&f::inbox_watch(), delivery);
    let answer = root.answer(wait).expect("failed follow-up still met");
    let failure = f::read_answer(&["task", "show", "1"], c::ReadResult::None(c::ReadNone::Unknown));
    assert_eq!(answer.status, 0);
    assert!(answer.stdout.starts_with(b"1 \xc2\xb7 question 2\n1 \xc2\xb7 task\nmore\ndigest: inbox "));
    assert!(answer.stdout.ends_with(b"met\n"));
    assert_eq!(answer.stderr, failure.stderr);
    assert_eq!(root.watches.len(), f::SHARED_WATCHES);
}

#[test]
fn owner_inbox_json_and_timeout_waits_do_not_read_again() {
    for json in [false, true] {
        let (_, delivery, _) = owner_question_inbox(true, true);
        let mut root = f::signed_in(f::PARTY);
        let argv = f::extended(
            &["inbox", "--until", if json { "waiting" } else { "changed" }, "--timeout", "2s"],
            if json { &["--json"] } else { &[] },
        );
        let wait = root.submit(argv.as_slice());
        root.deliver_subject(&f::inbox_watch(), delivery.clone());
        if !json {
            root.now = Time::ZERO.checked_add(skein_lib::Duration::from_secs(2)).expect("deadline");
            root.fire();
        }
        let answer = root.answer(wait).expect("wait answered without reading");
        assert!(root.reads.is_empty());
        if json {
            let subject = f::subject_json(&crate::jig::render::Part::delivery(&delivery).expect("inbox"));
            let expected = f::appended(b"{\"inbox\":", &subject);
            assert_eq!(answer.stdout, f::appended(&expected, b",\"met\":true}"));
            assert_eq!(answer.status, 0);
        } else {
            assert_eq!(answer.status, 3);
            assert!(answer.stdout.starts_with(b"1 \xc2\xb7 question 2\n1 \xc2\xb7 task\nmore\ndigest: inbox "));
            assert!(answer.stdout.ends_with(b"not met: timeout\n"));
        }
    }
}
