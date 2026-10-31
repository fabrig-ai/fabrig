//! Selected amendments are independent of displayed source facts (view.md, 18.5).

use crate::tests::fixtures::Fixture;
use crate::tests::kit::Goldens;
use crate::tests::tree::cards_objects::*;

fn empty() -> Amendment {
    Amendment {
        llm: None,
        addressed: None,
        spec: None,
        instructions: None,
        wake: None,
        dependencies: None,
        authority: None,
        tracked: None,
    }
}
fn sent(out: &[Request]) -> &Amendment {
    out.iter()
        .find_map(|request| match request {
            Request::Client { asked: jig_client::Event::Send { request: jig_client::Ask::Jig(ask), .. } } => {
                match &**ask {
                    Ask::Amend { change, .. } => Some(change.as_ref()),
                    Ask::Compound { .. } => crate::tests::fixtures::selected_change(ask),
                    _ => None,
                }
            }
            _ => None,
        })
        .expect("the selected amendment is sent")
}
fn open(row: Task) -> (Fixture, Id<Object>) {
    let (mut domain, id) = loaded(ObjectKey::Task(8), ReadValue::Task(Box::new(row)));
    drive(&mut domain, Event::Act { action: Action::Intend { object: id.token(), act: ActKind::Amend } });
    drive(&mut domain, Event::Act { action: Action::Edit { field: Purpose::Reason, text: text(b"") } });
    (domain, id)
}

#[test]
fn amendment_dependency_and_wake_controls_preserve_only_prior_selected_changes() {
    let mut goldens = Goldens::new("tree/cards_objects/amendment_presence/amendment_dependency_and");
    let mut row = task_doc(&[ActKind::Amend]);
    row.dependencies = list([OwnerTaskLink::Live(chip())]);
    row.wake.notices = WakeRule::Batch { count: 11, age: Duration::from_nanos(22) };
    row.wake.news = WakeRule::Immediate;
    row.wake.questions = true;
    row.subscriptions.entries =
        list([Subscription { number: 11, kind: SubscriptionKind::Topic { connector: 2, topic: 22 } }]);
    let initial_wake = row.wake;
    let (mut domain, _) = open(row);
    let words = words();
    let mut bindings = JigBindings::new(limits().bindings, CLIENT.client.text);
    let tree = dialog(&domain, words, &mut bindings);
    drive(&mut domain, Event::Act { action: press(&tree, b"Drop dependency 8", &mut bindings) });
    let mut expected = Amendment { dependencies: Some(list([])), ..empty() };
    assert_eq!(
        domain.confirmation().unwrap().choices.change.as_ref().unwrap().dependencies.as_ref().unwrap().capacity(),
        1
    );
    assert_eq!(domain.confirmation().unwrap().choices.change.as_ref(), Some(&expected));
    let tree = dialog(&domain, words, &mut bindings);
    drive(&mut domain, Event::Act { action: press(&tree, b"Wakes on inbox messages", &mut bindings) });
    let mut wake = initial_wake;
    wake.words = WakeRule::Never;
    expected.wake = Some(wake);
    assert_eq!(domain.confirmation().unwrap().choices.change.as_ref(), Some(&expected));
    let tree = dialog(&domain, words, &mut bindings);
    drive(&mut domain, Event::Act { action: press(&tree, b"Wakes on a delegate result", &mut bindings) });
    wake.results = ResultsWake::Never;
    expected.wake = Some(wake);
    let tree = dialog(&domain, words, &mut bindings);
    goldens.check(&tree);
    drive(&mut domain, Event::Act { action: input(&tree, b"New instructions", b"", &mut bindings) });
    expected.instructions = Some(text(b""));
    let out = drive(&mut domain, Event::Act { action: Action::Confirm });
    assert_eq!(sent(&out), &expected);
}

#[test]
fn amendment_loss_in_specification_does_not_select_or_block_independent_fields() {
    let mut goldens = Goldens::new("tree/cards_objects/amendment_presence/amendment_loss_in");
    for mask in 1_u8..8 {
        let mut row = task_doc(&[ActKind::Amend]);
        row.spec.specification.words.cut = mask & 1 != 0;
        row.spec.specification.parameters = list([ParameterDocument::Bytes {
            name: 8,
            value: DocumentText { bytes: text(UTF8_TEXTS[0]), cut: mask & 2 != 0 },
        }]);
        row.spec.cut = mask & 4 != 0;
        let (mut domain, _) = open(row);
        let mut bindings = JigBindings::new(limits().bindings, CLIENT.client.text);
        let tree = dialog(&domain, words(), &mut bindings);
        goldens.check(&tree);
        drive(&mut domain, Event::Act { action: input(&tree, b"New instructions", b"", &mut bindings) });
        let out = drive(&mut domain, Event::Act { action: Action::Confirm });
        assert_eq!(sent(&out), &Amendment { instructions: Some(text(b"")), ..empty() });
    }
}

const UTF8_TEXTS: [&[u8]; 1] = [b"\0~"];
