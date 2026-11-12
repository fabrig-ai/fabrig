//! Findings (temper/docs/design/protocol/store.md, 2).
use super::{FITS, w};
use crate::Undecodable;
use alloc::boxed::Box;
use skein_lib::Wall;
use temper_engine_domain_forge as d;
fn encode_holder(value: d::Holder) -> w::FindingHolder {
    match value {
        d::Holder::Task(number) => w::FindingHolder::Task(w::Number::new(&w::CEILINGS, w::NumberParts { number }).expect(FITS)),
        d::Holder::Party(number) => w::FindingHolder::Party(w::Number::new(&w::CEILINGS, w::NumberParts { number }).expect(FITS)),
        d::Holder::Role(number) => w::FindingHolder::Role(w::FindingCount::new(&w::CEILINGS, w::FindingCountParts { number }).expect(FITS)),
    }
}
fn decode_holder(value: w::FindingHolder) -> d::Holder {
    match value { w::FindingHolder::Task(value) => d::Holder::Task(value.number()), w::FindingHolder::Party(value) => d::Holder::Party(value.number()), w::FindingHolder::Role(value) => d::Holder::Role(value.number()) }
}
fn encode_refusal(value: &d::FindingRefusal) -> w::FindingRefusal {
    match value {
        d::FindingRefusal::NoArea(names) => {
            let mut listed = skein_lib::List::with_capacity(16);
            for name in names { listed.push(name.clone()).expect(FITS); }
            w::FindingRefusal::NoArea(w::FindingAreas::new(&w::CEILINGS, w::FindingAreasParts { names: listed }).expect(FITS))
        }
        d::FindingRefusal::Full(number) => w::FindingRefusal::Full(w::FindingCount::new(&w::CEILINGS, w::FindingCountParts { number: *number }).expect(FITS)),
        d::FindingRefusal::NotYours(holder) => w::FindingRefusal::NotYours(w::FindingHeld::new(&w::CEILINGS, w::FindingHeldParts { holder: encode_holder(*holder) }).expect(FITS)),
        d::FindingRefusal::NoFinding => w::FindingRefusal::NoFinding,
        d::FindingRefusal::NoGoal => w::FindingRefusal::NoGoal,
        d::FindingRefusal::Ended => w::FindingRefusal::Ended,
        d::FindingRefusal::Words(words) => w::FindingRefusal::Words(w::FindingWords::new(&w::CEILINGS, w::FindingWordsParts { words: words.clone() }).expect(FITS)),
    }
}
fn decode_refusal(value: w::FindingRefusal) -> d::FindingRefusal {
    match value {
        w::FindingRefusal::NoArea(value) => d::FindingRefusal::NoArea(value.into_parts().names.into_boxed()),
        w::FindingRefusal::Full(value) => d::FindingRefusal::Full(value.number()),
        w::FindingRefusal::NotYours(value) => d::FindingRefusal::NotYours(decode_holder(value.into_parts().holder)),
        w::FindingRefusal::NoFinding => d::FindingRefusal::NoFinding,
        w::FindingRefusal::NoGoal => d::FindingRefusal::NoGoal,
        w::FindingRefusal::Ended => d::FindingRefusal::Ended,
        w::FindingRefusal::Words(value) => d::FindingRefusal::Words(value.into_parts().words),
    }
}
pub(super) fn encode_reply(reply: &d::FindingReply) -> w::FindingReply {
    let result = match &reply.result {
        Ok(d::FindingDone::Filed(holder)) => w::FindingResult::Filed(w::FindingHeld::new(&w::CEILINGS, w::FindingHeldParts { holder: encode_holder(*holder) }).expect(FITS)),
        Ok(d::FindingDone::Handed(holder)) => w::FindingResult::Handed(w::FindingHeld::new(&w::CEILINGS, w::FindingHeldParts { holder: encode_holder(*holder) }).expect(FITS)),
        Ok(d::FindingDone::Routed(number)) => w::FindingResult::Routed(w::Number::new(&w::CEILINGS, w::NumberParts { number: *number }).expect(FITS)),
        Ok(d::FindingDone::Backlog) => w::FindingResult::Backlog,
        Ok(d::FindingDone::Dropped) => w::FindingResult::Dropped,
        Err(why) => w::FindingResult::Refused(w::FindingRefused::new(&w::CEILINGS, w::FindingRefusedParts { why: encode_refusal(why) }).expect(FITS)),
    };
    w::FindingReply::new(&w::CEILINGS, w::FindingReplyParts { project: reply.project, number: reply.number, area: reply.area.clone(), result }).expect(FITS)
}
pub(super) fn decode_reply(reply: w::FindingReply) -> d::FindingReply {
    let w::FindingReplyParts { project, number, area, result } = reply.into_parts();
    let result = match result {
        w::FindingResult::Filed(value) => Ok(d::FindingDone::Filed(decode_holder(value.into_parts().holder))),
        w::FindingResult::Handed(value) => Ok(d::FindingDone::Handed(decode_holder(value.into_parts().holder))),
        w::FindingResult::Routed(value) => Ok(d::FindingDone::Routed(value.number())),
        w::FindingResult::Backlog => Ok(d::FindingDone::Backlog),
        w::FindingResult::Dropped => Ok(d::FindingDone::Dropped),
        w::FindingResult::Refused(value) => Err(decode_refusal(value.into_parts().why)),
    };
    d::FindingReply { project, number, area, result }
}

fn encode_findingkind(value: d::FindingKind) -> w::FindingKind {
    match value {
        d::FindingKind::Qa => w::FindingKind::Qa,
        d::FindingKind::Flake => w::FindingKind::Flake,
        d::FindingKind::Red => w::FindingKind::Red,
        d::FindingKind::Review => w::FindingKind::Review,
        d::FindingKind::Feedback => w::FindingKind::Feedback,
    }
}
fn decode_findingkind(value: w::FindingKind) -> d::FindingKind {
    match value {
        w::FindingKind::Qa => d::FindingKind::Qa,
        w::FindingKind::Flake => d::FindingKind::Flake,
        w::FindingKind::Red => d::FindingKind::Red,
        w::FindingKind::Review => d::FindingKind::Review,
        w::FindingKind::Feedback => d::FindingKind::Feedback,
    }
}
fn encode_severity(value: d::Severity) -> w::Severity {
    match value {
        d::Severity::Low => w::Severity::Low,
        d::Severity::Medium => w::Severity::Medium,
        d::Severity::High => w::Severity::High,
    }
}
fn decode_severity(value: w::Severity) -> d::Severity {
    match value {
        w::Severity::Low => d::Severity::Low,
        w::Severity::Medium => d::Severity::Medium,
        w::Severity::High => d::Severity::High,
    }
}
fn encode_findingstate(value: d::FindingState) -> w::FindingState {
    match value {
        d::FindingState::New => w::FindingState::New,
        d::FindingState::Routed => w::FindingState::Routed,
        d::FindingState::Backlog => w::FindingState::Backlog,
        d::FindingState::Dropped => w::FindingState::Dropped,
    }
}
fn decode_findingstate(value: w::FindingState) -> d::FindingState {
    match value {
        w::FindingState::New => d::FindingState::New,
        w::FindingState::Routed => d::FindingState::Routed,
        w::FindingState::Backlog => d::FindingState::Backlog,
        w::FindingState::Dropped => d::FindingState::Dropped,
    }
}
fn encode_filer(value: d::Filer) -> w::Filer {
    match value {
        d::Filer::Task(number) => w::Filer::Task(w::Number::new(&w::CEILINGS, w::NumberParts { number }).expect(FITS)),
        d::Filer::Party(number) => {
            w::Filer::Party(w::Number::new(&w::CEILINGS, w::NumberParts { number }).expect(FITS))
        }
    }
}
fn decode_filer(value: w::Filer) -> d::Filer {
    match value {
        w::Filer::Task(value) => d::Filer::Task(value.number()),
        w::Filer::Party(value) => d::Filer::Party(value.number()),
    }
}
pub(super) fn encode(row: &d::Finding) -> w::Finding {
    w::Finding::new(
        &w::CEILINGS,
        w::FindingParts {
            project: row.project,
            number: row.number,
            kind: encode_findingkind(row.kind),
            severity: encode_severity(row.severity),
            area: row.area.clone(),
            state: encode_findingstate(row.state),
            title: row.title.clone(),
            body: row.body.clone(),
            filer: encode_filer(row.filer),
            at: row.at.as_nanos(),
            why: row.why.clone(),
            hander: match row.hander {
                Some(value) => Some(encode_filer(value)),
                None => None,
            },
            goal: row.goal,
            reason: row.reason.clone(),
        },
    )
    .expect(FITS)
}
pub(super) fn decode(row: w::Finding) -> Result<d::Finding, Undecodable> {
    let w::FindingParts {
        project,
        number,
        kind,
        severity,
        area,
        state,
        title,
        body,
        filer,
        at,
        why,
        hander,
        goal,
        reason,
    } = row.into_parts();
    if number == 0 || !d::settings::words_fit(&title, 200) || !d::findings::body_fits(&body) {
        return Err(Undecodable);
    }
    if let Some(words) = &why
        && !d::settings::words_fit(words, 200)
    {
        return Err(Undecodable);
    }
    if let Some(words) = &reason
        && !d::findings::reason_fits(words)
    {
        return Err(Undecodable);
    }
    if why.is_some() != hander.is_some() {
        return Err(Undecodable);
    }
    Ok(d::Finding {
        project,
        number,
        kind: decode_findingkind(kind),
        severity: decode_severity(severity),
        area,
        state: decode_findingstate(state),
        title,
        body,
        filer: decode_filer(filer),
        at: Wall::from_nanos(at),
        why,
        hander: match hander {
            Some(value) => Some(decode_filer(value)),
            None => None,
        },
        goal,
        reason,
    })
}
