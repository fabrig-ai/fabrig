use super::tests::{at, fact, intent, opened, prompt};
use super::*;

#[test]
fn a_model_change_compactions_conversation_names_the_windows_model() {
    let owner = Token::new(11);
    let mut book = Book::new(1);
    book.opened(at(1), owner, &opened(11, 0));
    let kind = session::FactKind::compaction_started(owner, 1, b"old-model", 256, 7, Some(b"low"));
    let records = book.fact(Time::ZERO, &fact(2, kind, 1, None));
    let Event::ConversationOpened(opened) = &records.get(0).expect("opening").event else {
        panic!("compaction opening")
    };
    assert_eq!(opened.model.model.as_ref(), b"old-model");
    assert_eq!((opened.model.window_tokens, opened.model.output_tokens), (256, 7));
    assert_eq!(opened.model.effort.as_deref(), Some(b"low".as_slice()));
    intent(&mut book, owner, session::Choice::Auto, 0);
    let mut requested = prompt(0);
    requested.model = b"old-model".as_slice().into();
    requested.effort = Some(b"low".as_slice().into());
    let started = book.request(at(4), Time::ZERO, Token::new(99), &requested);
    let Event::ResponseStarted(response) = started.event else { panic!("compaction request") };
    assert_eq!(response.model, opened.model);
}
