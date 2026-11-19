//! Agents settings form (temper/docs/design/protocol/configuration/figures.md, 5).
use temper_client_protocol_wire::v1::Record;
use crate::file::{Decoder, Error, error, field};
use alloc::boxed::Box;
use skein_json::writer::Encoder;
use skein_lib::{List, Reader, Writer};
use temper_client_protocol_wire::v1 as w;

pub(super) fn value(d: &Decoder<'_>, at: u32) -> Result<Box<[u8]>, Error> {
    let record = match read(d, at) {
        Ok(record) => record,
        Err(why) => return Err(why.at(b"agents")),
    };
    let mut out = Writer::new(usize::try_from(record.measure(&d.kinds).expect("checked wire value")).expect("record length fits"));
    record.write(&d.kinds, &mut out).expect("measured record");
    Ok(out.finish())
}
fn read(d: &Decoder<'_>, at: u32) -> Result<w::AgentFigures, Error> {
    let fields = d.object(at, &[b"profile", b"endpoints", b"fallback"])?;
    let mut profile = List::with_capacity(d.kinds.agent_figures_profile.min(w::CEILINGS.agent_figures_profile));
    for (name, at) in d.named(field(&fields, b"profile")?, profile.capacity())?.into_boxed() {
        let value = match d.number(at) {
            Ok(value) => value,
            Err(why) => return Err(why.at(&name)),
        };
        profile
            .push(
                { let value = w::AgentFigure { name: name.clone(), value }; match value.encode(&d.kinds) { Ok(_) => Ok(value), Err(problem) => Err(problem) } }
                    .or(Err(error(&name, b"profile name bound")))?,
            )
            .expect("bounded profile");
    }
    let mut endpoints = List::with_capacity(d.kinds.agent_figures_endpoints.min(w::CEILINGS.agent_figures_endpoints));
    for (endpoint, at) in d.named(field(&fields, b"endpoints")?, endpoints.capacity())?.into_boxed() {
        let limits = match limits(d, at) {
            Ok(value) => value,
            Err(why) => return Err(why.at(&endpoint)),
        };
        endpoints
            .push(
                { let value = w::AgentEndpointLimits { endpoint: endpoint.clone(), limits }; match value.encode(&d.kinds) { Ok(_) => Ok(value), Err(problem) => Err(problem) } }
                .or(Err(error(&endpoint, b"endpoint name bound")))?,
            )
            .expect("bounded endpoints");
    }
    let fallback = match limits(d, field(&fields, b"fallback")?) {
        Ok(value) => value,
        Err(why) => return Err(why.at(b"fallback")),
    };
    { let value = w::AgentFigures { profile, endpoints, fallback }; match value.encode(&d.kinds) { Ok(_) => Ok(value), Err(problem) => Err(problem) } }
        .or(Err(error(b"agents", b"wire value bound")))
}
fn limits(d: &Decoder<'_>, at: u32) -> Result<w::AgentLimits, Error> {
    let fields = d.object(at, &[b"window", b"output", b"reasoning_item", b"head_ms", b"idle_ms", b"oversized"])?;
    let oversized = match d.string(field(&fields, b"oversized")?)? {
        b"fail" => w::Oversized::Fail,
        b"drop" => w::Oversized::Drop,
        _ => return Err(error(b"oversized", b"fail or drop required")),
    };
    let head_ms = d.number(field(&fields, b"head_ms")?).or(Err(error(b"head_ms", b"u64 required")))?;
    let idle_ms = d.number(field(&fields, b"idle_ms")?).or(Err(error(b"idle_ms", b"u64 required")))?;
    { let value = w::AgentLimits {
            window: super::number(d, &fields, b"window")?,
            output: super::number(d, &fields, b"output")?,
            reasoning_item: super::number(d, &fields, b"reasoning_item")?,
            head_ms,
            idle_ms,
            oversized,
        }; match value.encode(&d.kinds) { Ok(_) => Ok(value), Err(problem) => Err(problem) } }
    .or(Err(error(b"limits", b"wire value bound")))
}
pub(super) fn render(bytes: &[u8], out: &mut Encoder) -> bool {
    let mut reader = Reader::new(bytes);
    let Ok(value) = w::AgentFigures::decode(&w::CEILINGS, &mut reader) else {
        return false;
    };
    if !reader.is_empty() {
        return false;
    }
    out.object_start();
    out.key(b"profile");
    out.object_start();
    for figure in &value.profile {
        out.key(&figure.name);
        out.unsigned(figure.value);
    }
    out.object_end();
    out.key(b"endpoints");
    out.object_start();
    for endpoint in &value.endpoints {
        out.key(&endpoint.endpoint);
        render_limits(&endpoint.limits, out);
    }
    out.object_end();
    out.key(b"fallback");
    render_limits(&value.fallback, out);
    out.object_end();
    true
}
fn render_limits(value: &w::AgentLimits, out: &mut Encoder) {
    out.object_start();
    for (name, amount) in [
        (b"window".as_slice(), u64::from(value.window)),
        (b"output", u64::from(value.output)),
        (b"reasoning_item", u64::from(value.reasoning_item)),
        (b"head_ms", value.head_ms),
        (b"idle_ms", value.idle_ms),
    ] {
        out.key(name);
        out.unsigned(amount);
    }
    out.key(b"oversized");
    out.string(match &value.oversized {
        w::Oversized::Fail => b"fail",
        w::Oversized::Drop => b"drop",
    });
    out.object_end();
}
