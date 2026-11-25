//! Live host operations become named channel records (protocol/channel.md,
//! sections 3–5; domain/run.md, section 5, and domain/run-delivery.md, section 1).
//!
//! This module keeps no state. The component retains each live relay until its
//! one host answer; these encoders know neither host policy nor callbacks.

use alloc::boxed::Box;
use skein_channel::Frame;
use skein_lib::Duration;
use smith_channel as wire;
use smith_channel::Record;
use smith_domain::run;

use crate::{Error, generated::channel};

/// Encode a declared host tool relay without interpreting its attested input.
pub(crate) fn host_call(
    name: run::CallName,
    tool: Box<[u8]>,
    effect: run::HostEffect,
    input: run::HostInput,
    deadline: Duration,
    limits: &wire::Limits,
) -> Result<Frame, Error> {
    let ask = wire::Ask::Host(wire::HostAsk { tool, input: Box::from(input.bytes()) });
    call(name, effect, ask, deadline, limits)
}

/// Encode one checked delivery request with the domain's host-named fields.
pub(crate) fn delivery_call(
    name: run::CallName,
    change: run::outcome::Change,
    deadline: Duration,
    limits: &wire::Limits,
) -> Result<Frame, Error> {
    let record = channel::deliver_ask_to_record(&change).or(Err(Error::Capacity))?;
    let ask = wire::Ask::Deliver(record);
    call(name, run::HostEffect::Write, ask, deadline, limits)
}

/// Encode an unchecked save with no fields under the delivery reply route.
pub(crate) fn save_call(name: run::CallName, deadline: Duration, limits: &wire::Limits) -> Result<Frame, Error> {
    call(name, run::HostEffect::Write, wire::Ask::Save, deadline, limits)
}

fn call(
    name: run::CallName,
    effect: run::HostEffect,
    ask: wire::Ask,
    deadline: Duration,
    limits: &wire::Limits,
) -> Result<Frame, Error> {
    let name = channel::call_name_to_record(&name);
    let effect = channel::effect_to_record(&effect);
    let record = wire::Call { name, effect, deadline, ask };
    match record.measure(limits) {
        Ok(_) => (),
        Err(problem) => return Err(Error::ChannelBody(problem)),
    }
    let frame = match smith_channel::frame(smith_channel::Body::Call(record)) {
        Ok(value) => value,
        Err(problem) => return Err(Error::Frame(problem)),
    };
    Ok(frame)
}

/// Ask the host to settle one relay without releasing its durable name.
pub(crate) fn withdraw(name: run::CallName, limits: &wire::Limits) -> Result<Frame, Error> {
    let name = channel::call_name_to_record(&name);
    let record = wire::Withdraw { name };
    match record.measure(limits) {
        Ok(_) => (),
        Err(problem) => return Err(Error::ChannelBody(problem)),
    }
    let frame = match smith_channel::frame(smith_channel::Body::Withdraw(record)) {
        Ok(value) => value,
        Err(problem) => return Err(Error::Frame(problem)),
    };
    Ok(frame)
}

/// Interpret one host terminal for a declared tool; temporary refusals stay distinct.
pub(crate) fn host_reply(reply: &wire::Reply) -> Option<run::HostReply> {
    let value = match reply {
        wire::Reply::Host(host) => match run::HostAnswer::new(Box::from(host.text.as_ref()), host.error) {
            Some(answer) => run::HostReply::Answered(answer),
            None => run::HostReply::TooLarge,
        },
        wire::Reply::Busy => run::HostReply::Busy,
        wire::Reply::Unavailable => run::HostReply::Unanswered(run::Unanswered::Lost),
        wire::Reply::Withdrawn => run::HostReply::Withdrawn,
        wire::Reply::Unsent => run::HostReply::Unsent,
        wire::Reply::TooLarge => run::HostReply::TooLarge,
        wire::Reply::Delivery(_) => return None,
    };
    Some(value)
}
