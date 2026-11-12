//! A person Create made whole (temper/docs/design/client/ctl/tools.md, 3).
use super::{Call, Failed, Limits, Prepared, Request, c, number, value, words};
use crate::jig::{call::{Value, Values}, render::Text};
use skein_lib::List;

pub(super) fn ask(call: &Call, me: &c::Me, limits: &Limits) -> Prepared {
    let project = super::super::reads::project(call, me).ok_or(Failed::Fixed(b"unknown project"))?;
    if value(call, 1).is_some() && value(call, 2).is_some() {
        return Err(Failed::Fixed(b"--party and --role are alternatives"));
    }
    let Some(Some(Values::Many(choices))) = call.values.get(3) else {
        return Err(Failed::Fixed(b"--choice needs two to six choices"));
    };
    if choices.len() < 2 || choices.len() > 6 {
        return Err(Failed::Fixed(b"--choice needs two to six choices"));
    }
    let mut verdicts = List::with_capacity(6);
    let mut parameters = List::with_capacity(6);
    for (index, choice) in choices.iter().enumerate() {
        let Value::Text(choice) = choice else { return Err(Failed::Fixed(b"choice must be text")) };
        let code = u32::try_from(index).expect("six choices").checked_add(1).expect("choice code");
        let choice = c::Text::new(choice.clone(), limits.words).ok_or(Failed::Fixed(b"choice past words bound"))?;
        if choice.bytes().is_empty() { return Err(Failed::Fixed(b"choice must not be empty")); }
        parameters.push(c::Parameter::Bytes { name: code, value: choice }).expect("six choices");
        verdicts.push(c::VerdictChoice { code, words: limits.words, followups: 0, fields: List::with_capacity(0) }).expect("six choices");
    }
    let question = words(call, 5, limits)?;
    let why = words(call, 4, limits)?;
    let mut measure = Text::measure();
    spec(&question, &why, &mut measure);
    let mut text = Text::writing_exact(measure);
    spec(&question, &why, &mut text);
    let words = c::Text::new(text.finish(), limits.words).ok_or(Failed::Fixed(b"question and why past words bound"))?;
    let address = match number(call, 1) {
        Some(party) => c::PersonAddress::Party(party),
        None => c::PersonAddress::Role(u32::try_from(number(call, 2).unwrap_or(0)).expect("role fits")),
    };
    let task = c::NewTask {
        symbols: List::with_capacity(0), executor: c::Executor::Person(address),
        spec: c::Specification { words, parameters, inputs: List::with_capacity(0), asks: false },
        contract: c::NewContract::Verdict { choices: verdicts },
        authority: c::Authority { models: None, tools: 0, grants: List::with_capacity(0),
            delegation: c::Delegation { kinds: List::with_capacity(0), tasks: 0, depth: 0 },
            spend: 0, deadline: None, notes: 0, note_resources: List::with_capacity(0) },
        budget: c::NewBudget::Named, deadline: c::NewDeadline::Named, wake: None,
        dependencies: List::with_capacity(0), named: List::with_capacity(0), tracked: None, label: 0,
        flags: c::TaskFlags { conversation: false, listed: true, retention: c::Retention::Kept, ended_by: c::EndedBy::Result },
        llm: None,
    };
    Ok((Request::Ask(c::Ask::Create { project, tasks: List::from_boxed(alloc::boxed::Box::new([task])) }), None))
}
fn spec(question: &c::Text, why: &c::Text, out: &mut Text) {
    out.put(question.bytes()); out.put(b"\n\nWhy: "); out.put(why.bytes());
}
