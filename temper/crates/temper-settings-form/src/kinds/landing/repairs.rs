//! The named template choice in kind 4 (domain/red-repairs.md, 7).
use super::{number, uint};
use crate::file::{Decoder, Error, Fields, error, field, optional};
use skein_json::{Kind, writer::Encoder};
use skein_lib::Reader;
use temper_client_protocol_wire::v1 as w;
use temper_client_protocol_wire::v1::Record;

pub(super) fn read(d: &Decoder<'_>, f: &Fields) -> Result<Option<w::RepairSettings>, Error> {
    let Some(at) = optional(f, b"repairs") else { return Ok(None) };
    match repairs(d, at) {
        Ok(v) => Ok(Some(v)),
        Err(why) => Err(why.at(b"repairs")),
    }
}
fn repairs(d: &Decoder<'_>, at: u32) -> Result<w::RepairSettings, Error> {
    let f = d.object(at, &[b"repair_goals", b"repair_attempts", b"repair_deadline_ms", b"goal"])?;
    let goal = match goal(d, field(&f, b"goal")?) {
        Ok(goal) => goal,
        Err(why) => return Err(why.at(b"goal")),
    };
    {
        let value = w::RepairSettings {
            goals: u8::try_from(number(d, &f, b"repair_goals", 2)?).or(Err(error(b"repair_goals", b"u8 bound")))?,
            attempts: u8::try_from(number(d, &f, b"repair_attempts", 2)?)
                .or(Err(error(b"repair_attempts", b"u8 bound")))?,
            deadline_ms: number(d, &f, b"repair_deadline_ms", 86_400_000)?,
            goal,
        };
        match value.encode(&d.kinds) {
            Ok(_) => Ok(value),
            Err(problem) => Err(problem),
        }
    }
    .or(Err(error(b"", b"wire value bound")))
}
fn goal(d: &Decoder<'_>, at: u32) -> Result<w::RepairGoalSettings, Error> {
    let f = d.object(at, &[b"charter", b"llm", b"priority", b"budget"])?;
    let llm = match optional(&f, b"llm") {
        None => None,
        Some(at) if d.kind(at)? == Kind::Null => None,
        Some(at) => {
            let choice = crate::file::values::llm_choice(d, at)?;
            let choice = crate::wire::llm_wire_choice(&choice);
            let bytes = choice.encode(&d.limits).or(Err(error(b"llm", b"wire value bound")))?;
            Some(w::NamedLlm::decode(&d.kinds, &mut Reader::new(&bytes)).or(Err(error(b"llm", b"wire value bound")))?)
        }
    };
    {
        let value = w::RepairGoalSettings {
            charter: super::super::number(d, &f, b"charter")?,
            llm,
            priority: super::super::number(d, &f, b"priority")?,
            budget: d.number(field(&f, b"budget")?)?,
        };
        match value.encode(&d.kinds) {
            Ok(_) => Ok(value),
            Err(problem) => Err(problem),
        }
    }
    .or(Err(error(b"", b"wire value bound")))
}
pub(super) fn render(v: &w::RepairSettings, out: &mut Encoder) {
    out.object_start();
    for (name, value, default) in [
        (b"repair_goals".as_slice(), u64::from(v.goals), 2),
        (b"repair_attempts".as_slice(), u64::from(v.attempts), 2),
        (b"repair_deadline_ms".as_slice(), v.deadline_ms, 86_400_000),
    ] {
        if value != default {
            uint(out, name, value);
        }
    }
    out.key(b"goal");
    out.object_start();
    let g = &v.goal;
    uint(out, b"charter", u64::from(g.charter));
    out.key(b"llm");
    match &g.llm {
        Some(v) => llm(v, out),
        None => out.null(),
    }
    uint(out, b"priority", u64::from(g.priority));
    uint(out, b"budget", g.budget);
    out.object_end();
    out.object_end();
}
fn llm(v: &w::NamedLlm, out: &mut Encoder) {
    out.object_start();
    out.key(b"account");
    match &v.account {
        Some(n) => out.unsigned(u64::from(*n)),
        None => out.null(),
    }
    out.key(b"main");
    model(&v.main, out);
    out.key(b"subagents");
    models(&v.subagents, out);
    if let Some(v) = &v.later {
        out.key(b"later");
        out.object_start();
        out.key(b"main");
        model(&v.main, out);
        out.key(b"subagents");
        models(&v.subagents, out);
        out.object_end();
    }
    out.object_end();
}
fn model(v: &w::NamedModel, out: &mut Encoder) {
    out.object_start();
    out.key(b"model");
    out.string(&v.model);
    out.key(b"effort");
    match &v.effort {
        Some(effort) => out.string(effort),
        None => out.null(),
    }
    out.object_end();
}
fn models(v: &skein_lib::List<w::NamedModel>, out: &mut Encoder) {
    out.array_start();
    for v in v {
        model(v, out);
    }
    out.array_end();
}
