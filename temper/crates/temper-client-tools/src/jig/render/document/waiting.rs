//! A wait's result envelope is outside its subject's common digest.
use super::{Limits, Text};
use crate::jig::render::Part;
use jig_client::contract as c;
use skein_json::writer::Encoder;
use skein_ui::phrases::Phrasebook;
#[derive(Debug)]
pub(crate) struct Waiting<'a> {
    pub digest: Option<crate::jig::digest::Digest>,
    pub specification: Option<&'a crate::jig::runner::specification::Specification>,
    pub result: Option<&'a c::Result>,
    pub delivery: Option<&'a c::Delivery>,
    pub me: Option<&'a c::Me>,
    pub name: &'a [u8],
    pub met: bool,
    pub watch_ended: bool,
    pub why: &'a [u8],
}
pub(super) fn encode(wait: &Waiting<'_>, limits: &Limits, out: &mut Encoder) {
    out.object_start();
    out.key(wait.name);
    match wait.delivery {
        Some(c::Delivery::Task(task)) if wait.result.is_some() => {
            let mut cut = super::Cut {
                room: usize::try_from(limits.text).expect("text fits"),
                left: 0,
                turn: None,
                subject: true,
                digest: wait.digest,
            };
            super::tasks::encode_ended_task(task, wait.result.expect("ended task result"), out, &mut cut);
        }
        Some(delivery) => {
            super::subjects::encode_as(&Part::delivery(delivery).expect("subject form"), limits, wait.digest, out);
        }
        None => out.null(),
    }
    if watch_tree(wait) {
        out.key(b"completion");
        out.object_start();
        out.key(b"source");
        out.string(b"watch");
        out.key(b"ended");
        out.boolean(true);
        out.object_end();
        out.key(b"snapshot");
        out.string(b"last_observed");
    }
    if !wait.met {
        out.key(b"why");
        out.string(wait.why);
    }
    out.key(b"met");
    out.boolean(wait.met);
    out.object_end();
}
pub(super) fn text(
    wait: &Waiting<'_>,
    book: &Phrasebook,
    limits: &Limits,
    vocabulary: &dyn crate::jig::render::Parts,
    out: &mut Text,
) {
    let most = usize::try_from(limits.rows).expect("rows fit");
    if watch_tree(wait) {
        out.put(b"tree ended (watch)\nlast observed snapshot:\n");
    }
    if let Some(delivery) = wait.delivery {
        subject_text(wait, book, limits, vocabulary, out);
        let part = Part::delivery(delivery).expect("subject form");
        let list = match part {
            Part::Inbox(_) | Part::Tasks(_) => true,
            Part::Me(_)
            | Part::Task(_)
            | Part::Tree(_)
            | Part::Run(_)
            | Part::RunState(_)
            | Part::InboxPage(_)
            | Part::TaskPage(_) => false,
        };
        if list {
            out.put(if part.more() || part.rows() > most { b"more\n" } else { b"no more\n" });
        }
        super::subjects::text_digest(wait.name, &part, limits, out);
    }
    if wait.delivery.is_none() && (wait.met || wait.why == b"ended") {
        out.put(wait.name);
        out.put(b" ended\n");
    }
    if wait.met {
        out.put(b"met\n");
    } else {
        out.put(b"not met: ");
        out.put(wait.why);
        out.put(b"\n");
    }
}

fn watch_tree(wait: &Waiting<'_>) -> bool {
    wait.watch_ended && wait.name == b"tree"
}
fn subject_text(
    wait: &Waiting<'_>,
    book: &Phrasebook,
    limits: &Limits,
    vocabulary: &dyn crate::jig::render::Parts,
    out: &mut Text,
) {
    let most = usize::try_from(limits.rows).expect("rows fit");
    let Some(delivery) = wait.delivery else { return };
    match delivery {
        c::Delivery::Me(me) => super::me_text(me, book, limits, out),
        c::Delivery::Task(task) => {
            super::task_with_specification(task, wait.me, book, vocabulary, wait.specification, limits, out);
        }
        c::Delivery::Tree(_) => super::super::lists::tree(delivery, None, book, out),
        c::Delivery::Tasks(tasks) => {
            for row in tasks.rows.as_slice().iter().take(most) {
                super::super::lists::chip(&row.chip, book, out);
            }
        }

        c::Delivery::Inbox(inbox) => {
            for row in inbox.needs.as_slice().iter().chain(inbox.updates.as_slice()).take(most) {
                super::super::lists::owner_inbox_row(row, book, out);
            }
        }

        c::Delivery::Run(_) => super::super::activity::run(delivery, None, out),
        c::Delivery::RunState(run) => super::super::activity::run(delivery, run.writing.as_ref(), out),
        c::Delivery::Project(_)
        | c::Delivery::Proposal(_)
        | c::Delivery::Chips(_)
        | c::Delivery::Settings(_)
        | c::Delivery::Deployment(_)
        | c::Delivery::Terminal(_)
        | c::Delivery::Accounts(_) => unreachable!("wait subject"),
    }
    if let Some(result) = wait.result {
        super::result_text(result, vocabulary, out);
    }
    if let c::Delivery::Task(task) = delivery {
        vocabulary.task_next(task, out);
    }
}
