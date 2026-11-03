//! Public reads decide nothing (jig/docs/design/domain/engine.md, 11).
#![expect(
    clippy::disallowed_macros,
    reason = "unit tests snapshot the complete core state to check that public messages change nothing"
)]
use crate::{Event, Held, PublicAnswer, PublicApplication, PublicRead, Request, step};
use skein_lib::{Env, ReplyTo, Time, Token, Wall};

#[test]
fn a_public_read_is_answered_unknown_and_commits_nothing() {
    let (mut core, limits) = crate::tests::core();
    crate::tests::make_task(&mut core, &limits, 1);
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    core.start_input();
    let before = format!("{core:?}");
    for (owner, kind, project, task) in
        [(0, 1, None, None), (0, 2, Some(1), None), (0, 3, Some(1), Some(1)), (1, 1, Some(1), Some(1))]
    {
        let token = Token::new(90);
        let read = PublicRead { owner, version: 1, kind, project, task };
        let event = Event::PublicRead { reply: ReplyTo::new(token), read };
        assert!(event.new_work());
        let mut out = step(&mut core, &env, event);
        assert_eq!(out.len(), 1, "only the terminal leaves: no write, load or mark");
        let Some(Request::Held(held)) = out.pop() else { panic!("a public terminal is held") };
        let Held::PublicRead { reply, answer } = *held else { panic!("a public read has its own answer") };
        assert_eq!(reply.into_token(), token);
        assert_eq!(answer, PublicAnswer::Unknown);
        assert_eq!(format!("{core:?}"), before, "no party or other core state changes");
        assert!(crate::finish_decision(&mut core, &env).is_empty(), "no deferred output or marks");
        assert_eq!(format!("{core:?}"), before);
    }
}

#[test]
fn an_unasked_public_answer_changes_nothing() {
    let (mut core, limits) = crate::tests::core();
    crate::tests::make_task(&mut core, &limits, 1);
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    core.start_input();
    let before = format!("{core:?}");
    for answer in [None, Some(PublicApplication { tasks: Box::new([1]), record: Box::new([7]) })] {
        let event = Event::PublicAnswered { reply: ReplyTo::new(Token::new(91)), answer };
        assert!(!event.new_work());
        assert!(step(&mut core, &env, event).is_empty());
        assert_eq!(format!("{core:?}"), before, "an unsolicited answer changes no state");
        assert!(crate::finish_decision(&mut core, &env).is_empty());
        assert_eq!(format!("{core:?}"), before);
    }
}
