//! A held mark follows the head, and a drain replaces it with a saved mark
//! (temper/docs/design/domain/worker/checkouts.md, A workspace kept for its run).
use super::*;
use crate::git::{Kept, KeptState};

fn marked() -> (Harness, Token, Kept) {
    let mut h = Harness::new(LIMITS);
    h.observe_marks = true;
    let (hold, mut op) = h.prepare(1, Spec { key: bytes(b"run"), repositories: Box::new([repository(b"a", true)]) });
    for _ in 0..32_u32 {
        if let Op::Keep { mark: Some(mark), .. } = op {
            return (h, hold, mark);
        }
        let done = if let Op::Fetch { .. } = op { Done::Fetched { commit: commit(1) } } else { Done::Succeeded };
        op = h.next(hold, done);
    }
    panic!("prepare writes its mark before answering");
}

fn finish_mark(h: &mut Harness, hold: Token, done: Done) {
    let Request::Prepared { prepared: Prepared::Ready { .. }, .. } = h.one(Event::Done { owner: hold, done }) else {
        panic!("prepare succeeds after its mark");
    };
}

fn save(h: &mut Harness, hold: Token, head: Commit) -> Kept {
    assert_eq!(io(h.one(Event::Save { hold, message: message() }), hold).kind(), Kind::Commit);
    let Op::Push { commit, .. } = h.next(hold, Done::Committed { commit: head }) else { panic!("push save") };
    assert_eq!(commit, head);
    let requests = h.step(Event::Done { owner: hold, done: Done::Succeeded });
    let [
        Request::Saved { outcome: Outcome::Pushed { .. }, .. },
        Request::Io { op: Op::Keep { mark: Some(mark), .. }, .. },
    ] = requests.as_slice()
    else {
        panic!("the save answers without waiting for its rewrite: {requests:?}");
    };
    h.none(Event::Done { owner: hold, done: Done::Succeeded });
    mark.clone()
}

#[test]
fn a_run_marks_its_start_save_delivery_and_release() {
    let (mut h, hold, mark) = marked();
    assert_eq!(mark.state, KeptState::Held);
    assert_eq!(mark.key.as_ref(), b"run");
    let repository = mark.repositories.first().expect("one repository");
    assert_eq!(repository.commit, commit(1));
    assert_eq!(repository.merging, None);
    assert!(!repository.unchecked);
    finish_mark(&mut h, hold, Done::Succeeded);
    let saved = save(&mut h, hold, commit(4));
    assert_eq!(saved.state, KeptState::Held);
    assert_eq!(saved.repositories.first().expect("one repository").commit, commit(4));
    let Op::Commit { parent, .. } = io(h.one(Event::Push { hold, message: message() }), hold) else {
        panic!("commit delivery")
    };
    assert_eq!(parent, commit(4));
    assert_eq!(h.next(hold, Done::Committed { commit: commit(5) }).kind(), Kind::Push);
    let requests = h.step(Event::Done { owner: hold, done: Done::Succeeded });
    let [Request::Pushed { .. }, Request::Io { op: Op::Keep { mark: Some(rewritten), .. }, .. }] = requests.as_slice()
    else {
        panic!("delivery answers beside its rewrite: {requests:?}");
    };
    assert_eq!(rewritten.repositories.first().expect("one repository").commit, commit(5));
    h.none(Event::Release { hold });
    let Op::Keep { mark: None, .. } = io(h.one(Event::Done { owner: hold, done: Done::Succeeded }), hold) else {
        panic!("remove after the pending rewrite")
    };
    assert_eq!(h.one(Event::Done { owner: hold, done: Done::Succeeded }), Request::Released { client: Token::new(1) });
}

#[test]
fn a_drain_replaces_the_held_mark_with_the_saved_commits() {
    let (mut h, hold, _) = marked();
    finish_mark(&mut h, hold, Done::Succeeded);
    assert_eq!(io(h.one(Event::Save { hold, message: message() }), hold).kind(), Kind::Commit);
    assert_eq!(h.next(hold, Done::Committed { commit: commit(4) }).kind(), Kind::Push);
    let saved = h.step(Event::Done { owner: hold, done: Done::Succeeded });
    let [Request::Saved { .. }, Request::Io { op: Op::Keep { .. }, .. }] = saved.as_slice() else {
        panic!("save and pending held mark")
    };
    h.none(Event::Keep { hold });
    let Op::Keep { mark: Some(mark), .. } = io(h.one(Event::Done { owner: hold, done: Done::Succeeded }), hold) else {
        panic!("drain mark follows held mark")
    };
    assert_eq!(mark.state, KeptState::Kept);
    assert_eq!(mark.repositories.first().expect("one repository").commit, commit(4));
    assert_eq!(h.one(Event::Done { owner: hold, done: Done::Succeeded }), Request::Released { client: Token::new(1) });
}

#[test]
fn a_failed_held_mark_does_not_fail_the_prepare() {
    let (mut h, hold, _) = marked();
    finish_mark(&mut h, hold, Done::Failed { fault: Fault::Broken });
    assert_eq!(io(h.one(Event::Save { hold, message: message() }), hold).kind(), Kind::Commit);
}

#[test]
fn a_cut_does_not_start_git_queued_behind_a_held_mark() {
    let (mut h, hold, _) = marked();
    finish_mark(&mut h, hold, Done::Succeeded);
    assert_eq!(io(h.one(Event::Save { hold, message: message() }), hold).kind(), Kind::Commit);
    assert_eq!(h.next(hold, Done::Committed { commit: commit(4) }).kind(), Kind::Push);
    let requests = h.step(Event::Done { owner: hold, done: Done::Succeeded });
    let [Request::Saved { .. }, Request::Io { op: Op::Keep { .. }, .. }] = requests.as_slice() else {
        panic!("save and held mark")
    };
    h.none(Event::Save { hold, message: message() });
    assert_eq!(h.one(Event::Cut { hold }), Request::Cancel { owner: hold });
    assert_eq!(
        h.one(Event::Done { owner: hold, done: Done::Failed { fault: Fault::Cancelled } }),
        Request::Saved { client: Token::new(1), outcome: Outcome::Pushed { landings: Box::new([Landing::Aborted]) } }
    );
    let Op::Keep { mark: None, .. } = io(h.one(Event::Release { hold }), hold) else { panic!("remove held mark") };
    assert_eq!(h.one(Event::Done { owner: hold, done: Done::Succeeded }), Request::Released { client: Token::new(1) });
}

#[test]
fn holds_without_saved_work_write_no_mark() {
    for writable in [false, true] {
        let mut h = Harness::new(LIMITS);
        h.observe_marks = true;
        let mut repo = repository(b"a", writable);
        repo.save = None;
        let hold = h.ready(1, Spec { key: bytes(b"job"), repositories: Box::new([repo]) });
        for fact in h.facts().as_slice() {
            if let Fact::Started { op: Kind::Keep, .. } = fact {
                panic!("no saved-work destination")
            }
        }
        assert_eq!(h.one(Event::Release { hold }), Request::Released { client: Token::new(1) });
    }
}
