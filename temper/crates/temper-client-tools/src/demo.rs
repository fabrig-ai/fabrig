//! The playground's answers and a forge's removal (client/tools.md, 5.5, area 6).
use crate::{
    jig::{
        self,
        render::{self, Text},
    },
    root::Limits,
};
use alloc::boxed::Box;
use jig_client::Outcome;
use skein_json::writer::Encoder;
use temper_client_domain::forge::{
    demo::{self, StartAnswer, StartFailure},
    words,
};

fn text(answer: &StartAnswer, kept: bool, limits: &Limits, out: &mut Text) {
    if kept {
        out.put(b"kept: demo-start ");
    }
    match answer {
        StartAnswer::EntranceRefused(why) => {
            render::acts::refusal(why.refusal(), &render::document::phrasebook(limits.tools.text), out);
            out.put(b"\n");
        }
        StartAnswer::Running => out.put(b"running\n"),
        StartAnswer::NotOffered => out.put(b"refused: not offered\n"),
        StartAnswer::Unknown => out.put(b"unknown\n"),
        StartAnswer::Failed(why) => {
            match why {
                StartFailure::Group(why) => {
                    out.put(b"the fakes did not start (");
                    out.put(&words::lossy(why));
                    out.put(b")");
                }
                StartFailure::Account => out.put(b"the account refused"),
                StartFailure::Adoption => out.put(b"the adoption refused"),
                StartFailure::Project => out.put(b"the playground's project was refused"),
                StartFailure::NotConnected => out.put(b"the forge not connected in time"),
            }
            out.put(b"\n");
        }
    }
}
fn json(answer: &StartAnswer, kept: bool, out: &mut Encoder) {
    if kept {
        out.object_start();
        out.key(b"operation");
        out.string(b"demo-start");
        out.key(b"outcome");
    }
    out.object_start();
    out.key(match answer {
        StartAnswer::EntranceRefused(_) => b"entrance_refused",
        StartAnswer::Running => b"running",
        StartAnswer::NotOffered => b"start_refused",
        StartAnswer::Failed(_) => b"start_failed",
        StartAnswer::Unknown => b"unknown",
    });
    out.object_start();
    match answer {
        StartAnswer::EntranceRefused(why) => {
            out.key(b"refusal");
            render::document::refusal_json(why.refusal(), out);
        }
        StartAnswer::NotOffered => {
            out.key(b"why");
            out.string(b"not_offered");
        }
        StartAnswer::Failed(why) => {
            out.key(b"why");
            match why {
                StartFailure::Group(why) => {
                    out.object_start();
                    out.key(b"group");
                    out.object_start();
                    out.key(b"why");
                    out.string(&words::lossy(why));
                    out.object_end();
                    out.object_end();
                }
                StartFailure::Account => out.string(b"account"),
                StartFailure::Adoption => out.string(b"adoption"),
                StartFailure::Project => out.string(b"project"),
                StartFailure::NotConnected => out.string(b"not_connected"),
            }
        }
        StartAnswer::Running | StartAnswer::Unknown => {}
    }
    out.object_end();
    out.object_end();
    if kept {
        out.object_end();
    }
}
pub(crate) fn print(outcome: &Outcome, as_json: bool, kept: bool, limits: &Limits) -> jig::Printed {
    let answer = match outcome {
        Outcome::Full { held, room_at } => {
            return render::answers_full(crate::root::DIAGNOSTIC_PREFIX, *held, *room_at);
        }
        Outcome::Answered(answer) => demo::answer(answer, &limits.domain.forge),
        Outcome::Unknown => Some(StartAnswer::Unknown),
        Outcome::Held | Outcome::Limit | Outcome::Unconfirmed | Outcome::NotKept | Outcome::Unsent | Outcome::Lost => {
            let mut operation =
                render::outcome::Operation::of(&demo::ask(&limits.domain.forge).expect("checked limits"));
            operation.word = b"demo-start";
            if kept {
                return jig::Printed {
                    file: jig::FileEnd::Keep,
                    status: 1,
                    stdout: Box::default(),
                    stderr: render::outcome::kept(
                        &operation,
                        outcome,
                        as_json,
                        None,
                        &limits.tools,
                        crate::root::DIAGNOSTIC_PREFIX,
                    ),
                };
            }
            return render::outcome::print(
                &operation,
                outcome,
                as_json,
                None,
                &limits.tools,
                crate::root::DIAGNOSTIC_PREFIX,
            );
        }
    };
    let Some(answer) = answer else { return render::failure(b"demo start answer decoding failed", as_json, 1) };
    let status = match answer {
        StartAnswer::Running => 0,
        StartAnswer::NotOffered | StartAnswer::EntranceRefused(_) => 2,
        StartAnswer::Failed(_) | StartAnswer::Unknown => 1,
    };
    let bytes = if as_json {
        let bounds = skein_json::writer::Limits { depth: 8, length: limits.tools.text };
        let mut measure = Encoder::measure(&bounds);
        json(&answer, kept, &mut measure);
        let Ok(size) = measure.measured() else { return render::failure(b"result limit", true, 1) };
        let mut out = Encoder::write(size, &bounds);
        json(&answer, kept, &mut out);
        let bytes = out.finish();
        if kept { render::outcome::newline(&bytes) } else { bytes }
    } else {
        let mut measure = Text::measure();
        text(&answer, kept, limits, &mut measure);
        let mut out = Text::writing_exact(measure);
        text(&answer, kept, limits, &mut out);
        out.finish()
    };
    if bytes.len() > usize::try_from(limits.tools.text).expect("text fits") {
        return render::failure(b"result limit", as_json, 1);
    }
    if status == 0 {
        jig::Printed { file: jig::FileEnd::Keep, status, stdout: bytes, stderr: Box::default() }
    } else {
        jig::Printed { file: jig::FileEnd::Keep, status, stdout: Box::default(), stderr: bytes }
    }
}

// The ctl result preserves refusals that a page can treat as already removed.
#[derive(Clone, Copy, Debug)]
enum RemovalAnswer {
    Reset,
    Removing,
    NotOffered,
    NoSuchForge,
    Role,
    Unknown,
}
impl RemovalAnswer {
    fn fields(self, reset: bool) -> (&'static [u8], Option<&'static [u8]>, &'static [u8], u8) {
        match self {
            Self::Reset => (b"reset", None, b"reset\n", 0),
            Self::Removing => (b"removing", None, b"removing\n", 0),
            Self::Unknown => (b"unknown", None, b"unknown\n", 1),
            Self::NotOffered => (
                if reset { b"end_refused" } else { b"remove_refused" },
                Some(b"not_offered"),
                b"refused: not offered\n",
                2,
            ),
            Self::NoSuchForge => (b"remove_refused", Some(b"no_such_forge"), b"refused: no such forge\n", 2),
            Self::Role => (b"remove_refused", Some(b"role"), b"refused: not an owner\n", 2),
        }
    }
}
pub(crate) fn print_reset(outcome: &Outcome, as_json: bool, kept: bool, limits: &Limits) -> jig::Printed {
    let answer = match outcome {
        Outcome::Answered(value) => match demo::end_answer(value, &limits.domain.forge) {
            Some(demo::EndAnswer::Reset) => RemovalAnswer::Reset,
            Some(demo::EndAnswer::NotOffered) => RemovalAnswer::NotOffered,
            Some(demo::EndAnswer::EntranceRefused(why))
                if why.refusal() == &jig_client::contract::Refusal::NotOffered =>
            {
                RemovalAnswer::NotOffered
            }
            Some(demo::EndAnswer::Unknown | demo::EndAnswer::EntranceRefused(_)) | None => RemovalAnswer::Unknown,
        },
        Outcome::NotKept
        | Outcome::Unknown
        | Outcome::Limit
        | Outcome::Unconfirmed
        | Outcome::Held
        | Outcome::Unsent
        | Outcome::Full { .. }
        | Outcome::Lost => RemovalAnswer::Unknown,
    };
    print_removal(outcome, answer, &crate::forges::PLAYGROUND_RESET, as_json, kept, limits)
}
pub(crate) fn print_remove(outcome: &Outcome, as_json: bool, kept: bool, limits: &Limits) -> jig::Printed {
    use temper_client_domain::forge::remove::{self, Answer, RemoveRefusal};
    let answer = match outcome {
        Outcome::Answered(value) => {
            if no_such_forge(value, limits) {
                RemovalAnswer::NoSuchForge
            } else {
                match remove::answer(value, &limits.domain.forge) {
                    Some(Answer::Removing) => RemovalAnswer::Removing,
                    Some(Answer::Refused(RemoveRefusal::NotOffered)) => RemovalAnswer::NotOffered,
                    Some(Answer::Refused(RemoveRefusal::Role)) => RemovalAnswer::Role,
                    Some(Answer::Unknown) | None => RemovalAnswer::Unknown,
                }
            }
        }
        Outcome::NotKept
        | Outcome::Unknown
        | Outcome::Limit
        | Outcome::Unconfirmed
        | Outcome::Held
        | Outcome::Unsent
        | Outcome::Full { .. }
        | Outcome::Lost => RemovalAnswer::Unknown,
    };
    print_removal(outcome, answer, &crate::forges::REMOVE, as_json, kept, limits)
}
fn no_such_forge(value: &jig_client::Answered, limits: &Limits) -> bool {
    use temper_client_protocol_wire::{self as wire, v1 as w};
    let jig_client::Answered::Family(record) = value else { return false };
    if record.family != wire::FORGE || record.version != wire::CLIENT_VERSION {
        return false;
    }
    let Some(limits) = temper_client_domain::forge::wire_limits(&limits.domain.forge) else { return false };
    let mut reader = skein_lib::Reader::new(&record.bytes);
    let Ok(answer) = w::Answer::decode(&limits, &mut reader) else { return false };
    if !reader.is_empty() {
        return false;
    }
    let w::Outcome::RemoveRefused(value) = answer.into_parts().outcome else { return false };
    value.into_parts().why == w::RemoveRefusal::NoSuchForge
}
fn removal_json(answer: RemovalAnswer, tool: &jig::table::Tool, kept: bool, out: &mut Encoder) {
    if kept {
        out.object_start();
        out.key(b"operation");
        out.string(tool.result);
        out.key(b"outcome");
    }
    let (tag, why, _, _) = answer.fields(tool.name == crate::forges::PLAYGROUND_RESET.name);
    out.object_start();
    out.key(tag);
    out.object_start();
    if let Some(why) = why {
        out.key(b"why");
        out.string(why);
    }
    out.object_end();
    out.object_end();
    if kept {
        out.object_end();
    }
}
fn print_removal(
    outcome: &Outcome,
    answer: RemovalAnswer,
    tool: &'static jig::table::Tool,
    as_json: bool,
    kept: bool,
    limits: &Limits,
) -> jig::Printed {
    match outcome {
        Outcome::Full { held, room_at } => {
            return render::answers_full(crate::root::DIAGNOSTIC_PREFIX, *held, *room_at);
        }
        Outcome::Held | Outcome::Limit | Outcome::Unconfirmed | Outcome::NotKept | Outcome::Unsent | Outcome::Lost => {
            let request = if tool.name == crate::forges::PLAYGROUND_RESET.name {
                demo::end_ask(&limits.domain.forge)
            } else {
                temper_client_domain::forge::remove::ask(1, &limits.domain.forge)
            }
            .expect("checked limits");
            let mut operation = render::outcome::Operation::of(&request);
            operation.word = tool.result;
            if kept {
                return jig::Printed {
                    file: jig::FileEnd::Keep,
                    status: 1,
                    stdout: Box::default(),
                    stderr: render::outcome::kept(
                        &operation,
                        outcome,
                        as_json,
                        None,
                        &limits.tools,
                        crate::root::DIAGNOSTIC_PREFIX,
                    ),
                };
            }
            return render::outcome::print(
                &operation,
                outcome,
                as_json,
                None,
                &limits.tools,
                crate::root::DIAGNOSTIC_PREFIX,
            );
        }
        Outcome::Answered(_) | Outcome::Unknown => {}
    }
    let (_, _, words, status) = answer.fields(tool.name == crate::forges::PLAYGROUND_RESET.name);
    let bytes = if as_json {
        let bounds = skein_json::writer::Limits { depth: 8, length: limits.tools.text };
        let mut measure = Encoder::measure(&bounds);
        removal_json(answer, tool, kept, &mut measure);
        let Ok(size) = measure.measured() else { return render::failure(b"result limit", true, 1) };
        let mut out = Encoder::write(size, &bounds);
        removal_json(answer, tool, kept, &mut out);
        let bytes = out.finish();
        if kept { render::outcome::newline(&bytes) } else { bytes }
    } else {
        let mut measure = Text::measure();
        removal_text(tool, words, kept, &mut measure);
        let mut out = Text::writing_exact(measure);
        removal_text(tool, words, kept, &mut out);
        out.finish()
    };
    if bytes.len() > usize::try_from(limits.tools.text).expect("text fits") {
        return render::failure(b"result limit", as_json, 1);
    }
    if status == 0 {
        jig::Printed { file: jig::FileEnd::Keep, status, stdout: bytes, stderr: Box::default() }
    } else {
        jig::Printed { file: jig::FileEnd::Keep, status, stdout: Box::default(), stderr: bytes }
    }
}
fn removal_text(tool: &jig::table::Tool, words: &[u8], kept: bool, out: &mut Text) {
    if kept {
        out.put(b"kept: ");
        out.put(tool.result);
        out.put(b" ");
    }
    out.put(words);
}
