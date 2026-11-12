//! Operations whose reply has gone but whose late landing remains.
use super::{
    Refusal,
    codec::{self, Sink, bytes, flag, optional_u64, u64},
    fields,
};
use crate::{
    Limits,
    api::{Git, Op, Want, Write},
};
use skein_lib::Reader;
pub(super) fn put(out: &mut Sink, op: &Op) {
    match op {
        Op::Read(_) => unreachable!("reads never land late"),
        Op::Write(write) => {
            out.u8(0);
            put_write(out, write);
        }
        Op::Git(git) => {
            out.u8(1);
            put_git(out, git);
        }
    }
}
pub(super) fn get(input: &mut Reader<'_>, limits: &Limits) -> Result<Op, Refusal> {
    match codec::u8(input)? {
        0 => Ok(Op::Write(get_write(input, limits)?)),
        1 => Ok(Op::Git(get_git(input, limits)?)),
        _tag => Err(Refusal::Malformed("landing operation")),
    }
}
#[expect(clippy::too_many_lines, reason = "the exhaustive delayed-write vocabulary has one tag per operation")]
fn put_write(out: &mut Sink, write: &Write) {
    match write {
        Write::Copy { .. } => unreachable!("copies await their reply"),
        Write::DeleteRepository => {
            out.u8(1);
        }
        Write::SetProtection { protection } => {
            out.u8(2);
            fields::put_optional_protection(out, protection.as_ref());
        }
        Write::CreateIssue { title, body, labels } => {
            out.u8(3);
            out.bytes(title);
            out.bytes(body);
            fields::put_labels(out, labels);
        }
        Write::Comment { number, body } => {
            out.u8(4);
            out.u64(*number);
            out.bytes(body);
        }
        Write::EditItem { number, title, body } => {
            out.u8(5);
            out.u64(*number);
            fields::put_optional_title(out, title.as_deref());
            fields::put_optional_body(out, body.as_deref());
        }
        Write::SetDependencies { number, dependencies } => {
            out.u8(6);
            out.u64(*number);
            fields::put_dependencies(out, dependencies);
        }
        Write::EditComment { id, body } => {
            out.u8(7);
            out.u64(*id);
            out.bytes(body);
        }
        Write::DeleteComment { id } => {
            out.u8(8);
            out.u64(*id);
        }
        Write::SetLabels { number, labels } => {
            out.u8(9);
            out.u64(*number);
            fields::put_labels(out, labels);
        }
        Write::AddLabels { number, labels } => {
            out.u8(10);
            out.u64(*number);
            fields::put_labels(out, labels);
        }
        Write::RemoveLabels { number, labels } => {
            out.u8(11);
            out.u64(*number);
            fields::put_labels(out, labels);
        }
        Write::DefineLabel { name } => {
            out.u8(12);
            out.bytes(name);
        }
        Write::OpenPull { title, body, head, base, opens_empty } => {
            out.u8(13);
            out.bytes(title);
            out.bytes(body);
            out.bytes(head);
            out.bytes(base);
            out.flag(*opens_empty);
        }
        Write::RequestReviewers { number, reviewers } => {
            out.u8(14);
            out.u64(*number);
            fields::put_reviewers(out, reviewers);
        }
        Write::RemoveReviewers { number, reviewers } => {
            out.u8(15);
            out.u64(*number);
            fields::put_reviewers(out, reviewers);
        }
        Write::SetReviewers { number, reviewers } => {
            out.u8(16);
            out.u64(*number);
            fields::put_reviewers(out, reviewers);
        }
        Write::Review { number, verdict, body } => {
            out.u8(17);
            out.u64(*number);
            fields::put_optional_verdict(out, *verdict);
            out.bytes(body);
        }
        Write::Submit { number, review, verdict } => {
            out.u8(18);
            out.u64(*number);
            out.u64(*review);
            fields::put_verdict(out, *verdict);
        }
        Write::Merge { number, head, how } => {
            out.u8(19);
            out.u64(*number);
            out.u64(*head);
            fields::put_how(out, *how);
        }
        Write::Enqueue { number, head } => {
            out.u8(20);
            out.u64(*number);
            out.u64(*head);
        }
        Write::Update { number, style } => {
            out.u8(21);
            out.u64(*number);
            fields::put_update_style(out, *style);
        }
        Write::CreateBranch { branch, commit } => {
            out.u8(22);
            out.bytes(branch);
            out.u64(*commit);
        }
        Write::Close { number } => {
            out.u8(23);
            out.u64(*number);
        }
        Write::Reopen { number } => {
            out.u8(24);
            out.u64(*number);
        }
        Write::DeleteBranch { branch } => {
            out.u8(25);
            out.bytes(branch);
        }
        Write::Status { commit, context, state, description, link } => {
            out.u8(26);
            out.u64(*commit);
            out.bytes(context);
            fields::put_check(out, *state);
            out.bytes(description);
            out.bytes(link);
        }
        Write::Dispatch { workflow, branch, inputs } => {
            out.u8(27);
            out.bytes(workflow);
            out.bytes(branch);
            fields::put_inputs(out, inputs);
        }
        Write::Rerun { job } => {
            out.u8(28);
            out.u64(*job);
        }
        Write::PutPage { name, content } => {
            out.u8(29);
            out.bytes(name);
            out.bytes(content);
        }
        Write::DeletePage { name } => {
            out.u8(30);
            out.bytes(name);
        }
    }
}
fn get_write(input: &mut Reader<'_>, limits: &Limits) -> Result<Write, Refusal> {
    match codec::u8(input)? {
        1 => Ok(Write::DeleteRepository),
        2 => Ok(Write::SetProtection { protection: fields::get_optional_protection(input, limits)? }),
        3 => Ok(Write::CreateIssue {
            title: bytes(input, limits.title_bytes, "title")?,
            body: bytes(input, limits.body_bytes, "body")?,
            labels: fields::get_labels(input, limits)?,
        }),
        4 => Ok(Write::Comment { number: u64(input)?, body: bytes(input, limits.body_bytes, "body")? }),
        5 => Ok(Write::EditItem {
            number: u64(input)?,
            title: fields::get_optional_title(input, limits)?,
            body: fields::get_optional_body(input, limits)?,
        }),
        6 => Ok(Write::SetDependencies { number: u64(input)?, dependencies: fields::get_dependencies(input, limits)? }),
        7 => Ok(Write::EditComment { id: u64(input)?, body: bytes(input, limits.body_bytes, "body")? }),
        8 => Ok(Write::DeleteComment { id: u64(input)? }),
        9 => Ok(Write::SetLabels { number: u64(input)?, labels: fields::get_labels(input, limits)? }),
        10 => Ok(Write::AddLabels { number: u64(input)?, labels: fields::get_labels(input, limits)? }),
        11 => Ok(Write::RemoveLabels { number: u64(input)?, labels: fields::get_labels(input, limits)? }),
        12 => Ok(Write::DefineLabel { name: bytes(input, limits.name_bytes, "name")? }),
        13 => Ok(Write::OpenPull {
            title: bytes(input, limits.title_bytes, "title")?,
            body: bytes(input, limits.body_bytes, "body")?,
            head: bytes(input, limits.name_bytes, "name")?,
            base: bytes(input, limits.name_bytes, "name")?,
            opens_empty: flag(input)?,
        }),
        14 => Ok(Write::RequestReviewers { number: u64(input)?, reviewers: fields::get_reviewers(input, limits)? }),
        15 => Ok(Write::RemoveReviewers { number: u64(input)?, reviewers: fields::get_reviewers(input, limits)? }),
        16 => Ok(Write::SetReviewers { number: u64(input)?, reviewers: fields::get_reviewers(input, limits)? }),
        17 => Ok(Write::Review {
            number: u64(input)?,
            verdict: fields::get_optional_verdict(input, limits)?,
            body: bytes(input, limits.body_bytes, "body")?,
        }),
        18 => {
            Ok(Write::Submit { number: u64(input)?, review: u64(input)?, verdict: fields::get_verdict(input, limits)? })
        }
        19 => Ok(Write::Merge { number: u64(input)?, head: u64(input)?, how: fields::get_how(input, limits)? }),
        20 => Ok(Write::Enqueue { number: u64(input)?, head: u64(input)? }),
        21 => Ok(Write::Update { number: u64(input)?, style: fields::get_update_style(input, limits)? }),
        22 => Ok(Write::CreateBranch { branch: bytes(input, limits.name_bytes, "name")?, commit: u64(input)? }),
        23 => Ok(Write::Close { number: u64(input)? }),
        24 => Ok(Write::Reopen { number: u64(input)? }),
        25 => Ok(Write::DeleteBranch { branch: bytes(input, limits.name_bytes, "name")? }),
        26 => Ok(Write::Status {
            commit: u64(input)?,
            context: bytes(input, limits.name_bytes, "name")?,
            state: fields::get_check(input, limits)?,
            description: bytes(input, limits.body_bytes, "body")?,
            link: bytes(input, limits.body_bytes.max(limits.name_bytes.saturating_add(51)), "link")?,
        }),
        27 => Ok(Write::Dispatch {
            workflow: bytes(input, limits.name_bytes, "name")?,
            branch: bytes(input, limits.name_bytes, "name")?,
            inputs: fields::get_inputs(input, limits)?,
        }),
        28 => Ok(Write::Rerun { job: u64(input)? }),
        29 => Ok(Write::PutPage {
            name: bytes(input, limits.name_bytes, "name")?,
            content: bytes(input, limits.content_bytes, "content")?,
        }),
        30 => Ok(Write::DeletePage { name: bytes(input, limits.name_bytes, "name")? }),
        _tag => Err(Refusal::Malformed("landing write")),
    }
}
fn put_git(out: &mut Sink, git: &Git) {
    match git {
        Git::Clone => out.u8(0),
        Git::Fetch { want } => {
            out.u8(1);
            match want {
                Want::Branch(branch) => {
                    out.u8(0);
                    out.bytes(branch);
                }
                Want::Commit(commit) => {
                    out.u8(1);
                    out.u64(*commit);
                }
                Want::Default => out.u8(2),
            }
        }
        Git::Push { branch, commit, expected } => {
            out.u8(2);
            out.bytes(branch);
            out.u64(*commit);
            out.optional_u64(*expected);
        }
        Git::Create { branch, commit } => {
            out.u8(3);
            out.bytes(branch);
            out.u64(*commit);
        }
    }
}
fn get_git(input: &mut Reader<'_>, limits: &Limits) -> Result<Git, Refusal> {
    match codec::u8(input)? {
        0 => Ok(Git::Clone),
        1 => {
            let want = match codec::u8(input)? {
                0 => Want::Branch(bytes(input, limits.name_bytes, "fetch branch")?),
                1 => Want::Commit(u64(input)?),
                2 => Want::Default,
                _tag => return Err(Refusal::Malformed("fetch target")),
            };
            Ok(Git::Fetch { want })
        }
        2 => Ok(Git::Push {
            branch: bytes(input, limits.name_bytes, "push branch")?,
            commit: u64(input)?,
            expected: optional_u64(input)?,
        }),
        3 => Ok(Git::Create { branch: bytes(input, limits.name_bytes, "create branch")?, commit: u64(input)? }),
        _tag => Err(Refusal::Malformed("landing git")),
    }
}
