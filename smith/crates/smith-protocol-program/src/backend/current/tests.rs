use super::*;
use crate::backend::fixtures;

#[test]
fn a_terminal_releases_its_progress_deadline_and_retains_only_context() {
    let backend = fixtures::backend();
    let request = Complete::from_request(fixtures::request(&backend));
    let mut current = Current::new();
    let mut up = Queue::uncapped();
    current.claim(request.owner, &mut up);
    current.retain(request, &mut up);
    current.progress(fixtures::env(0).now, true, &mut up);
    assert_eq!(current.next_deadline(), Some(fixtures::env(1).now));
    assert_eq!(current.phase(fixtures::env(0).now), llm::CompletionPhase::Head);
    current.progress(fixtures::env(29).now, false, &mut up);
    assert_eq!(current.next_deadline(), Some(fixtures::env(30).now));
    assert_eq!(current.phase(fixtures::env(29).now), llm::CompletionPhase::Idle);
    assert_eq!(current.phase(fixtures::env(30).now), llm::CompletionPhase::Whole);
    current.settle(Terminal::Cancelled, &mut up);
    let Some(ToDomain::Cancelled { owner }) = up.pop() else { panic!("one completion terminal") };
    assert_eq!(owner, fixtures::COMPLETION);
    assert!(up.is_empty());
    assert_eq!(current.owner(), None);
    assert_eq!(current.next_deadline(), None);
    assert!(current.request().is_some());
    current.clear(&mut up);
    assert!(current.request().is_none());
    assert!(up.is_empty());
}
