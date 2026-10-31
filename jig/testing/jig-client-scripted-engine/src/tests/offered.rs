//! Offers and paged rows have independent admission and allocation bounds.
use super::*;

fn offered(kinds: &[ActKind], capacity: u32) -> List<Offered> {
    let mut result = List::with_capacity(capacity);
    for act in kinds {
        result.push(Offered { effect: None, act: *act, admission: Admission::Allowed }).unwrap();
    }
    result
}
fn seven() -> Task {
    let mut value = task();
    value.acts = offered(
        &[
            ActKind::Message,
            ActKind::Amend,
            ActKind::Move,
            ActKind::Release,
            ActKind::Stop,
            ActKind::Cancel,
            ActKind::Seen,
        ],
        7,
    );
    value
}
#[test]
fn seven_offers_cross_normal_task_read_and_delivery_whole_with_two_paged_rows() {
    let value = seven();
    let mut setup = script();
    setup.bounds = Bounds { rows: 2, text: 32, ..setup.bounds };
    setup.truth.objects.insert(ObjectKey::Task(8), ReadValue::Task(Box::new(value.clone())));
    let mut engine = Engine::new(setup, 17).unwrap();
    engine.read(Time::ZERO, 11, Token::new(1), Read::Task { task: 8 }).unwrap();
    engine.open(Time::ZERO, 11, Token::new(2), Subject::Task { task: 8 }).unwrap();
    let outputs = engine.poll(Time::ZERO);
    assert!(outputs.contains(&Up::Read {
        token: Token::new(1),
        terminal: ReadResult::Value(ReadValue::Task(Box::new(value.clone())))
    }));
    assert!(outputs.contains(&Up::Watch {
        token: Token::new(2),
        event: WatchEvent::Delivered(Delivery::Task(Box::new(value.clone())))
    }));
}
