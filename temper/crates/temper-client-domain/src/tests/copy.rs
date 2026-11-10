//! The command’s Copy (temper/docs/design/client/pages/README.md).
use super::fixtures;
use crate::{
    Action, Event, Request,
    copy::{Label, SHOWN, Written},
};
use skein_lib::{Queue, Time};

#[test]
fn presses_report_only_the_latest_write_and_expire_after_two_seconds() {
    let limits = fixtures::limits();
    let mut root = crate::unlinked(&limits, fixtures::SEED);
    let mut env = fixtures::env(&limits);
    let mut out = Queue::uncapped();
    assert_eq!(root.copy_label(), Label::Copy);
    assert_eq!(crate::next_deadline(&root), None);
    let mut previous = crate::shown(&root);
    for (index, written) in [Written::Done, Written::Refused, Written::Failed].into_iter().enumerate() {
        let token = u32::try_from(index + 1).unwrap();
        crate::step(
            &mut root,
            &env,
            Event::Act { action: Action::Copy { words: b"exact words".as_slice().into() } },
            &mut out,
        );
        assert_eq!(out.pop(), Some(Request::Copy { token, words: b"exact words".as_slice().into() }));
        assert!(out.is_empty());
        crate::step(&mut root, &env, Event::Copied { token, written }, &mut out);
        assert_eq!(root.copy_label(), if written == Written::Done { Label::Copied } else { Label::NotCopied });
        assert!(crate::shown(&root) > previous);
        previous = crate::shown(&root);
        let deadline = env.now.checked_add(SHOWN).unwrap();
        assert_eq!(crate::next_deadline(&root), Some(deadline));
        crate::step(&mut root, &env, Event::Copied { token: token - 1, written: Written::Done }, &mut out);
        assert_eq!(crate::shown(&root), previous);
        assert_eq!(crate::next_deadline(&root), Some(deadline));
        env.now = Time::from_nanos(deadline.as_nanos() - 1);
        let copy_shown = root.copy.shown;
        crate::fire(&mut root, &env, &mut out);
        assert_ne!(root.copy_label(), Label::Copy);
        assert_eq!(root.copy.shown, copy_shown);
        previous = crate::shown(&root);
        env.now = deadline;
        crate::fire(&mut root, &env, &mut out);
        assert_eq!(root.copy_label(), Label::Copy);
        assert_eq!(crate::next_deadline(&root), None);
        assert!(crate::shown(&root) > previous);
        previous = crate::shown(&root);
    }
}

#[test]
fn pressing_during_feedback_writes_again_and_ignores_the_earlier_press() {
    let limits = fixtures::limits();
    let mut root = crate::unlinked(&limits, fixtures::SEED);
    let env = fixtures::env(&limits);
    let mut out = Queue::uncapped();
    for token in 1..=2 {
        crate::step(
            &mut root,
            &env,
            Event::Act { action: Action::Copy { words: b"command".as_slice().into() } },
            &mut out,
        );
        assert_eq!(out.pop(), Some(Request::Copy { token, words: b"command".as_slice().into() }));
        crate::step(&mut root, &env, Event::Copied { token, written: Written::Done }, &mut out);
    }
    let shown = crate::shown(&root);
    crate::step(&mut root, &env, Event::Copied { token: 1, written: Written::Failed }, &mut out);
    assert_eq!(root.copy_label(), Label::Copied);
    assert_eq!(crate::shown(&root), shown);
}
