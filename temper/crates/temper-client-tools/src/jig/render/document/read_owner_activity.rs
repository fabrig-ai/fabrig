//! Integer JSON for supplied read facts.
use skein_json::writer::Encoder;
#[expect(clippy::too_many_lines, reason = "one exhaustive encoding of jig's owner activity vocabulary")]
pub(super) fn encode_owner_activity(
    value: &jig_client::contract::Activity,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    match value {
        jig_client::contract::Activity::Words { message, by, kind, words } => {
            out.object_start();
            out.key(b"words");
            out.object_start();
            out.key(b"message");
            out.unsigned(*(message));
            out.key(b"by");
            super::encode_owner_requester(by, out, budget);
            out.key(b"kind");
            super::encode_owner_message_kind(kind, out, budget);
            out.key(b"words");
            budget.words((words).bytes(), out);
            out.object_end();
            out.object_end();
        }
        jig_client::contract::Activity::Heard(field_0) => {
            out.object_start();
            out.key(b"heard");
            encode_owner_word(field_0, out, budget);
            out.object_end();
        }
        jig_client::contract::Activity::Notified { recipient, word } => {
            out.object_start();
            out.key(b"notified");
            out.object_start();
            out.key(b"recipient");
            super::encode_owner_requester(recipient, out, budget);
            out.key(b"word");
            encode_owner_word(word, out, budget);
            out.object_end();
            out.object_end();
        }
        jig_client::contract::Activity::RunBegan { attempt, why, resumed } => {
            out.object_start();
            out.key(b"run_began");
            out.object_start();
            out.key(b"attempt");
            out.unsigned(*(attempt));
            out.key(b"why");
            encode_owner_run_why(why, out, budget);
            out.key(b"resumed");
            out.boolean(*(resumed));
            out.object_end();
            out.object_end();
        }
        jig_client::contract::Activity::RunEnded { attempt, end, spent } => {
            out.object_start();
            out.key(b"run_ended");
            out.object_start();
            out.key(b"attempt");
            out.unsigned(*(attempt));
            out.key(b"end");
            encode_owner_run_end(*end, out, budget);
            out.key(b"spent");
            out.unsigned(*(spent));
            out.object_end();
            out.object_end();
        }
        jig_client::contract::Activity::Turn { reply, attempt, turn, read } => {
            out.object_start();
            out.key(b"turn");
            out.object_start();
            out.key(b"reply");
            out.unsigned(*(reply));
            out.key(b"attempt");
            out.unsigned(*(attempt));
            out.key(b"turn");
            out.unsigned(u64::from(*(turn)));
            out.key(b"read");
            match read {
                Some(item_1) => {
                    out.unsigned(*(item_1));
                }
                None => out.null(),
            }
            out.object_end();
            out.object_end();
        }
        jig_client::contract::Activity::Decided { revision, .. } => {
            out.object_start();
            out.key(b"decided");
            out.object_start();
            out.key(b"revision");
            out.unsigned(*(revision));
            out.object_end();
            out.object_end();
        }
        jig_client::contract::Activity::Stood { attempt, words } => {
            out.object_start();
            out.key(b"stood");
            out.object_start();
            out.key(b"attempt");
            out.unsigned(*attempt);
            out.key(b"words");
            out.string(words.bytes());
            out.object_end();
            out.object_end();
        }
        jig_client::contract::Activity::SessionEnded { attempt, end } => {
            out.object_start();
            out.key(b"session_ended");
            out.object_start();
            out.key(b"attempt");
            out.unsigned(*attempt);
            out.key(b"end");
            out.string(match end {
                jig_client::contract::SessionEnd::Quiet => b"quiet",
                jig_client::contract::SessionEnd::Bound => b"bound",
                jig_client::contract::SessionEnd::Unoffered => b"unoffered",
                jig_client::contract::SessionEnd::Fresh => b"fresh",
                jig_client::contract::SessionEnd::Closed => b"closed",
                jig_client::contract::SessionEnd::Rules => b"rules",
            });
            out.object_end();
            out.object_end();
        }
        jig_client::contract::Activity::Kept { attempt, pending, .. } => {
            out.object_start();
            out.key(b"kept");
            out.object_start();
            out.key(b"attempt");
            out.unsigned(*attempt);
            out.key(b"pending");
            out.array_start();
            for task in pending {
                out.unsigned(*task);
            }
            out.array_end();
            out.object_end();
            out.object_end();
        }
        jig_client::contract::Activity::Milestone { position } => {
            out.object_start();
            out.key(b"milestone");
            out.object_start();
            out.key(b"position");
            out.unsigned(*(position));
            out.object_end();
            out.object_end();
        }
    }
}

pub(super) fn encode_owner_activity_line(
    value: &jig_client::contract::ActivityLine,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    out.object_start();
    out.key(b"task");
    out.unsigned(value.task);
    out.key(b"position");
    out.unsigned(value.position);
    out.key(b"at");
    out.unsigned(value.at.as_nanos());
    out.key(b"line");
    encode_owner_activity(&value.line, out, budget);
    out.object_end();
}

pub(super) fn encode_owner_run_end(value: jig_client::contract::RunEnd, out: &mut Encoder, budget: &mut super::Cut) {
    let value = &value;
    match value {
        jig_client::contract::RunEnd::Parked => out.string(b"parked"),
        jig_client::contract::RunEnd::Finished => out.string(b"finished"),
        jig_client::contract::RunEnd::Cancelled => out.string(b"cancelled"),
        jig_client::contract::RunEnd::Failed { class, invalid_reason } => {
            out.object_start();
            out.key(b"failed");
            out.object_start();
            out.key(b"class");
            super::encode_hold_failure(*(class), out, budget);
            encode_invalid_reason(*invalid_reason, out);
            out.object_end();
            out.object_end();
        }
        jig_client::contract::RunEnd::Refused => out.string(b"refused"),
    }
}

pub(super) fn encode_owner_run_why(value: &jig_client::contract::RunWhy, out: &mut Encoder, budget: &mut super::Cut) {
    match value {
        jig_client::contract::RunWhy::Made => out.string(b"made"),
        jig_client::contract::RunWhy::Ready => out.string(b"ready"),
        jig_client::contract::RunWhy::Message { message } => {
            out.object_start();
            out.key(b"message");
            out.object_start();
            out.key(b"message");
            out.unsigned(*(message));
            out.object_end();
            out.object_end();
        }
        jig_client::contract::RunWhy::Released => out.string(b"released"),
        jig_client::contract::RunWhy::Retry { tries } => {
            out.object_start();
            out.key(b"retry");
            out.object_start();
            out.key(b"tries");
            super::encode_owner_failure_tries(tries, out, budget);
            out.object_end();
            out.object_end();
        }
        jig_client::contract::RunWhy::Narrowed => out.string(b"narrowed"),
        jig_client::contract::RunWhy::Changed => out.string(b"changed"),
    }
}

pub(super) fn encode_owner_word(value: &jig_client::contract::Word, out: &mut Encoder, budget: &mut super::Cut) {
    out.object_start();
    out.key(b"number");
    out.unsigned(value.number);
    out.key(b"from");
    super::encode_owner_requester(&value.from, out, budget);
    out.key(b"kind");
    super::encode_owner_message_kind(&value.kind, out, budget);
    out.key(b"words");
    budget.words(value.words.bytes(), out);
    out.key(b"at");
    out.unsigned(value.at.as_nanos());
    out.key(b"hits");
    out.unsigned(u64::from(value.hits));
    out.key(b"eligible");
    out.boolean(value.eligible);
    out.object_end();
}

fn encode_invalid_reason(value: Option<jig_client::contract::InvalidReason>, out: &mut Encoder) {
    let Some(reason) = value else { return };
    out.key(b"invalid_reason");
    out.string(match reason {
        jig_client::contract::InvalidReason::Form => b"form",
        jig_client::contract::InvalidReason::Verdict => b"verdict",
        jig_client::contract::InvalidReason::Words => b"words",
        jig_client::contract::InvalidReason::Change => b"change",
        jig_client::contract::InvalidReason::Followups => b"followups",
        jig_client::contract::InvalidReason::UnreadableResult => b"unreadable result",
    });
}
