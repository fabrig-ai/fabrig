//! The same receiving admission as the client's link (protocol/client.md, 3 and 13).
use super::{c, encoded, values, w};
use crate::{Bounds, Invalid};
use jig_client_protocol_link::{Limits, translate};
use skein_lib::{Duration, Reader};

pub(super) fn limits(bounds: &Bounds) -> Result<Limits, Invalid> {
    let retry = Duration::from_secs(1);
    let client = jig_client_domain::Limits {
        owners: &[],
        requests: skein_lib::RequestLimits {
            requests: 1,
            bytes: bounds.document_bytes,
            out: 1,
            first: retry,
            most: retry,
            margin: retry,
        },
        ask_bytes: bounds.document_bytes,
        handles: 1,
        watches: bounds.watches.max(1),
        reads: 1,
        turn_bytes: u32::try_from(bounds.document_bytes).map_err(|_| Invalid::Payload)?,
        document_bytes: bounds.document_bytes,
        backoff: jig_client_domain::Backoff { first: retry, most: retry },
        rows: bounds.rows,
        accounts: jig_client_domain::AccountCounts {
            accounts: bounds.rows.max(1),
            sources: bounds.rows.max(1),
            sign_ins: bounds.rows.max(1),
        },
        segments: bounds.segments,
        authority: bounds.authority,
        text: bounds.text,
        turn_text: bounds.text.min(u32::try_from(bounds.document_bytes).map_err(|_| Invalid::Payload)?),
        projects: bounds.rows,
        fields: bounds.rows,
        delegates: bounds.rows,
        records: bounds.rows,
        kept: bounds.rows,
        parts: bounds.rows,
        heartbeat: bounds.heartbeat,
    };
    Limits::new(client, 1).map_err(|_| Invalid::Payload)
}

pub(crate) fn read(value: &c::ReadValue, bounds: &Bounds) -> Result<c::ReadValue, Invalid> {
    let record = values::wire_read_value(value, &w::CEILINGS).map_err(|_| Invalid::Payload)?;
    let bytes = encoded(&record, w::ReadValue::measure, w::ReadValue::encode).map_err(|_| Invalid::Payload)?;
    translate::reads::value(&bytes, &limits(bounds)?).map_err(|_| Invalid::Payload)
}

pub(crate) fn delivery(value: &c::Delivery, bounds: &Bounds) -> Result<c::Delivery, Invalid> {
    let record = values::wire_delivery_value(value, &w::CEILINGS).map_err(|_| Invalid::Payload)?;
    let bytes = encoded(&record, w::DeliveryValue::measure, w::DeliveryValue::encode).map_err(|_| Invalid::Payload)?;
    translate::watches::delivery(&bytes, &limits(bounds)?).map_err(|_| Invalid::Payload)
}

pub(crate) fn none(value: c::ReadNone) -> c::ReadNone {
    let wire = &w::CEILINGS;
    let record = match value {
        c::ReadNone::TooLarge { bytes } => w::ReadNone::TooLarge(
            w::TurnTooLarge::new(wire, w::TurnTooLargeParts { bytes }).expect("scalar read refusal"),
        ),
        c::ReadNone::TooLinked { joins } => w::ReadNone::TooLinked(
            w::ReadTooLinked::new(wire, w::ReadTooLinkedParts { joins }).expect("scalar read refusal"),
        ),
        c::ReadNone::Malformed => w::ReadNone::Malformed,
        c::ReadNone::Unknown => w::ReadNone::Unknown,
        c::ReadNone::Refused => w::ReadNone::Refused,
        c::ReadNone::Gone => w::ReadNone::Gone,
        c::ReadNone::Busy => w::ReadNone::Busy,
        c::ReadNone::SignedOut => w::ReadNone::SignedOut,
        c::ReadNone::Unreachable => return c::ReadNone::Unreachable,
    };
    let record = w::ReadAnswer::None(
        w::ReadAnswerNone::new(wire, w::ReadAnswerNoneParts { value: record }).expect("scalar read answer"),
    );
    let bytes = encoded(&record, w::ReadAnswer::measure, w::ReadAnswer::encode).expect("measured read answer");
    match w::ReadAnswer::decode(wire, &mut Reader::new(&bytes)).expect("encoded read answer") {
        w::ReadAnswer::None(value) => translate::reads::none(value.value()),
        w::ReadAnswer::Value(_) | w::ReadAnswer::NotServed(_) => unreachable!("encoded refusal"),
    }
}

pub(crate) fn answer(value: &c::Answer, bounds: &Bounds) -> Result<c::Answer, Invalid> {
    let wire = &w::CEILINGS;
    let record = match value {
        c::Answer::Done(value) => w::Final::Done(
            w::Done::new(
                wire,
                w::DoneParts { outcome: values::wire_outcome(value, wire).map_err(|_| Invalid::Payload)? },
            )
            .map_err(|_| Invalid::Payload)?,
        ),
        c::Answer::Refused { refusal, at } => w::Final::Refused(
            w::Refused::new(
                wire,
                w::RefusedParts {
                    refusal: values::wire_refusal(refusal, wire).map_err(|_| Invalid::Payload)?,
                    at: *at,
                },
            )
            .map_err(|_| Invalid::Payload)?,
        ),
    };
    let record = encoded(&record, w::Final::measure, w::Final::encode).map_err(|_| Invalid::Payload)?;
    let owned =
        w::Owned::new(wire, w::OwnedParts { owner: w::Owner::Jig, version: jig_client_protocol_wire::VERSION, record })
            .map_err(|_| Invalid::Payload)?;
    let keyed = w::KeyedAnswer::FinalAnswer(owned);
    let bytes = encoded(&keyed, w::KeyedAnswer::measure, w::KeyedAnswer::encode).map_err(|_| Invalid::Payload)?;
    let limits = limits(bounds)?;
    let record = w::KeyedAnswer::decode(limits.wire(), &mut Reader::new(&bytes)).map_err(|_| Invalid::Payload)?;
    translate::answers::answer(record, &limits).map_err(|_| Invalid::Payload)
}

/// Outgoing requests use the link's wire translation and the same receiving limits.
pub(crate) fn ask(value: &c::Ask, bounds: &Bounds) -> Result<(), Invalid> {
    let limits = limits(bounds)?;
    let record = translate::asks::ask(value, limits.wire()).map_err(|_| Invalid::Payload)?;
    let bytes = encoded(&record, w::Ask::measure, w::Ask::encode).map_err(|_| Invalid::Payload)?;
    if u64::try_from(bytes.len()).map_err(|_| Invalid::Payload)? > bounds.document_bytes {
        return Err(Invalid::Payload);
    }
    w::Ask::decode(limits.wire(), &mut Reader::new(&bytes)).map_err(|_| Invalid::Payload)?;
    Ok(())
}

pub(crate) fn selector(value: &c::Read, bounds: &Bounds) -> Result<(), Invalid> {
    translate::reads::read(value, limits(bounds)?.wire()).map_err(|_| Invalid::Payload)?;
    Ok(())
}

pub(crate) fn subject(value: &c::Subject, bounds: &Bounds) -> Result<(), Invalid> {
    translate::watches::subject(value, limits(bounds)?.wire()).map_err(|_| Invalid::Payload)?;
    Ok(())
}
