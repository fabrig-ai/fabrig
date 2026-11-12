//! Repository records; indexes are rebuilt, not trusted from the image.
use super::{
    Refusal,
    codec::{Sink, bytes, count, flag, optional_u64, require, time, u32, u64},
    fields,
};
use crate::{Limits, actions, store};
use skein_lib::{List, Map, Reader, Set};

pub(super) fn put(out: &mut Sink, r: &store::Repository) {
    out.u32(r.id);
    out.flag(r.gone);
    out.bytes(&r.name);
    out.bytes(&r.default);
    fields::put_checks(out, &r.checks);
    fields::put_styles(out, r.styles);
    out.u32(r.permissions.len());
    for (user, permission) in &r.permissions {
        out.u64(*user);
        fields::put_permission(out, *permission);
    }
    out.u32(r.labels.len());
    for label in &r.labels {
        out.bytes(label);
    }
    out.u64(r.numbered);
    out.u32(r.items.len());
    for (number, item) in &r.items {
        out.u64(*number);
        put_item(out, item);
    }
    out.u32(r.queue.len());
    for entry in &r.queue {
        fields::put_queue_entry(out, entry);
    }
    out.u32(r.branches.len());
    for (name, commit) in &r.branches {
        out.bytes(name);
        out.u64(*commit);
    }
    out.u32(r.has.len());
    for commit in &r.has {
        out.u64(*commit);
    }
    out.u64(r.run_number);
    out.u32(r.runs.len());
    for (commit, run) in &r.runs {
        out.u64(*commit);
        put_kept(out, run);
    }
    fields::put_workflows(out, &r.workflows);
    out.u32(r.dispatched.len());
    for (id, run) in &r.dispatched {
        out.u64(*id);
        put_kept(out, run);
    }
    out.u32(r.statuses.len());
    for (commit, statuses) in &r.statuses {
        out.u64(*commit);
        out.u32(statuses.len());
        for (context, status) in statuses {
            out.bytes(context);
            fields::put_status(out, status);
        }
    }
    out.u32(r.wiki.len());
    for (name, page) in &r.wiki {
        out.bytes(name);
        fields::put_page(out, page);
    }
    out.u64(r.revisions);
    out.len(r.failing.len());
    for failing in &r.failing {
        fields::put_failing(out, *failing);
    }
    out.len(r.opening.len());
    for opening in &r.opening {
        out.bytes(opening);
    }
    out.u32(r.opening_limit);
    fields::put_optional_protection(out, r.protection.as_ref());
    out.flag(r.hooked);
    out.flag(r.reachable);
    out.flag(r.refusing);
}
pub(super) fn get(input: &mut Reader<'_>, limits: &Limits) -> Result<store::Repository, Refusal> {
    let id = u32(input)?;
    let gone = flag(input)?;
    let name = bytes(input, limits.name_bytes, "repository name")?;
    let default = bytes(input, limits.name_bytes, "default branch")?;
    let checks = fields::get_checks(input, limits)?;
    let mut r = store::Repository::empty(limits, name, default, checks, None, false);
    r.id = id;
    r.gone = gone;
    r.styles = fields::get_styles(input, limits)?;
    for _ in 0..count(input, limits.users, "permissions")? {
        let user = u64(input)?;
        let permission = fields::get_permission(input, limits)?;
        require(
            r.permissions.insert(user, permission).expect("bounded permissions").is_none(),
            "duplicate permission",
        )?;
    }
    for _ in 0..count(input, limits.labels, "labels")? {
        let name = bytes(input, limits.name_bytes, "label")?;
        require(r.labels.insert(name).expect("bounded labels"), "duplicate label")?;
    }
    r.numbered = u64(input)?;
    for _ in 0..count(input, limits.items, "items")? {
        let number = u64(input)?;
        let item = get_item(input, limits)?;
        require(number != 0 && number <= r.numbered, "item number")?;
        require(r.recent.insert((item.updated, number)).expect("one index per item"), "duplicate item index")?;
        for (comment, _) in &item.comments {
            require(
                r.comments.insert(*comment, number).expect("bounded comment index").is_none(),
                "duplicate comment id",
            )?;
        }
        require(r.items.insert(number, item).expect("bounded items").is_none(), "duplicate item")?;
    }
    for _ in 0..count(input, limits.items, "queue")? {
        r.queue.push(fields::get_queue_entry(input, limits)?).expect("bounded queue");
    }
    for _ in 0..count(input, limits.branches, "branches")? {
        let name = bytes(input, limits.name_bytes, "branch")?;
        let commit = u64(input)?;
        require(r.branches.insert(name, commit).expect("bounded branches").is_none(), "duplicate branch")?;
    }
    for _ in 0..count(input, limits.commits, "repository commits")? {
        require(r.has.insert(u64(input)?).expect("bounded commits"), "duplicate repository commit")?;
    }
    r.run_number = u64(input)?;
    for _ in 0..count(input, limits.statuses, "CI runs")? {
        let commit = u64(input)?;
        let run = get_kept(input, limits)?;
        require(r.runs.insert(commit, run).expect("bounded runs").is_none(), "duplicate CI run")?;
    }
    r.workflows = fields::get_workflows(input, limits)?;
    for _ in 0..count(input, limits.workflow_runs, "dispatched runs")? {
        let id = u64(input)?;
        let run = get_kept(input, limits)?;
        require(r.dispatched.insert(id, run).expect("bounded dispatches").is_none(), "duplicate dispatched run")?;
    }
    for _ in 0..count(input, limits.statuses, "statuses")? {
        let commit = u64(input)?;
        let mut statuses = Map::with_capacity(limits.contexts);
        for _ in 0..count(input, limits.contexts, "status contexts")? {
            let name = bytes(input, limits.name_bytes, "status context")?;
            let status = fields::get_status(input, limits)?;
            require(statuses.insert(name, status).expect("bounded statuses").is_none(), "duplicate status")?;
        }
        require(
            r.statuses.insert(commit, statuses).expect("bounded status commits").is_none(),
            "duplicate status commit",
        )?;
    }
    for _ in 0..count(input, limits.pages, "wiki")? {
        let name = bytes(input, limits.name_bytes, "page name")?;
        let page = fields::get_page(input, limits)?;
        require(r.wiki.insert(name, page).expect("bounded wiki").is_none(), "duplicate page")?;
    }
    r.revisions = u64(input)?;
    let n = count(input, limits.contexts, "failing contexts")?;
    let mut failing = List::with_capacity(n);
    for _ in 0..n {
        failing.push(fields::get_failing(input, limits)?).expect("bounded failing contexts");
    }
    r.failing = failing.into_boxed();
    let n = count(input, limits.contexts, "opening contexts")?;
    let mut opening = List::with_capacity(n);
    for _ in 0..n {
        opening.push(bytes(input, limits.content_bytes, "opening")?).expect("bounded opening contexts");
    }
    r.opening = opening.into_boxed();
    // This admission bound belongs to the new config, not to retained content.
    let _old_opening_limit = u32(input)?;
    r.opening_limit = limits.content_bytes;
    r.protection = fields::get_optional_protection(input, limits)?;
    r.hooked = flag(input)?;
    r.reachable = flag(input)?;
    r.refusing = flag(input)?;
    Ok(r)
}
fn put_item(out: &mut Sink, item: &store::Item) {
    out.bytes(&item.title);
    out.bytes(&item.body);
    out.u64(item.author);
    fields::put_state(out, item.state);
    out.u32(item.labels.len());
    for label in &item.labels {
        out.bytes(label);
    }
    out.u32(item.comments.len());
    for (id, comment) in &item.comments {
        out.u64(*id);
        fields::put_comment(out, comment);
    }
    out.u32(item.dependencies.len());
    for number in &item.dependencies {
        out.u64(*number);
    }
    out.u64(item.created.as_nanos());
    out.u64(item.updated.as_nanos());
    out.flag(item.pull.is_some());
    if let Some(pull) = &item.pull {
        put_pull(out, pull);
    }
}
fn get_item(input: &mut Reader<'_>, limits: &Limits) -> Result<store::Item, Refusal> {
    let title = bytes(input, limits.title_bytes, "item title")?;
    let body = bytes(input, limits.body_bytes, "item body")?;
    let author = u64(input)?;
    let state = fields::get_state(input, limits)?;
    let mut labels = Set::with_capacity(limits.labels);
    for _ in 0..count(input, limits.labels, "item labels")? {
        require(
            labels.insert(bytes(input, limits.name_bytes, "item label")?).expect("bounded labels"),
            "duplicate item label",
        )?;
    }
    let mut comments = Map::with_capacity(limits.comments);
    for _ in 0..count(input, limits.comments, "comments")? {
        let id = u64(input)?;
        let comment = fields::get_comment(input, limits)?;
        require(comments.insert(id, comment).expect("bounded comments").is_none(), "duplicate comment")?;
    }
    let mut dependencies = Set::with_capacity(limits.dependencies);
    for _ in 0..count(input, limits.dependencies, "dependencies")? {
        require(dependencies.insert(u64(input)?).expect("bounded dependencies"), "duplicate dependency")?;
    }
    let created = time(input)?;
    let updated = time(input)?;
    let pull = if flag(input)? { Some(get_pull(input, limits)?) } else { None };
    Ok(store::Item { title, body, author, state, labels, comments, dependencies, created, updated, pull })
}
fn put_pull(out: &mut Sink, pull: &store::Pull) {
    out.bytes(&pull.head);
    out.bytes(&pull.base);
    out.u64(pull.commit);
    out.u64(pull.checked.as_nanos());
    out.optional_u64(pull.merged);
    out.flag(pull.queue_merged);
    fields::put_optional_removal(out, pull.dequeued);
    fields::put_optional_group(out, pull.group);
    out.u64(pull.queued_by);
    out.u32(pull.requested.len());
    for user in &pull.requested {
        out.u64(*user);
    }
    out.u32(pull.reviews.len());
    for kept in &pull.reviews {
        fields::put_review_kept(out, kept);
    }
}
fn get_pull(input: &mut Reader<'_>, limits: &Limits) -> Result<store::Pull, Refusal> {
    let head = bytes(input, limits.name_bytes, "pull head")?;
    let base = bytes(input, limits.name_bytes, "pull base")?;
    let commit = u64(input)?;
    let checked = time(input)?;
    let merged = optional_u64(input)?;
    let queue_merged = flag(input)?;
    let dequeued = fields::get_optional_removal(input, limits)?;
    let group = fields::get_optional_group(input, limits)?;
    let queued_by = u64(input)?;
    let mut requested = Set::with_capacity(limits.reviews);
    for _ in 0..count(input, limits.reviews, "requested reviews")? {
        require(requested.insert(u64(input)?).expect("bounded reviewers"), "duplicate reviewer")?;
    }
    let mut reviews = List::with_capacity(limits.reviews);
    for _ in 0..count(input, limits.reviews, "reviews")? {
        reviews.push(fields::get_review_kept(input, limits)?).expect("bounded reviews");
    }
    Ok(store::Pull {
        head,
        base,
        commit,
        checked,
        merged,
        queue_merged,
        dequeued,
        group,
        queued_by,
        requested,
        reviews,
    })
}
fn put_kept(out: &mut Sink, kept: &actions::Kept) {
    fields::put_run(out, &kept.run);
    out.optional_u64(kept.pull);
    out.u32(kept.jobs.len());
    for job in &kept.jobs {
        fields::put_job(out, job);
    }
    out.flag(kept.artifact.is_some());
    if let Some(artifact) = &kept.artifact {
        fields::put_artifact(out, artifact);
    }
}
fn get_kept(input: &mut Reader<'_>, limits: &Limits) -> Result<actions::Kept, Refusal> {
    let run = fields::get_run(input, limits)?;
    let pull = optional_u64(input)?;
    let mut jobs = List::with_capacity(limits.contexts);
    for _ in 0..count(input, limits.contexts, "jobs")? {
        jobs.push(fields::get_job(input, limits)?).expect("bounded jobs");
    }
    let artifact = if flag(input)? { Some(fields::get_artifact(input, limits)?) } else { None };
    Ok(actions::Kept { run, pull, jobs, artifact })
}
