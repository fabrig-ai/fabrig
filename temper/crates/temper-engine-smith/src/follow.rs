//! The role's follow tool (temper/docs/design/domain/forge/follows.md, 5).
use crate::{Problem, calls::Spec, json::Value};
use skein_lib::List;
use temper_engine_domain::FollowCall;
use temper_engine_domain_forge::follows::FollowAct;

pub(crate) const SPEC: Spec = Spec {
    name: b"follow",
    description: b"start: follow commit; hold: follow commit reason; skip: follow commit reason; merge: follow commits (two or more, queue order); list: no members; show: follow. Act on your own follows; list and show read your project's follows.",
    schema: br#"{"type":"object","additionalProperties":false,"properties":{"act":{"enum":["start","hold","skip","merge","list","show"]},"follow":{"type":"integer","minimum":1,"maximum":4294967295},"commit":{"type":"string","pattern":"^([0-9a-f]{40}|[0-9a-f]{64})$"},"commits":{"type":"array","minItems":2,"items":{"type":"string","pattern":"^([0-9a-f]{40}|[0-9a-f]{64})$"}},"reason":{"type":"string","minLength":1}},"required":["act"],"oneOf":[{"properties":{"act":{"enum":["start"]}},"required":["follow","commit"]},{"properties":{"act":{"enum":["hold","skip"]}},"required":["follow","commit","reason"]},{"properties":{"act":{"enum":["merge"]}},"required":["follow","commits"]},{"properties":{"act":{"enum":["list"]}}},{"properties":{"act":{"enum":["show"]}},"required":["follow"]}]}"#,
    // One declaration covers writes and reads; the root serves list/show without a durable call.
    effect: smith_domain_run::HostEffect::Write,
    offer: crate::calls::Offer::Delegate,
};

pub(crate) fn decode(value: &Value) -> Result<FollowCall, Problem> {
    let act = value.required(b"act")?.text(b"act")?;
    let fields: &[&[u8]] = match act.as_ref() {
        b"start" => &[b"act", b"follow", b"commit"],
        b"hold" | b"skip" => &[b"act", b"follow", b"commit", b"reason"],
        b"merge" => &[b"act", b"follow", b"commits"],
        b"list" => &[b"act"],
        b"show" => &[b"act", b"follow"],
        _ => return Err(Problem::Range(b"act: start, hold, skip, merge, list or show")),
    };
    let Value::Object(members) = value else { return Err(Problem::Type(b"follow")) };
    for (name, _) in members {
        if !fields.contains(&name.as_ref()) {
            return Err(Problem::Range(b"follow: member does not belong to this act"));
        }
    }
    if act.as_ref() == b"list" {
        return Ok(FollowCall::List);
    }
    let follow = value.required(b"follow")?.small(b"follow")?;
    if follow == 0 {
        return Err(Problem::Range(b"follow"));
    }
    let act = match act.as_ref() {
        b"show" => return Ok(FollowCall::Show { follow }),
        b"start" => FollowAct::Start { after: commit(value.required(b"commit")?)? },
        b"hold" | b"skip" => {
            let after = commit(value.required(b"commit")?)?;
            let reason = value.required(b"reason")?.text(b"reason")?;
            if reason.is_empty() {
                return Err(Problem::Range(b"reason: nonempty"));
            }
            if act.as_ref() == b"hold" { FollowAct::Hold { after, reason } } else { FollowAct::Skip { after, reason } }
        }
        b"merge" => {
            let values = value.required(b"commits")?.array(b"commits")?;
            if values.len() < 2 {
                return Err(Problem::Range(b"commits: at least two"));
            }
            let mut afters = List::uncapped();
            for value in values {
                afters.push(commit(value)?).expect("input bounded commits");
            }
            FollowAct::Merge { afters: afters.into_boxed() }
        }
        _ => unreachable!("act checked above"),
    };
    Ok(FollowCall::Act { follow, act })
}
fn commit(value: &Value) -> Result<skein_forge::api::Commit, Problem> {
    crate::forge::commit(value, b"commit")
}
mod render;
pub(crate) use render::render;
