//! Dependency links and amendment facts (client/tools.md, 6.1 and 6.2).
use super::Text;
use jig_client::{contract as c, words::Word};
use skein_json::writer::Encoder;
use skein_ui::phrases::Phrasebook;

pub(super) fn text(task: &c::Task, me: Option<&c::Me>, book: &Phrasebook, out: &mut Text) {
    if task.dependencies.is_empty() {
        return;
    }
    out.put(b"\ndependencies:");
    for link in &task.dependencies {
        out.put(b"\n  task ");
        let number = match link {
            c::OwnerTaskLink::Live(chip) | c::OwnerTaskLink::Retained { chip, .. } => {
                out.number(chip.task);
                out.put(b" \xc2\xb7 ");
                out.put(&book.say(0, Word::Phase(chip.phase).id(), &[]).expect("phase phrase").text);
                chip.task
            }
            c::OwnerTaskLink::Stored { number, status } => {
                out.number(*number);
                if let Some(status) = status {
                    out.put(b" \xc2\xb7 ");
                    out.put(match status {
                        c::OwnerTaskStatus::Done => b"Completed",
                        c::OwnerTaskStatus::Failed => b"Failed",
                        c::OwnerTaskStatus::Cancelled => b"Cancelled",
                    });
                }
                *number
            }
        };
        for fact in &task.dependency_facts {
            if fact.number != number {
                continue;
            }
            if let Some(added) = fact.added {
                out.put(b" \xc2\xb7 added by ");
                super::requester(added.by, me, out);
                out.put(b" at ");
                super::clock::deadline(added.at, out);
            }
            if fact.keeps {
                out.put(b" \xc2\xb7 keeps task ");
                out.number(task.chip.task);
                out.put(b" from ending");
            }
            break;
        }
    }
}

pub(super) fn encode(value: &c::Dependency, out: &mut Encoder, budget: &mut super::Cut) {
    out.object_start();
    out.key(b"number");
    out.unsigned(value.number);
    out.key(b"pending");
    out.boolean(value.pending);
    out.key(b"added");
    match value.added {
        Some(added) => {
            out.object_start();
            out.key(b"at");
            out.unsigned(added.at.as_nanos());
            out.key(b"by");
            super::encode_owner_requester(&added.by, out, budget);
            out.object_end();
        }
        None => out.null(),
    }
    out.key(b"keeps");
    out.boolean(value.keeps);
    out.key(b"status");
    match value.status {
        Some(status) => super::owner_links::encode_owner_task_status(status, out, budget),
        None => out.null(),
    }
    out.object_end();
}
