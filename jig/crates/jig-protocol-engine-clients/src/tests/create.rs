use super::fixtures as f;
use crate::translate::asks;
use jig_client_protocol_wire::v19 as w;
use jig_domain_people as p;
#[test]
fn a_create_reaches_the_people_child_with_each_task_s_fields() {
    let (wire, expected) = f::create_ask();
    assert_eq!(asks::ask(wire, &f::models(), &f::wire()), Ok(expected));
}
#[test]
fn an_as_allowed_answer_names_what_was_given() {
    let l = f::wire();
    let w::Final::Done(value) = f::translated(
        &p::Reply::Outcome(p::Outcome::Created {
            first: f::FIRST_TASK,
            last: f::LAST_TASK,
            given: Some(p::Given { task: f::LAST_TASK, amount: 42 }),
        }),
        &l,
    ) else {
        panic!("done")
    };
    assert_eq!(value.outcome(), &f::created_outcome());
}
#[test]
fn a_create_refusal_keeps_its_member_and_exact_category() {
    let l = f::wire();
    for (name, why, expected) in f::create_refusals() {
        let w::Final::Refused(value) = f::translated(&p::Reply::Refused(why), &l) else { panic!("{name}: refusal") };
        assert_eq!((value.refusal(), *value.at()), (&expected, None), "{name}");
    }
}

#[test]
fn a_new_tasks_symbols_reach_the_people_member() {
    let symbols = [p::Symbol { connector: 7, symbol: 3 }, p::Symbol { connector: 2, symbol: 11 }];
    for values in [&[][..], &symbols[..]] {
        let ask = create_with_symbols(values);
        let p::Ask::Create { batch, .. } = asks::ask(ask, &f::models(), &f::wire()).unwrap() else { panic!("create") };
        assert_eq!(batch[0].symbols.as_ref(), values);
    }
}

#[test]
fn a_new_tasks_symbols_are_refused_above_the_receiving_bound() {
    let symbols = [p::Symbol { connector: 7, symbol: 3 }, p::Symbol { connector: 2, symbol: 11 }];
    let limits = w::Limits { new_task_symbols: 1, ..f::wire() };
    asks::ask(create_with_symbols(&symbols[..1]), &f::models(), &limits).unwrap();
    assert_eq!(
        asks::ask(create_with_symbols(&symbols), &f::models(), &limits),
        Err(crate::translate::Untranslatable { path: w::Path::NewTaskSymbols })
    );
}

fn create_with_symbols(symbols: &[p::Symbol]) -> w::Ask {
    let limits = f::wire();
    let (w::Ask::Create(create), _) = f::create_ask() else { panic!("create fixture") };
    let bytes = skein_codec::piece::split(&mut skein_lib::Reader::new(create.tasks())).unwrap().unwrap();
    let mut task = w::NewTask::decode(&limits, &mut skein_lib::Reader::new(&bytes)).unwrap().into_parts();
    task.symbols = skein_lib::List::with_capacity(u32::try_from(symbols.len()).unwrap());
    for symbol in symbols {
        task.symbols
            .push(
                w::Symbol::new(&limits, w::SymbolParts { connector: symbol.connector, symbol: symbol.symbol }).unwrap(),
            )
            .unwrap();
    }
    let task = w::NewTask::new(&limits, task).unwrap();
    let mut writer = skein_lib::Writer::new(skein_lib::length::usize_of(task.measure()));
    task.encode(&mut writer).unwrap();
    w::Ask::Create(
        w::Create::new(&limits, w::CreateParts { project: f::PROJECT, tasks: f::pieces(&[writer.finish()]) }).unwrap(),
    )
}
