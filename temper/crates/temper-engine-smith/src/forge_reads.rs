//! The forge's three read tables and their union inputs (domain/agent/tools.md).
use crate::{Problem, answers::Text, forge, json::Value};
use alloc::boxed::Box;
use skein_forge::api;
use skein_lib::{List, Overflow, Writer};
use smith_domain_run::HostAnswer;

pub(crate) struct Read {
    pub(crate) what: &'static [u8],
    pub(crate) former: &'static [u8],
    pub(crate) required: &'static [&'static [u8]],
    pub(crate) optional: &'static [&'static [u8]],
    rename: Option<(&'static [u8], &'static [u8])>,
}

const READ_FORGE: &[Read] = &[
    Read { what: b"branch", former: b"read_branch", required: &[b"branch"], optional: &[], rename: None },
    Read { what: b"branches", former: b"read_branches", required: &[b"prefix"], optional: &[], rename: None },
    Read { what: b"compare", former: b"read_compare", required: &[b"before", b"after"], optional: &[], rename: None },
    Read {
        what: b"file",
        former: b"read_file",
        required: &[b"commit", b"path"],
        optional: &[b"max_bytes"],
        rename: Some((b"commit", b"head")),
    },
    Read { what: b"protection", former: b"read_protection", required: &[b"branch"], optional: &[], rename: None },
    Read { what: b"settings", former: b"read_settings", required: &[], optional: &[], rename: None },
    Read { what: b"collaborators", former: b"read_collaborators", required: &[], optional: &[b"page"], rename: None },
    Read { what: b"permission", former: b"read_permission", required: &[b"user"], optional: &[], rename: None },
];

const READ_ISSUES: &[Read] = &[
    Read {
        what: b"items",
        former: b"read_items",
        required: &[],
        optional: &[b"since", b"page", b"item_kind"],
        rename: None,
    },
    Read {
        what: b"item",
        former: b"read_item",
        required: &[b"number"],
        optional: &[b"comment"],
        rename: Some((b"comment", b"after")),
    },
    Read { what: b"pull", former: b"read_pull", required: &[b"number"], optional: &[], rename: None },
    Read {
        what: b"pull_for",
        former: b"read_pull_for",
        required: &[b"branch", b"base"],
        optional: &[],
        rename: Some((b"branch", b"head")),
    },
    Read {
        what: b"reviews",
        former: b"read_reviews",
        required: &[b"number"],
        optional: &[b"page", b"skip"],
        rename: None,
    },
    Read {
        what: b"remarks",
        former: b"read_remarks",
        required: &[b"number", b"review"],
        optional: &[b"page", b"skip"],
        rename: None,
    },
    Read {
        what: b"pull_files",
        former: b"read_pull_files",
        required: &[b"number", b"commit"],
        optional: &[b"page", b"skip"],
        rename: Some((b"commit", b"head")),
    },
];

const READ_CI: &[Read] = &[
    Read { what: b"checks", former: b"read_checks", required: &[b"commit"], optional: &[], rename: None },
    Read {
        what: b"job",
        former: b"read_job",
        required: &[b"commit", b"run", b"job", b"attempt"],
        optional: &[b"max_bytes"],
        rename: Some((b"commit", b"head")),
    },
    Read { what: b"statuses", former: b"read_statuses", required: &[b"commit"], optional: &[b"page"], rename: None },
];

pub(crate) fn table(tool: &[u8]) -> &'static [Read] {
    match tool {
        b"read_forge" => READ_FORGE,
        b"read_issues" => READ_ISSUES,
        b"read_ci" => READ_CI,
        _ => &[],
    }
}

pub(crate) fn decode(tool: &[u8], value: &Value) -> Result<Result<api::Read, HostAnswer>, Problem> {
    let what = value.required(b"what")?.text(b"what")?;
    let reads = table(tool);
    for read in reads {
        if read.what != what.as_ref() {
            continue;
        }
        for member in read.required {
            if value.field(member)?.is_none() {
                let mut text = Text::new();
                text.add(tool)?;
                text.add(b" ")?;
                text.add(read.what)?;
                text.add(b" needs ")?;
                for (position, member) in read.required.iter().enumerate() {
                    if position > 0 {
                        text.add(if position.saturating_add(1) == read.required.len() { b" and " } else { b", " })?;
                    }
                    text.add(member)?;
                }
                return Ok(Err(text.finish(true)?));
            }
        }
        let mut fields = List::with_capacity(
            u32::try_from(read.required.len().saturating_add(read.optional.len())).expect("finite members"),
        );
        for member in read.required.iter().chain(read.optional) {
            if let Some(value) = value.field(member)? {
                let mut former = *member;
                if let Some((from, to)) = read.rename
                    && from == *member
                {
                    former = to;
                }
                fields.push((Box::from(former), value.clone())).expect("one slot per member");
            }
        }
        return Ok(Ok(forge::read_named(read.former, &Value::Object(fields.into_boxed()))?));
    }
    let mut text = Text::new();
    text.add(tool)?;
    text.add(b" what must be ")?;
    for (position, read) in reads.iter().enumerate() {
        if position > 0 {
            text.add(b", ")?;
        }
        text.add(read.what)?;
    }
    Ok(Err(text.finish(true)?))
}

pub(crate) fn reference(tool: &[u8], out: &mut Writer) -> Result<(), Overflow> {
    for read in table(tool) {
        out.put(b"- `what=")?;
        out.put(read.what)?;
        out.put(b"`: requires ")?;
        members(read.required, out)?;
        out.put(b"; takes repository")?;
        for member in read.required.iter().chain(read.optional) {
            out.put(b", ")?;
            out.put(member)?;
        }
        out.put(b".\n")?;
    }
    Ok(())
}

fn members(names: &[&[u8]], out: &mut Writer) -> Result<(), Overflow> {
    if names.is_empty() {
        return out.put(b"none");
    }
    for (position, name) in names.iter().enumerate() {
        if position > 0 {
            out.put(b", ")?;
        }
        out.put(name)?;
    }
    Ok(())
}

/// Answer selection is from the decoded read, independent of the declared tool name.
pub(crate) fn label(read: &api::Read) -> &'static [u8] {
    match read {
        api::Read::Branch { .. } => b"read_forge what=branch",
        api::Read::Branches { .. } => b"read_forge what=branches",
        api::Read::Compare { .. } => b"read_forge what=compare",
        api::Read::File { .. } => b"read_forge what=file",
        api::Read::Protection { .. } => b"read_forge what=protection",
        api::Read::Collaborators { .. } => b"read_forge what=collaborators",
        api::Read::Permission { .. } => b"read_forge what=permission",
        api::Read::Items { .. } => b"read_issues what=items",
        api::Read::Item { .. } => b"read_issues what=item",
        api::Read::Pull { .. } => b"read_issues what=pull",
        api::Read::PullFor { .. } => b"read_issues what=pull_for",
        api::Read::Reviews { .. } => b"read_issues what=reviews",
        api::Read::Remarks { .. } => b"read_issues what=remarks",
        api::Read::PullFiles { .. } => b"read_issues what=pull_files",
        api::Read::Checks { .. } => b"read_ci what=checks",
        api::Read::Job { .. } => b"read_ci what=job",
        api::Read::Statuses { .. } => b"read_ci what=statuses",
        api::Read::Settings => b"read_forge what=settings",
        api::Read::Locate { .. }
        | api::Read::JobState { .. }
        | api::Read::Runs { .. }
        | api::Read::Run { .. }
        | api::Read::Tree { .. }
        | api::Read::Artifact { .. }
        | api::Read::Commits { .. } => b"forge read",
    }
}

/// Kept names remain selectors for callers holding recorded tool names.
pub(crate) fn named_label(tool: &[u8]) -> &[u8] {
    match tool {
        b"read_branch" => b"read_forge what=branch",
        b"read_branches" => b"read_forge what=branches",
        b"read_compare" => b"read_forge what=compare",
        b"read_file" => b"read_forge what=file",
        b"read_protection" => b"read_forge what=protection",
        b"read_settings" => b"read_forge what=settings",
        b"read_collaborators" => b"read_forge what=collaborators",
        b"read_permission" => b"read_forge what=permission",
        b"read_items" => b"read_issues what=items",
        b"read_item" => b"read_issues what=item",
        b"read_pull" => b"read_issues what=pull",
        b"read_pull_for" => b"read_issues what=pull_for",
        b"read_reviews" => b"read_issues what=reviews",
        b"read_remarks" => b"read_issues what=remarks",
        b"read_pull_files" => b"read_issues what=pull_files",
        b"read_checks" => b"read_ci what=checks",
        b"read_job" => b"read_ci what=job",
        b"read_statuses" => b"read_ci what=statuses",
        _ => tool,
    }
}
