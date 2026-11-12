//! Field encodings of the lasting domain vocabulary (testing/fakes/forge.md, 2.7).
use super::Refusal;
use super::codec::{self, Sink, bytes, count, duration, flag, optional_time, time, u32, u64};
use crate::{Limits, actions, api, git, store};
use alloc::boxed::Box;
use skein_lib::{List, Reader};
pub(super) fn put_permission(out: &mut Sink, v: api::Permission) {
    out.u8(match v {
        api::Permission::None => 0,
        api::Permission::Read => 1,
        api::Permission::Write => 2,
        api::Permission::Admin => 3,
    });
}
pub(super) fn get_permission(input: &mut Reader<'_>, _limits: &Limits) -> Result<api::Permission, Refusal> {
    match codec::u8(input)? {
        0 => Ok(api::Permission::None),
        1 => Ok(api::Permission::Read),
        2 => Ok(api::Permission::Write),
        3 => Ok(api::Permission::Admin),
        _tag => Err(Refusal::Malformed("permission")),
    }
}
pub(super) fn put_state(out: &mut Sink, v: api::State) {
    out.u8(match v {
        api::State::Open => 0,
        api::State::Closed => 1,
    });
}
pub(super) fn get_state(input: &mut Reader<'_>, _limits: &Limits) -> Result<api::State, Refusal> {
    match codec::u8(input)? {
        0 => Ok(api::State::Open),
        1 => Ok(api::State::Closed),
        _tag => Err(Refusal::Malformed("state")),
    }
}
pub(super) fn put_check(out: &mut Sink, v: api::Check) {
    out.u8(match v {
        api::Check::Skipped => 0,
        api::Check::Pending => 1,
        api::Check::Passed => 2,
        api::Check::Failed => 3,
    });
}
pub(super) fn get_check(input: &mut Reader<'_>, _limits: &Limits) -> Result<api::Check, Refusal> {
    match codec::u8(input)? {
        0 => Ok(api::Check::Skipped),
        1 => Ok(api::Check::Pending),
        2 => Ok(api::Check::Passed),
        3 => Ok(api::Check::Failed),
        _tag => Err(Refusal::Malformed("check")),
    }
}
pub(super) fn put_verdict(out: &mut Sink, v: api::Verdict) {
    out.u8(match v {
        api::Verdict::Approve => 0,
        api::Verdict::RequestChanges => 1,
        api::Verdict::Comment => 2,
    });
}
pub(super) fn get_verdict(input: &mut Reader<'_>, _limits: &Limits) -> Result<api::Verdict, Refusal> {
    match codec::u8(input)? {
        0 => Ok(api::Verdict::Approve),
        1 => Ok(api::Verdict::RequestChanges),
        2 => Ok(api::Verdict::Comment),
        _tag => Err(Refusal::Malformed("verdict")),
    }
}
pub(super) fn put_removal(out: &mut Sink, v: api::QueueRemoval) {
    out.u8(match v {
        api::QueueRemoval::MergeConflict => 0,
        api::QueueRemoval::CiFailure => 1,
    });
}
pub(super) fn get_removal(input: &mut Reader<'_>, _limits: &Limits) -> Result<api::QueueRemoval, Refusal> {
    match codec::u8(input)? {
        0 => Ok(api::QueueRemoval::MergeConflict),
        1 => Ok(api::QueueRemoval::CiFailure),
        _tag => Err(Refusal::Malformed("removal")),
    }
}
pub(super) fn put_run_event(out: &mut Sink, v: actions::RunEvent) {
    out.u8(match v {
        actions::RunEvent::Push => 0,
        actions::RunEvent::PullRequest => 1,
        actions::RunEvent::Dispatch => 2,
    });
}
pub(super) fn get_run_event(input: &mut Reader<'_>, _limits: &Limits) -> Result<actions::RunEvent, Refusal> {
    match codec::u8(input)? {
        0 => Ok(actions::RunEvent::Push),
        1 => Ok(actions::RunEvent::PullRequest),
        2 => Ok(actions::RunEvent::Dispatch),
        _tag => Err(Refusal::Malformed("run_event")),
    }
}
pub(super) fn put_run_state(out: &mut Sink, v: actions::RunState) {
    out.u8(match v {
        actions::RunState::Running => 0,
        actions::RunState::Completed => 1,
    });
}
pub(super) fn get_run_state(input: &mut Reader<'_>, _limits: &Limits) -> Result<actions::RunState, Refusal> {
    match codec::u8(input)? {
        0 => Ok(actions::RunState::Running),
        1 => Ok(actions::RunState::Completed),
        _tag => Err(Refusal::Malformed("run_state")),
    }
}
pub(super) fn put_conclusion(out: &mut Sink, v: actions::Conclusion) {
    out.u8(match v {
        actions::Conclusion::Success => 0,
        actions::Conclusion::Failure => 1,
    });
}
pub(super) fn get_conclusion(input: &mut Reader<'_>, _limits: &Limits) -> Result<actions::Conclusion, Refusal> {
    match codec::u8(input)? {
        0 => Ok(actions::Conclusion::Success),
        1 => Ok(actions::Conclusion::Failure),
        _tag => Err(Refusal::Malformed("conclusion")),
    }
}
pub(super) fn put_step_status(out: &mut Sink, v: actions::StepStatus) {
    out.u8(match v {
        actions::StepStatus::Success => 0,
        actions::StepStatus::Failure => 1,
        actions::StepStatus::Cancelled => 2,
    });
}
pub(super) fn get_step_status(input: &mut Reader<'_>, _limits: &Limits) -> Result<actions::StepStatus, Refusal> {
    match codec::u8(input)? {
        0 => Ok(actions::StepStatus::Success),
        1 => Ok(actions::StepStatus::Failure),
        2 => Ok(actions::StepStatus::Cancelled),
        _tag => Err(Refusal::Malformed("step_status")),
    }
}
pub(super) fn put_failing(out: &mut Sink, v: api::Failing) {
    out.u8(match v {
        api::Failing::Workflow => 0,
        api::Failing::Setup => 1,
        api::Failing::NoStep => 2,
    });
}
pub(super) fn get_failing(input: &mut Reader<'_>, _limits: &Limits) -> Result<api::Failing, Refusal> {
    match codec::u8(input)? {
        0 => Ok(api::Failing::Workflow),
        1 => Ok(api::Failing::Setup),
        2 => Ok(api::Failing::NoStep),
        _tag => Err(Refusal::Malformed("failing")),
    }
}
pub(super) fn put_report_format(out: &mut Sink, v: api::ReportFormat) {
    out.u8(match v {
        api::ReportFormat::Junit => 0,
        api::ReportFormat::Lines => 1,
    });
}
pub(super) fn get_report_format(input: &mut Reader<'_>, _limits: &Limits) -> Result<api::ReportFormat, Refusal> {
    match codec::u8(input)? {
        0 => Ok(api::ReportFormat::Junit),
        1 => Ok(api::ReportFormat::Lines),
        _tag => Err(Refusal::Malformed("report_format")),
    }
}
pub(super) fn put_how(out: &mut Sink, v: api::MergeHow) {
    out.u8(match v {
        api::MergeHow::Style => 0,
        api::MergeHow::FastForwardOnly => 1,
    });
}
pub(super) fn get_how(input: &mut Reader<'_>, _limits: &Limits) -> Result<api::MergeHow, Refusal> {
    match codec::u8(input)? {
        0 => Ok(api::MergeHow::Style),
        1 => Ok(api::MergeHow::FastForwardOnly),
        _tag => Err(Refusal::Malformed("how")),
    }
}
pub(super) fn put_update_style(out: &mut Sink, v: api::UpdateStyle) {
    out.u8(match v {
        api::UpdateStyle::Merge => 0,
        api::UpdateStyle::Rebase => 1,
    });
}
pub(super) fn get_update_style(input: &mut Reader<'_>, _limits: &Limits) -> Result<api::UpdateStyle, Refusal> {
    match codec::u8(input)? {
        0 => Ok(api::UpdateStyle::Merge),
        1 => Ok(api::UpdateStyle::Rebase),
        _tag => Err(Refusal::Malformed("update_style")),
    }
}
pub(super) fn put_signature(out: &mut Sink, v: &git::Signature) {
    out.bytes(&v.name);
    out.bytes(&v.email);
}
pub(super) fn get_signature(input: &mut Reader<'_>, limits: &Limits) -> Result<git::Signature, Refusal> {
    Ok(git::Signature {
        name: bytes(input, limits.name_bytes, "name")?,
        email: bytes(input, limits.name_bytes, "name")?,
    })
}
pub(super) fn put_comment(out: &mut Sink, v: &store::Comment) {
    out.u64(v.author);
    out.bytes(&v.body);
    out.u64(v.created.as_nanos());
    out.flag(v.edited.is_some());
    if let Some(entry) = &v.edited {
        out.u64(entry.as_nanos());
    }
}
pub(super) fn get_comment(input: &mut Reader<'_>, limits: &Limits) -> Result<store::Comment, Refusal> {
    Ok(store::Comment {
        author: u64(input)?,
        body: bytes(input, limits.body_bytes, "body")?,
        created: time(input)?,
        edited: optional_time(input)?,
    })
}
pub(super) fn put_review(out: &mut Sink, v: &api::Review) {
    out.u64(v.id);
    out.u64(v.author);
    put_verdict(out, v.verdict);
    out.u64(v.commit);
    out.bytes(&v.body);
    out.u64(v.at.as_nanos());
    out.flag(v.official);
}
pub(super) fn get_review(input: &mut Reader<'_>, limits: &Limits) -> Result<api::Review, Refusal> {
    Ok(api::Review {
        id: u64(input)?,
        author: u64(input)?,
        verdict: get_verdict(input, limits)?,
        commit: u64(input)?,
        body: bytes(input, limits.body_bytes, "body")?,
        at: time(input)?,
        official: flag(input)?,
    })
}
pub(super) fn put_review_kept(out: &mut Sink, v: &store::Kept) {
    put_review(out, &v.review);
    out.flag(v.pending);
}
pub(super) fn get_review_kept(input: &mut Reader<'_>, limits: &Limits) -> Result<store::Kept, Refusal> {
    Ok(store::Kept { review: get_review(input, limits)?, pending: flag(input)? })
}
pub(super) fn put_job_ref(out: &mut Sink, v: &api::JobRef) {
    out.u64(v.run);
    out.u64(v.job);
    out.u32(v.attempt);
}
pub(super) fn get_job_ref(input: &mut Reader<'_>, _limits: &Limits) -> Result<api::JobRef, Refusal> {
    Ok(api::JobRef { run: u64(input)?, job: u64(input)?, attempt: u32(input)? })
}
pub(super) fn put_status(out: &mut Sink, v: &store::Status) {
    put_check(out, v.state);
    out.u64(v.author);
    out.u64(v.at.as_nanos());
    out.flag(v.rerun);
    put_optional_job_ref(out, v.job);
    out.bytes(&v.description);
    out.bytes(&v.link);
}
pub(super) fn get_status(input: &mut Reader<'_>, limits: &Limits) -> Result<store::Status, Refusal> {
    Ok(store::Status {
        state: get_check(input, limits)?,
        author: u64(input)?,
        at: time(input)?,
        rerun: flag(input)?,
        job: get_optional_job_ref(input, limits)?,
        // CI descriptions are fixed text, even when incoming bodies are bounded below ten bytes.
        description: bytes(input, limits.body_bytes.max(10), "status description")?,
        link: bytes(input, limits.body_bytes.max(limits.name_bytes.saturating_add(51)), "link")?,
    })
}
pub(super) fn put_page(out: &mut Sink, v: &store::Page) {
    out.bytes(&v.content);
    out.u64(v.author);
    out.u64(v.revision);
}
pub(super) fn get_page(input: &mut Reader<'_>, limits: &Limits) -> Result<store::Page, Refusal> {
    Ok(store::Page {
        content: bytes(input, limits.content_bytes, "content")?,
        author: u64(input)?,
        revision: u64(input)?,
    })
}
pub(super) fn put_group(out: &mut Sink, v: &crate::queue::Group) {
    out.u64(v.commit);
    put_check(out, v.verdict);
}
pub(super) fn get_group(input: &mut Reader<'_>, limits: &Limits) -> Result<crate::queue::Group, Refusal> {
    Ok(crate::queue::Group { commit: u64(input)?, verdict: get_check(input, limits)? })
}
pub(super) fn put_queue_entry(out: &mut Sink, v: &api::QueueEntry) {
    out.u64(v.number);
    out.u64(v.head);
    out.u32(v.position);
}
pub(super) fn get_queue_entry(input: &mut Reader<'_>, _limits: &Limits) -> Result<api::QueueEntry, Refusal> {
    Ok(api::QueueEntry { number: u64(input)?, head: u64(input)?, position: u32(input)? })
}
pub(super) fn put_styles(out: &mut Sink, v: store::Styles) {
    out.flag(v.merge);
    out.flag(v.rebase);
    out.flag(v.squash);
}
pub(super) fn get_styles(input: &mut Reader<'_>, _limits: &Limits) -> Result<store::Styles, Refusal> {
    Ok(store::Styles { merge: flag(input)?, rebase: flag(input)?, squash: flag(input)? })
}
pub(super) fn put_protection(out: &mut Sink, v: &api::Protection) {
    out.flag(v.linear);
    out.flag(v.code_owners);
    put_bypassing(out, &v.bypassing_users);
    out.flag(v.queue);
    out.bytes(&v.branch);
    put_contexts(out, &v.contexts);
    out.u32(v.approvals);
    out.flag(v.dismiss_stale);
}
pub(super) fn get_protection(input: &mut Reader<'_>, limits: &Limits) -> Result<api::Protection, Refusal> {
    Ok(api::Protection {
        linear: flag(input)?,
        code_owners: flag(input)?,
        bypassing_users: get_bypassing(input, limits)?,
        queue: flag(input)?,
        branch: bytes(input, limits.name_bytes, "name")?,
        contexts: get_contexts(input, limits)?,
        approvals: u32(input)?,
        dismiss_stale: flag(input)?,
    })
}
pub(super) fn put_checks(out: &mut Sink, v: &api::Checks) {
    put_cues(out, &v.cues);
    put_contexts(out, &v.contexts);
    out.u64(v.latency_min.as_nanos());
    out.u64(v.latency_max.as_nanos());
    out.u32(v.silent);
    out.u32(v.passes);
    out.u32(v.reruns);
    put_optional_cue(out, v.cue.as_ref());
}
pub(super) fn get_checks(input: &mut Reader<'_>, limits: &Limits) -> Result<api::Checks, Refusal> {
    Ok(api::Checks {
        cues: get_cues(input, limits)?,
        contexts: get_contexts(input, limits)?,
        latency_min: duration(input)?,
        latency_max: duration(input)?,
        silent: u32(input)?,
        passes: u32(input)?,
        reruns: u32(input)?,
        cue: get_optional_cue(input, limits)?,
    })
}
pub(super) fn put_context_cue(out: &mut Sink, v: &api::ContextCue) {
    out.bytes(&v.context);
    put_cue(out, &v.cue);
}
pub(super) fn get_context_cue(input: &mut Reader<'_>, limits: &Limits) -> Result<api::ContextCue, Refusal> {
    Ok(api::ContextCue { context: bytes(input, limits.name_bytes, "name")?, cue: get_cue(input, limits)? })
}
pub(super) fn put_workflow(out: &mut Sink, v: &api::Workflow) {
    put_ids(out, &v.fails);
    put_ids(out, &v.flaky);
    put_optional_report(out, v.report.as_ref());
    put_optional_name(out, v.select.as_deref());
    out.bytes(&v.file);
    put_contexts(out, &v.jobs);
    out.u64(v.latency_min.as_nanos());
    out.u64(v.latency_max.as_nanos());
    out.u32(v.silent);
    out.u32(v.passes);
    put_optional_cue(out, v.cue.as_ref());
}
pub(super) fn get_workflow(input: &mut Reader<'_>, limits: &Limits) -> Result<api::Workflow, Refusal> {
    Ok(api::Workflow {
        fails: get_ids(input, limits)?,
        flaky: get_ids(input, limits)?,
        report: get_optional_report(input, limits)?,
        select: get_optional_name(input, limits)?,
        file: bytes(input, limits.name_bytes, "name")?,
        jobs: get_contexts(input, limits)?,
        latency_min: duration(input)?,
        latency_max: duration(input)?,
        silent: u32(input)?,
        passes: u32(input)?,
        cue: get_optional_cue(input, limits)?,
    })
}
pub(super) fn put_report(out: &mut Sink, v: &api::Report) {
    out.bytes(&v.artifact);
    put_report_format(out, v.format);
    out.flag(v.deflate);
}
pub(super) fn get_report(input: &mut Reader<'_>, limits: &Limits) -> Result<api::Report, Refusal> {
    Ok(api::Report {
        artifact: bytes(input, limits.name_bytes, "name")?,
        format: get_report_format(input, limits)?,
        deflate: flag(input)?,
    })
}
pub(super) fn put_artifact(out: &mut Sink, v: &api::Artifact) {
    out.bytes(&v.name);
    out.bytes(&v.path);
    out.bytes(&v.bytes);
    out.flag(v.deflate);
}
pub(super) fn get_artifact(input: &mut Reader<'_>, limits: &Limits) -> Result<api::Artifact, Refusal> {
    Ok(api::Artifact {
        name: bytes(input, limits.name_bytes, "name")?,
        path: bytes(input, limits.name_bytes.max(12).saturating_add(4), "step_name")?,
        bytes: bytes(input, limits.content_bytes, "content")?,
        deflate: flag(input)?,
    })
}
pub(super) fn put_run(out: &mut Sink, v: &actions::Run) {
    out.u64(v.id);
    out.u64(v.number);
    out.u64(v.commit);
    put_optional_name(out, v.workflow.as_deref());
    put_run_event(out, v.event);
    out.u64(v.user);
    put_run_state(out, v.state);
    put_optional_conclusion(out, v.conclusion);
    out.u64(v.created.as_nanos());
    put_inputs(out, &v.inputs);
}
pub(super) fn get_run(input: &mut Reader<'_>, limits: &Limits) -> Result<actions::Run, Refusal> {
    Ok(actions::Run {
        id: u64(input)?,
        number: u64(input)?,
        commit: u64(input)?,
        workflow: get_optional_name(input, limits)?,
        event: get_run_event(input, limits)?,
        user: u64(input)?,
        state: get_run_state(input, limits)?,
        conclusion: get_optional_conclusion(input, limits)?,
        created: time(input)?,
        inputs: get_inputs(input, limits)?,
    })
}
pub(super) fn put_job(out: &mut Sink, v: &actions::Job) {
    out.u64(v.id);
    out.u64(v.run);
    out.u32(v.index);
    out.u32(v.attempt);
    out.bytes(&v.name);
    put_run_state(out, v.state);
    put_optional_conclusion(out, v.conclusion);
    put_steps(out, &v.steps);
}
pub(super) fn get_job(input: &mut Reader<'_>, limits: &Limits) -> Result<actions::Job, Refusal> {
    Ok(actions::Job {
        id: u64(input)?,
        run: u64(input)?,
        index: u32(input)?,
        attempt: u32(input)?,
        name: bytes(input, limits.name_bytes, "name")?,
        state: get_run_state(input, limits)?,
        conclusion: get_optional_conclusion(input, limits)?,
        steps: get_steps(input, limits)?,
    })
}
pub(super) fn put_step(out: &mut Sink, v: &actions::Step) {
    out.u32(v.number);
    out.bytes(&v.name);
    put_step_status(out, v.status);
    out.bytes(&v.log);
}
pub(super) fn get_step(input: &mut Reader<'_>, limits: &Limits) -> Result<actions::Step, Refusal> {
    Ok(actions::Step {
        number: u32(input)?,
        name: bytes(input, limits.name_bytes.max(12).saturating_add(4), "step_name")?,
        status: get_step_status(input, limits)?,
        log: bytes(
            input,
            limits
                .name_bytes
                .saturating_mul(6)
                .saturating_add(91)
                .saturating_add(limits.content_bytes.saturating_mul(2))
                .saturating_add(1),
            "log",
        )?,
    })
}
pub(super) fn put_contexts(out: &mut Sink, v: &[Box<[u8]>]) {
    out.len(v.len());
    for v in v {
        out.bytes(v);
    }
}
pub(super) fn get_contexts(input: &mut Reader<'_>, limits: &Limits) -> Result<Box<[Box<[u8]>]>, Refusal> {
    let n = count(input, limits.contexts, "contexts")?;
    let mut values = List::with_capacity(n);
    for _ in 0..n {
        let value = bytes(input, limits.name_bytes, "name")?;
        values.push(value).expect("bounded decoded array");
    }
    Ok(values.into_boxed())
}
pub(super) fn put_labels(out: &mut Sink, v: &[Box<[u8]>]) {
    out.len(v.len());
    for v in v {
        out.bytes(v);
    }
}
pub(super) fn get_labels(input: &mut Reader<'_>, limits: &Limits) -> Result<Box<[Box<[u8]>]>, Refusal> {
    let n = count(input, limits.labels, "labels")?;
    let mut values = List::with_capacity(n);
    for _ in 0..n {
        let value = bytes(input, limits.name_bytes, "name")?;
        values.push(value).expect("bounded decoded array");
    }
    Ok(values.into_boxed())
}
pub(super) fn put_reviewers(out: &mut Sink, v: &[u64]) {
    out.len(v.len());
    for v in v {
        out.u64(*v);
    }
}
pub(super) fn get_reviewers(input: &mut Reader<'_>, limits: &Limits) -> Result<Box<[u64]>, Refusal> {
    let n = count(input, limits.reviews, "reviewers")?;
    let mut values = List::with_capacity(n);
    for _ in 0..n {
        let value = u64(input)?;
        values.push(value).expect("bounded decoded array");
    }
    Ok(values.into_boxed())
}
pub(super) fn put_dependencies(out: &mut Sink, v: &[u64]) {
    out.len(v.len());
    for v in v {
        out.u64(*v);
    }
}
pub(super) fn get_dependencies(input: &mut Reader<'_>, limits: &Limits) -> Result<Box<[u64]>, Refusal> {
    let n = count(input, limits.dependencies, "dependencies")?;
    let mut values = List::with_capacity(n);
    for _ in 0..n {
        let value = u64(input)?;
        values.push(value).expect("bounded decoded array");
    }
    Ok(values.into_boxed())
}
pub(super) fn put_bypassing(out: &mut Sink, v: &[u64]) {
    out.len(v.len());
    for v in v {
        out.u64(*v);
    }
}
pub(super) fn get_bypassing(input: &mut Reader<'_>, limits: &Limits) -> Result<Box<[u64]>, Refusal> {
    let n = count(input, limits.bypassing_users, "bypassing")?;
    let mut values = List::with_capacity(n);
    for _ in 0..n {
        let value = u64(input)?;
        values.push(value).expect("bounded decoded array");
    }
    Ok(values.into_boxed())
}
pub(super) fn put_cues(out: &mut Sink, v: &[api::ContextCue]) {
    out.len(v.len());
    for v in v {
        put_context_cue(out, v);
    }
}
pub(super) fn get_cues(input: &mut Reader<'_>, limits: &Limits) -> Result<Box<[api::ContextCue]>, Refusal> {
    let n = count(input, limits.contexts, "cues")?;
    let mut values = List::with_capacity(n);
    for _ in 0..n {
        let value = get_context_cue(input, limits)?;
        values.push(value).expect("bounded decoded array");
    }
    Ok(values.into_boxed())
}
pub(super) fn put_workflows(out: &mut Sink, v: &[api::Workflow]) {
    out.len(v.len());
    for v in v {
        put_workflow(out, v);
    }
}
pub(super) fn get_workflows(input: &mut Reader<'_>, limits: &Limits) -> Result<Box<[api::Workflow]>, Refusal> {
    let n = count(input, limits.workflows, "workflows")?;
    let mut values = List::with_capacity(n);
    for _ in 0..n {
        let value = get_workflow(input, limits)?;
        values.push(value).expect("bounded decoded array");
    }
    Ok(values.into_boxed())
}
pub(super) fn put_steps(out: &mut Sink, v: &[actions::Step]) {
    out.len(v.len());
    for v in v {
        put_step(out, v);
    }
}
pub(super) fn get_steps(input: &mut Reader<'_>, limits: &Limits) -> Result<Box<[actions::Step]>, Refusal> {
    let n = count(input, 3, "steps")?;
    let mut values = List::with_capacity(n);
    for _ in 0..n {
        let value = get_step(input, limits)?;
        values.push(value).expect("bounded decoded array");
    }
    Ok(values.into_boxed())
}
pub(super) fn put_ids(out: &mut Sink, v: &[Box<[u8]>]) {
    out.len(v.len());
    for v in v {
        out.bytes(v);
    }
}
pub(super) fn get_ids(input: &mut Reader<'_>, limits: &Limits) -> Result<Box<[Box<[u8]>]>, Refusal> {
    let n = count(input, limits.content_bytes, "ids")?;
    let mut values = List::with_capacity(n);
    for _ in 0..n {
        let value = bytes(input, limits.content_bytes, "content")?;
        values.push(value).expect("bounded decoded array");
    }
    Ok(values.into_boxed())
}
pub(super) fn put_optional_job_ref(out: &mut Sink, v: Option<api::JobRef>) {
    out.flag(v.is_some());
    if let Some(v) = v {
        put_job_ref(out, &v);
    }
}
pub(super) fn get_optional_job_ref(input: &mut Reader<'_>, limits: &Limits) -> Result<Option<api::JobRef>, Refusal> {
    if flag(input)? { Ok(Some(get_job_ref(input, limits)?)) } else { Ok(None) }
}
pub(super) fn put_optional_name(out: &mut Sink, v: Option<&[u8]>) {
    out.flag(v.is_some());
    if let Some(v) = v {
        out.bytes(v);
    }
}
pub(super) fn get_optional_name(input: &mut Reader<'_>, limits: &Limits) -> Result<Option<Box<[u8]>>, Refusal> {
    if flag(input)? { Ok(Some(bytes(input, limits.name_bytes, "name")?)) } else { Ok(None) }
}
pub(super) fn put_optional_cue(out: &mut Sink, v: Option<&api::Cue>) {
    out.flag(v.is_some());
    if let Some(v) = v {
        put_cue(out, v);
    }
}
pub(super) fn get_optional_cue(input: &mut Reader<'_>, limits: &Limits) -> Result<Option<api::Cue>, Refusal> {
    if flag(input)? { Ok(Some(get_cue(input, limits)?)) } else { Ok(None) }
}
pub(super) fn put_optional_report(out: &mut Sink, v: Option<&api::Report>) {
    out.flag(v.is_some());
    if let Some(v) = v {
        put_report(out, v);
    }
}
pub(super) fn get_optional_report(input: &mut Reader<'_>, limits: &Limits) -> Result<Option<api::Report>, Refusal> {
    if flag(input)? { Ok(Some(get_report(input, limits)?)) } else { Ok(None) }
}
pub(super) fn put_optional_signature(out: &mut Sink, v: Option<&git::Signature>) {
    out.flag(v.is_some());
    if let Some(v) = v {
        put_signature(out, v);
    }
}
pub(super) fn get_optional_signature(
    input: &mut Reader<'_>,
    limits: &Limits,
) -> Result<Option<git::Signature>, Refusal> {
    if flag(input)? { Ok(Some(get_signature(input, limits)?)) } else { Ok(None) }
}
pub(super) fn put_optional_group(out: &mut Sink, v: Option<crate::queue::Group>) {
    out.flag(v.is_some());
    if let Some(v) = v {
        put_group(out, &v);
    }
}
pub(super) fn get_optional_group(
    input: &mut Reader<'_>,
    limits: &Limits,
) -> Result<Option<crate::queue::Group>, Refusal> {
    if flag(input)? { Ok(Some(get_group(input, limits)?)) } else { Ok(None) }
}
pub(super) fn put_optional_removal(out: &mut Sink, v: Option<api::QueueRemoval>) {
    out.flag(v.is_some());
    if let Some(v) = v {
        put_removal(out, v);
    }
}
pub(super) fn get_optional_removal(
    input: &mut Reader<'_>,
    limits: &Limits,
) -> Result<Option<api::QueueRemoval>, Refusal> {
    if flag(input)? { Ok(Some(get_removal(input, limits)?)) } else { Ok(None) }
}
pub(super) fn put_optional_conclusion(out: &mut Sink, v: Option<actions::Conclusion>) {
    out.flag(v.is_some());
    if let Some(v) = v {
        put_conclusion(out, v);
    }
}
pub(super) fn get_optional_conclusion(
    input: &mut Reader<'_>,
    limits: &Limits,
) -> Result<Option<actions::Conclusion>, Refusal> {
    if flag(input)? { Ok(Some(get_conclusion(input, limits)?)) } else { Ok(None) }
}
pub(super) fn put_optional_verdict(out: &mut Sink, v: Option<api::Verdict>) {
    out.flag(v.is_some());
    if let Some(v) = v {
        put_verdict(out, v);
    }
}
pub(super) fn get_optional_verdict(input: &mut Reader<'_>, limits: &Limits) -> Result<Option<api::Verdict>, Refusal> {
    if flag(input)? { Ok(Some(get_verdict(input, limits)?)) } else { Ok(None) }
}
pub(super) fn put_optional_body(out: &mut Sink, v: Option<&[u8]>) {
    out.flag(v.is_some());
    if let Some(v) = v {
        out.bytes(v);
    }
}
pub(super) fn get_optional_body(input: &mut Reader<'_>, limits: &Limits) -> Result<Option<Box<[u8]>>, Refusal> {
    if flag(input)? { Ok(Some(bytes(input, limits.body_bytes, "body")?)) } else { Ok(None) }
}
pub(super) fn put_optional_title(out: &mut Sink, v: Option<&[u8]>) {
    out.flag(v.is_some());
    if let Some(v) = v {
        out.bytes(v);
    }
}
pub(super) fn get_optional_title(input: &mut Reader<'_>, limits: &Limits) -> Result<Option<Box<[u8]>>, Refusal> {
    if flag(input)? { Ok(Some(bytes(input, limits.title_bytes, "title")?)) } else { Ok(None) }
}
pub(super) fn put_optional_protection(out: &mut Sink, v: Option<&api::Protection>) {
    out.flag(v.is_some());
    if let Some(v) = v {
        put_protection(out, v);
    }
}
pub(super) fn get_optional_protection(
    input: &mut Reader<'_>,
    limits: &Limits,
) -> Result<Option<api::Protection>, Refusal> {
    if flag(input)? { Ok(Some(get_protection(input, limits)?)) } else { Ok(None) }
}
pub(super) fn put_cue(out: &mut Sink, cue: &api::Cue) {
    match cue {
        api::Cue::Declared { path } => {
            out.u8(4);
            out.bytes(path);
        }
        api::Cue::Tests { path } => {
            out.u8(0);
            out.bytes(path);
        }
        api::Cue::PullFile { path, no_pull, step, log } => {
            out.u8(1);
            out.bytes(path);
            put_check(out, *no_pull);
            out.bytes(step);
            out.bytes(log);
        }
        api::Cue::File { path, green } => {
            out.u8(2);
            out.bytes(path);
            out.bytes(green);
        }
        api::Cue::Message { fail } => {
            out.u8(3);
            out.bytes(fail);
        }
    }
}
pub(super) fn get_cue(input: &mut Reader<'_>, limits: &Limits) -> Result<api::Cue, Refusal> {
    let cue = match codec::u8(input)? {
        0 => api::Cue::Tests { path: bytes(input, limits.name_bytes, "cue path")? },
        1 => api::Cue::PullFile {
            path: bytes(input, limits.name_bytes, "cue path")?,
            no_pull: get_check(input, limits)?,
            step: bytes(input, limits.name_bytes, "cue step")?,
            log: bytes(input, limits.content_bytes, "cue log")?,
        },
        2 => api::Cue::File {
            path: bytes(input, limits.name_bytes, "cue path")?,
            green: bytes(input, limits.content_bytes, "cue content")?,
        },
        4 => api::Cue::Declared { path: bytes(input, limits.name_bytes, "cue path")? },
        3 => api::Cue::Message { fail: bytes(input, limits.content_bytes, "cue message")? },
        _tag => return Err(Refusal::Malformed("cue")),
    };
    match &cue {
        api::Cue::PullFile { path, no_pull, log, .. } => {
            codec::require(*no_pull != api::Check::Pending, "cue verdict")?;
            if crate::ci::template_bound(path) > limits.name_bytes
                || crate::ci::template_bound(log) > limits.content_bytes
            {
                return Err(Refusal::Limits("cue expansion"));
            }
        }
        api::Cue::Declared { .. } | api::Cue::Tests { .. } | api::Cue::File { .. } | api::Cue::Message { .. } => {}
    }
    Ok(cue)
}
pub(super) fn put_inputs(out: &mut Sink, inputs: &[api::WorkflowInput]) {
    out.len(inputs.len());
    for (key, value) in inputs {
        out.bytes(key);
        out.bytes(value);
    }
}
pub(super) fn get_inputs(input: &mut Reader<'_>, limits: &Limits) -> Result<Box<[api::WorkflowInput]>, Refusal> {
    let n = count(input, limits.contexts, "inputs")?;
    let mut inputs = List::with_capacity(n);
    for _ in 0..n {
        let key = bytes(input, limits.name_bytes, "input key")?;
        let value = bytes(input, limits.body_bytes, "input value")?;
        inputs.push((key, value)).expect("bounded inputs");
    }
    Ok(inputs.into_boxed())
}
