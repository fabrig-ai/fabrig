//! Optional gap records (qa/ledger.md, 2.11).
use crate::document::{self as d, integer, optional_number, text};
use crate::{Gap, GapClosed, GapState, Refusal, refused};
use serde_json::{Value, json};

pub(crate) fn document(gap: &Gap) -> Value {
    json!({"number":gap.number,"kind":gap.kind,"why":gap.why,"owner":gap.owner,"landing":gap.landing,
        "case":gap.case,"session":gap.session,"time":gap.time,"state":gap.state.name(),"filed":gap.filed,
        "closed":gap.closed.as_ref().map(|closed| json!({"landing":closed.landing,"time":closed.time}))})
}
pub(crate) fn gap(value: &Value) -> Result<Gap, Refusal> {
    let object = d::object(
        value,
        &["number", "kind", "why", "owner", "landing", "case", "session", "time", "state", "filed", "closed"],
    )?;
    let kind = text(object, "kind")?;
    if !["ctl", "fake", "qa"].contains(&kind.as_str()) {
        return Err(refused(format!("unknown gap kind {kind}")));
    }
    let state = match text(object, "state")?.as_str() {
        "open" => GapState::Open,
        "filed" => GapState::Filed,
        "closed" => GapState::Closed,
        word => return Err(refused(format!("unknown gap state {word}"))),
    };
    let closed = object
        .get("closed")
        .map(|value| {
            let object = d::object(value, &["landing", "time"])?;
            Ok(GapClosed { landing: integer(object, "landing")?, time: integer(object, "time")? })
        })
        .transpose()?;
    Ok(Gap {
        number: integer(object, "number")?,
        kind,
        why: text(object, "why")?,
        owner: text(object, "owner")?,
        landing: integer(object, "landing")?,
        case: optional_number(object, "case")?,
        session: integer(object, "session")?,
        time: integer(object, "time")?,
        state,
        filed: optional_number(object, "filed")?,
        closed,
    })
}
