//! A change's own workflow gate (temper/docs/design/domain/forge/follows.md, 6–7).
use crate::{
    ChangeRow, Check, Domain, FollowCheck, FollowParity, FollowVerdict, Limits, Receipt, ReceiptKey, ReceiptRun,
    Request, Stored, runs, settings,
};
use alloc::boxed::Box;
use skein_forge::api;
use skein_lib::{Decimal, Env, List, Queue, Writer};
use temper_engine_domain_forge_change as c;
use temper_engine_domain_forge_landing as key;
use temper_engine_domain_forge_tiers as report;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) enum ReadKind {
    Tree(u32),
    Run,
    Report,
}
pub(crate) enum Action {
    Read { kind: ReadKind, read: api::Read },
    Write,
    Wait,
    Done,
    Held,
}

fn configured(d: &Domain, row: &ChangeRow) -> Option<(u32, settings::Follow)> {
    let project = d.repository(row.repository)?.project;
    Some((project, d.follow(project, row.follows.as_ref()?.follow)?.clone()))
}
fn joined(parts: &[&[u8]]) -> Box<[u8]> {
    let mut length = 0_usize;
    for part in parts {
        length = length.checked_add(part.len()).expect("bounded follow words");
    }
    let mut out = Writer::new(length);
    for part in parts {
        out.put(part).expect("measured follow words");
    }
    out.finish()
}
fn name(number: u32) -> Box<[u8]> {
    joined(&[b"follow.", Decimal::of(u64::from(number)).as_bytes()])
}
fn definition(check: &settings::FollowCheck) -> Box<[u8]> {
    let length =
        check.workflow.len().checked_add(check.artifact.len()).expect("bounded check").checked_add(5).expect("headers");
    let mut out = Writer::new(length);
    for word in [&check.workflow, &check.artifact] {
        out.put(&u16::try_from(word.len()).expect("bounded check member").to_be_bytes()).expect("word length");
        out.put(word).expect("word");
    }
    out.put(&[match check.format {
        report::Format::Junit => 0,
        report::Format::Lines => 1,
    }])
    .expect("format");
    out.finish()
}
/// The extra gate is selected from the change's provenance, not a landing rule.
pub(crate) fn select(d: &Domain, row: &mut ChangeRow, limits: &Limits) -> bool {
    let config = configured(d, row);
    let selected = config.is_some();
    if let Some(check) = &row.follow_check {
        let current = match &row.follows {
            Some(follows) => match &config {
                Some((_, setting)) => {
                    follows.follow == check.follow
                        && follows.commits.last() == Some(&check.source)
                        && definition(&setting.check) == check.definition
                }
                None => false,
            },
            None => false,
        };
        if !current {
            row.follow_check = None;
            invalidate(row);
        }
    }
    let mut gates = List::with_capacity(limits.change_policy.gates);
    for gate in &row.change.gates {
        if gate.kind != c::GateKind::Follow && gates.push(*gate).is_err() {
            return false;
        }
    }
    if selected
        && gates
            .push(c::Gate {
                number: c::FOLLOW_CHECK_GATE,
                kind: c::GateKind::Follow,
                blocking: true,
                freshness: c::Freshness::Exact,
                eager: false,
            })
            .is_err()
    {
        return false;
    }
    row.change.gates = gates.into_boxed();
    if !selected {
        row.follow_check = None;
    }
    true
}
fn receipt_key(d: &Domain, row: &ChangeRow) -> ReceiptKey {
    let check = row.follow_check.as_ref().expect("active follow check");
    ReceiptKey {
        project: d.repository(row.repository).expect("adopted target").project,
        repository: row.repository,
        check: Check::Gate(name(check.follow)),
        key: check.key.expect("computed key"),
    }
}
fn inputs(check: &FollowCheck) -> Box<[api::Input]> {
    let mut inputs = List::with_capacity(3);
    for input in crate::landing::inputs(check.head, check.key.expect("computed key")) {
        inputs.push(input).expect("two check inputs");
    }
    inputs
        .push(api::Input { name: Box::from(b"temper_follows".as_slice()), value: crate::commit_text(&check.source) })
        .expect("source input");
    inputs.into_boxed()
}
fn following(d: &Domain, row: &ChangeRow) -> runs::Follow {
    let (_, setting) = configured(d, row).expect("configured follow check");
    let check = row.follow_check.as_ref().expect("active check");
    let mut follow = runs::follow(
        row.task,
        0,
        crate::purpose::EffectPurpose::Step { task: row.task, purpose: 18 }.identity().expect("check purpose"),
        row.repository,
        setting.check.workflow,
        row.branch.clone(),
        check.head,
        inputs(check),
    );
    if let Some(flight) = &check.flight {
        follow.restore(flight, d.writers.get(&row.repository.forge).copied().unwrap_or(0));
    }
    follow.page = check.page;
    follow
}
fn remember(row: &mut ChangeRow, follow: &runs::Follow) {
    let check = row.follow_check.as_mut().expect("active check");
    check.flight = Some(follow.progress(None));
    check.page = follow.page;
}
fn invalidate(row: &mut ChangeRow) {
    let mut verdicts = List::with_capacity(u32::try_from(row.verdicts.len()).expect("bounded verdicts"));
    for verdict in &row.verdicts {
        if verdict.number != c::FOLLOW_CHECK_GATE {
            verdicts.push(*verdict).expect("old verdicts");
        }
    }
    row.verdicts = verdicts.into_boxed();
}
/// Called only once the policy has asked for this gate at the head.
pub(crate) fn advance(
    d: &mut Domain,
    env: &Env<Limits>,
    row: &mut ChangeRow,
    head: api::Commit,
    out: &mut Queue<Request>,
) -> Action {
    let Some((_, setting)) = configured(d, row) else { return Action::Done };
    let follows = row.follows.as_ref().expect("follow provenance");
    let Some(source) = follows.commits.last().copied() else { return Action::Held };
    let definition = definition(&setting.check);
    let same = match &row.follow_check {
        Some(check) => {
            check.follow == follows.follow
                && check.source == source
                && check.head == head
                && check.definition == definition
        }
        None => false,
    };
    if !same {
        row.follow_check = Some(FollowCheck {
            follow: follows.follow,
            source,
            head,
            definition,
            key: None,
            flight: None,
            page: 1,
            faults: 0,
            held: None,
        });
        invalidate(row);
    }
    let check = row.follow_check.as_ref().expect("active check");
    if check.held.is_some() {
        return Action::Held;
    }
    if check.key.is_none() {
        return Action::Read { kind: ReadKind::Tree(1), read: api::Read::Tree { object: head, page: 1 } };
    }
    if let Some(receipt) = d.receipt(&receipt_key(d, row)).cloned() {
        judged(d, env, row, &receipt, out);
        return Action::Done;
    }
    let mut follow = following(d, row);
    if let Some(key::Verdict::Fault(reason)) = follow.outcome {
        fault(d, row, &mut follow, fault_words(reason));
        remember(row, &follow);
        if row.follow_check.as_ref().expect("check").held.is_some() {
            return Action::Held;
        }
    }
    let check = row.follow_check.as_ref().expect("active check");
    if !follow.waiting && !follow.writes.is_empty() {
        return Action::Write;
    }
    if let Some(flight) = &check.flight
        && let Some(run) = &flight.report
        && let Some(read) =
            crate::tiers::parts::artifact_read(run.id, setting.check.artifact, setting.check.format, env)
    {
        return Action::Read { kind: ReadKind::Report, read };
    }
    match follow.read() {
        Some(read) => Action::Read { kind: ReadKind::Run, read },
        None => Action::Wait,
    }
}
/// Take the next write, keeping its progress with the outbox entry in one decision.
pub(crate) fn write(d: &Domain, row: &mut ChangeRow, entry: u64) -> Option<api::Write> {
    let mut follow = following(d, row);
    let write = follow.write(entry);
    remember(row, &follow);
    write
}
/// A terminal dispatch outcome is observed before its outbox entry is retired.
pub(crate) fn outcome(d: &mut Domain, entry: u64, outcome: skein_forge_client::Outcome, out: &mut Queue<Request>) {
    use skein_forge_client::Outcome;
    if outcome == Outcome::Queued || outcome == Outcome::Uncertain {
        return;
    }
    let Some(kept) = d.entries.get(&entry) else { return };
    let task = kept.entry.task;
    let Some(mut row) = d.changes.get(&task).cloned() else { return };
    let Some(check) = &row.follow_check else { return };
    let Some(flight) = &check.flight else { return };
    if flight.entry != entry || !flight.waiting {
        return;
    }
    let mut follow = following(d, &row);
    follow.waiting = false;
    if let api::Write::Dispatch { .. } = &kept.entry.effect.write
        && let Some(start) = kept.entry.start
    {
        follow.start = Some(start.at);
    }
    if let Some(attempt) = kept.entry.attempt {
        follow.since = Some(attempt.sent);
    }
    match outcome {
        Outcome::Made { .. } => {}
        Outcome::Failed(_) | Outcome::Raced { .. } | Outcome::Withdrawn | Outcome::Held => {
            follow.outcome = Some(key::Verdict::Fault(key::Fault::Lost));
        }
        Outcome::Queued | Outcome::Uncertain => unreachable!("nonterminal outcomes returned"),
    }
    remember(&mut row, &follow);
    row.effect = c::EffectResult::None;
    d.changes.insert(task, row.clone()).expect("known change");
    crate::domain::save_change(d, out, row);
    out.push(Request::WakeProcedure { task });
}
/// Reconstructing a follower keeps the run id, dispatch window, retry jobs and report.
pub(crate) fn answered(
    d: &mut Domain,
    env: &Env<Limits>,
    row: &mut ChangeRow,
    kind: ReadKind,
    answer: Result<api::Answer, api::Error>,
    entries: &mut List<key::Entry>,
    out: &mut Queue<Request>,
) -> Option<Action> {
    if let Err(api::Error::Busy | api::Error::Unavailable | api::Error::Timeout) = &answer {
        return Some(Action::Wait);
    }
    match kind {
        ReadKind::Tree(page) => tree_answer(env, row, page, answer, entries),
        ReadKind::Run | ReadKind::Report => {
            let mut follow = following(d, row);
            if kind == ReadKind::Report {
                let (_, setting) = configured(d, row).expect("configured check");
                match crate::tiers::parts::report_answer(setting.check.format, env, answer) {
                    report::Outcome::Failed(failures) => finish(d, env, row, &follow, Some(failures), out),
                    report::Outcome::Fault => fault(d, row, &mut follow, b"artifact report"),
                    report::Outcome::Passed => unreachable!("a failed report never passes"),
                }
            } else {
                let writer = d.writers.get(&row.repository.forge).copied().unwrap_or(0);
                let _ended = follow.answered(answer, writer, env.now, env.limits.client.lifetime);
                match follow.outcome {
                    Some(key::Verdict::Passed) => finish(d, env, row, &follow, None, out),
                    Some(key::Verdict::Fault(reason)) => fault(d, row, &mut follow, fault_words(reason)),
                    Some(key::Verdict::Running | key::Verdict::Failed) | None => {}
                }
            }
            remember(row, &follow);
            if row.follow_check.as_ref().expect("check").held.is_some() {
                Some(Action::Held)
            } else if follow.outcome.is_none() && follow.run.is_some() && follow.writes.is_empty() {
                Some(Action::Wait)
            } else {
                None
            }
        }
    }
}
fn tree_answer(
    env: &Env<Limits>,
    row: &mut ChangeRow,
    page: u32,
    answer: Result<api::Answer, api::Error>,
    entries: &mut List<key::Entry>,
) -> Option<Action> {
    let check = row.follow_check.as_mut().expect("keyed check");
    let Ok(api::Answer::Tree { entries: page_entries, more }) = answer else { return Some(Action::Wait) };
    for entry in page_entries {
        if entries
            .push(key::Entry {
                name: entry.name,
                mode: entry.mode,
                object: entry.object,
                kind: match entry.kind {
                    api::TreeEntryKind::Blob => key::EntryKind::Blob,
                    api::TreeEntryKind::Tree => key::EntryKind::Tree,
                    api::TreeEntryKind::Commit => key::EntryKind::Commit,
                },
            })
            .is_err()
        {
            return Some(Action::Held);
        }
    }
    if more {
        let Some(page) = page.checked_add(1) else { return Some(Action::Held) };
        return Some(Action::Read { kind: ReadKind::Tree(page), read: api::Read::Tree { object: check.head, page } });
    }
    let tree = key::Tree { object: check.head, entries: entries.to_boxed() };
    match key::follow_key(
        key::Repository { forge: row.repository.forge, repository: row.repository.repository },
        &name(check.follow),
        &check.definition,
        check.source,
        check.head,
        &[tree],
        &key::KeyLimits { entries: env.limits.key_entries, path_bytes: env.limits.key_path_bytes },
    ) {
        key::Keyed::Key(value) => {
            check.key = Some(value);
            None
        }
        key::Keyed::Need(_) | key::Keyed::TooLarge => Some(Action::Held),
    }
}
fn fault_words(fault: key::Fault) -> &'static [u8] {
    match fault {
        key::Fault::SetUp => b"set-up",
        key::Fault::Completion => b"completion",
        key::Fault::NoStep => b"no step",
        key::Fault::Cancelled => b"cancelled",
        key::Fault::OtherCommit => b"other commit",
        key::Fault::Lost => b"lost",
    }
}
fn fault(d: &Domain, row: &mut ChangeRow, follow: &mut runs::Follow, kind: &[u8]) {
    let retries = d
        .after_landing(d.repository(row.repository).expect("target").project)
        .expect("project tier settings")
        .verdict
        .retries;
    let check = row.follow_check.as_mut().expect("faulted check");
    check.faults = check.faults.saturating_add(1);
    if check.faults > retries {
        check.held = Some(joined(&[
            b"follow check held after ",
            Decimal::of(u64::from(check.faults)).as_bytes(),
            b" runner faults, last: ",
            kind,
        ]));
    } else {
        let attempt = u16::try_from(check.faults).expect("bounded retries").checked_add(1).expect("retry attempt");
        runs::retry(follow, attempt);
        if follow.start.is_none() {
            follow.since = None;
        }
    }
}
fn finish(
    d: &mut Domain,
    env: &Env<Limits>,
    row: &mut ChangeRow,
    follow: &runs::Follow,
    failures: Option<report::Failures>,
    out: &mut Queue<Request>,
) {
    let run = follow.run.as_ref().expect("terminal dispatched run");
    let receipt = Receipt {
        passed: failures.is_none(),
        failures,
        run: ReceiptRun { id: run.id, number: run.number, url: run.url.clone() },
        candidate: follow.commit,
        ended: env.wall,
    };
    crate::landing::put_receipt(d, env, receipt_key(d, row), receipt.clone(), out);
    judged(d, env, row, &receipt, out);
}
fn judged(d: &mut Domain, env: &Env<Limits>, row: &mut ChangeRow, receipt: &Receipt, out: &mut Queue<Request>) {
    let check = row.follow_check.as_ref().expect("judged follow check");
    let source = check.source;
    let head = check.head;
    let number = check.follow;
    let mut verdicts = List::with_capacity(env.limits.change_policy.gates);
    for prior in &row.verdicts {
        if prior.number != c::FOLLOW_CHECK_GATE {
            verdicts.push(*prior).expect("prior verdicts");
        }
    }
    verdicts
        .push(c::GateReport {
            number: c::FOLLOW_CHECK_GATE,
            head,
            status: if receipt.passed { c::Status::Passed } else { c::Status::Failed },
        })
        .expect("selected gate");
    row.verdicts = verdicts.into_boxed();
    let verdict = if receipt.passed {
        FollowVerdict::Passed
    } else {
        match &receipt.failures {
            Some(report::Failures::Tests(tests)) => {
                FollowVerdict::Failing(u32::try_from(tests.len()).expect("bounded tests"))
            }
            Some(report::Failures::Many) | None => FollowVerdict::Many,
        }
    };
    if !receipt.passed {
        let words = remarks(d, row, receipt);
        let words = crate::ResultText { bytes: words, cut: 0 }
            .fitted(env.limits.client.answer_bytes)
            .expect("the configured gate remark bound fits its cut account");
        let mut remarks = List::with_capacity(env.limits.change_policy.gates);
        for prior in &row.gate_remarks {
            if prior.number != c::FOLLOW_CHECK_GATE {
                remarks.push(prior.clone()).expect("prior remarks");
            }
        }
        remarks.push(crate::GateRemark { number: c::FOLLOW_CHECK_GATE, head, words }).expect("selected gate");
        row.gate_remarks = remarks.into_boxed();
    }
    let project = d.repository(row.repository).expect("target").project;
    if let Some(follow) = d.follow_rows.get_mut(&(project, number)) {
        follow.parity = Some(FollowParity { source, head, verdict, run: receipt.run.url.clone() });
        out.push(Request::Save { record: Stored::Follow(follow.clone()), release: None, read_afresh: None });
        out.push(Request::Marked { tasks: Box::new([crate::follows_subject(project)]) });
    }
}
fn remarks(d: &Domain, row: &ChangeRow, receipt: &Receipt) -> Box<[u8]> {
    let (_, setting) = configured(d, row).expect("configured check");
    let check = row.follow_check.as_ref().expect("check");
    let source = d
        .repository(api::Repository {
            forge: setting.source.repository.forge,
            repository: setting.source.repository.repository,
        })
        .expect("adopted source");
    let count = match &receipt.failures {
        Some(report::Failures::Tests(tests)) => Decimal::of(skein_lib::len_u64(tests.len())).as_bytes().into(),
        Some(report::Failures::Many) | None => Box::from(b"many".as_slice()),
    };
    let head = crate::commit_text(&check.head);
    let after = crate::commit_text(&check.source);
    let mut parts = List::uncapped();
    parts
        .push(joined(&[
            b"The follow's check failed at ",
            head.get(..12).expect("head prefix"),
            b" against ",
            &source.owner,
            b"/",
            &source.name,
            b" ",
            after.get(..12).expect("source prefix"),
            b": ",
            &count,
            b" of the source's tests fail on the target.\n",
        ]))
        .expect("remark header");
    match &receipt.failures {
        Some(report::Failures::Tests(tests)) => {
            for test in tests {
                parts.push(joined(&[test, b"\n"])).expect("test line");
            }
        }
        Some(report::Failures::Many) | None => {
            parts.push(Box::from(b"many\n".as_slice())).expect("many tests");
        }
    }
    parts.push(joined(&[b"Run: ", &receipt.run.url])).expect("run link");
    let mut length = 0_usize;
    for part in &parts {
        length = length.checked_add(part.len()).expect("bounded remarks");
    }
    let mut words = Writer::new(length);
    for part in parts.into_boxed() {
        words.put(&part).expect("measured remarks");
    }
    words.finish()
}

pub(crate) fn has_verdict(row: &ChangeRow, head: api::Commit) -> bool {
    for report in &row.verdicts {
        if report.number == c::FOLLOW_CHECK_GATE && report.head == head {
            return true;
        }
    }
    false
}
pub(crate) fn reports(row: &ChangeRow, supplied: &[c::GateReport], capacity: u32) -> Box<[c::GateReport]> {
    if row.follows.is_none() {
        return Box::from(supplied);
    }
    let mut reports = List::with_capacity(capacity);
    for report in supplied {
        if report.number != c::FOLLOW_CHECK_GATE {
            reports.push(*report).expect("bounded supplied reports");
        }
    }
    for report in &row.verdicts {
        if report.number == c::FOLLOW_CHECK_GATE {
            reports.push(*report).expect("selected follow gate");
        }
    }
    reports.into_boxed()
}

pub(crate) fn hinted(d: &Domain, hint: &api::Hint, out: &mut Queue<Request>) {
    if let api::Change::Commit(commit) = hint.change {
        for (_, row) in &d.changes {
            if row.repository == hint.repository
                && !row.terminal()
                && let Some(check) = &row.follow_check
                && check.head == commit
                && check.held.is_none()
            {
                out.push(Request::WakeProcedure { task: row.task });
            }
        }
    }
}

pub(crate) fn stored_bytes(check: &FollowCheck) -> Option<u64> {
    let mut bytes = skein_lib::len_u64(check.definition.len());
    if let Some(words) = &check.held {
        bytes = bytes.checked_add(skein_lib::len_u64(words.len()))?;
    }
    if let Some(flight) = &check.flight {
        bytes = bytes.checked_add(
            skein_lib::len_u64(flight.pending.len()).checked_mul(u64::try_from(size_of::<api::JobAttempt>()).ok()?)?,
        )?;
        if let Some(report) = &flight.report {
            bytes = bytes.checked_add(skein_lib::len_u64(report.url.len()))?;
        }
    }
    Some(bytes)
}

/// Only an asked gate without an exact-head verdict owns a workflow step.
pub(crate) fn asked(row: &ChangeRow) -> Option<api::Commit> {
    match &row.change.state {
        c::State::Gating { head, asked } => {
            if asked.contains(&c::FOLLOW_CHECK_GATE) && !has_verdict(row, *head) {
                Some(*head)
            } else {
                None
            }
        }
        c::State::Producing { .. }
        | c::State::Opening { .. }
        | c::State::Recreating { .. }
        | c::State::Reopening { .. }
        | c::State::Checking { .. }
        | c::State::Ready { .. }
        | c::State::Preparing { .. }
        | c::State::CancellingGates { .. }
        | c::State::Queued { .. }
        | c::State::First { .. }
        | c::State::Updating { .. }
        | c::State::WaitingProducer { .. }
        | c::State::Resolving { .. }
        | c::State::Repairing { .. }
        | c::State::Landing { .. }
        | c::State::Landed { .. }
        | c::State::NoChange { .. }
        | c::State::Ended
        | c::State::Held { .. }
        | c::State::Red { .. }
        | c::State::Cleared { .. }
        | c::State::Uncleared { .. } => None,
    }
}
pub(crate) fn veto(row: &mut ChangeRow, entry: u64, write: &api::Write) {
    let Some(check) = &mut row.follow_check else { return };
    let Some(flight) = &mut check.flight else { return };
    if flight.entry != entry {
        return;
    }
    flight.waiting = false;
    match write {
        api::Write::Dispatch { .. } => check.flight = None,
        api::Write::Rerun { attempt } => {
            let mut pending = List::with_capacity(
                u32::try_from(flight.pending.len()).expect("bounded jobs").checked_add(1).expect("vetoed job"),
            );
            pending.push(*attempt).expect("vetoed retry");
            for attempt in &flight.pending {
                pending.push(*attempt).expect("unsent retries");
            }
            flight.pending = pending.into_boxed();
        }
        api::Write::CreateIssue { .. }
        | api::Write::OpenPull { .. }
        | api::Write::Post { .. }
        | api::Write::Review { .. }
        | api::Write::Edit { .. }
        | api::Write::SetReviewers { .. }
        | api::Write::Close { .. }
        | api::Write::Reopen { .. }
        | api::Write::Merge { .. }
        | api::Write::Update { .. }
        | api::Write::Status { .. }
        | api::Write::CreateBranch { .. }
        | api::Write::DeleteBranch { .. } => {}
    }
}
