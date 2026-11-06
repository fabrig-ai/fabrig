//! Merge scenarios against the real checkout child, fake disk and fake forge.
//! The referee predicts complete trees, parent pairs and remote refs from
//! the scenario; it never reads the domain's private state.
use crate::{forge::Forge, translate};
use skein_lib::{Duration, Env, Queue, Time, Token, Wall};
use temper_fake_checkout::{Checkout, git::Remote};
use temper_worker_domain_checkout::git::{Commit, Done, Fault, Op};
use temper_worker_domain_checkout::{
    Domain, Event, Failure, Landing, Limits, Message, Outcome, Prepared, Repository, Request, Spec, Start, step,
};

#[derive(Clone, Copy, Debug)]
pub enum Case {
    Clean,
    Resumed,
    Contained,
    Conflict,
    Unchanged,
    Moved,
    Ambiguous,
    Saved,
}

#[derive(PartialEq, Eq, Debug)]
pub struct Report {
    trace: Vec<String>,
    tree: temper_fake_checkout::git::Tree,
    parents: (u64, u64),
}

struct World {
    domain: Domain,
    env: Env<Limits>,
    out: Queue<Request>,
    forge: Forge,
    disk: Checkout,
    hold: Option<Token>,
    trace: Vec<String>,
    commits: Vec<(Commit, Commit, u32, Vec<u8>)>,
    expected: temper_fake_checkout::git::Tree,
    ours: u64,
    base: u64,
}

impl World {
    fn new(seed: u64, case: Case, facts: u32) -> Self {
        let limits = Limits {
            workspaces: 1,
            repositories: 1,
            name_bytes: 64,
            message_bytes: 128,
            conflicts: 4,
            path_bytes: 128,
            remote_timeout: Duration::from_secs(10),
            local_timeout: Duration::from_secs(5),
            facts,
        };
        let mut forge = Forge::new(seed);
        let first = forge.repository(b"forge/temper", b"main", [(b"code".to_vec(), b"original\n".to_vec())].into());
        forge.create_branch(b"forge/temper", b"topic", first).expect("topic created");
        let left = format!("ours {seed}\n");
        let right = format!("base {seed}\n");
        let (ours, base, expected) = match case {
            Case::Contained => {
                let ours = forge.advance(b"forge/temper", b"topic", b"ours", left.as_bytes());
                (ours, first, forge.tree(ours))
            }
            Case::Unchanged | Case::Moved => {
                let base = forge.advance(b"forge/temper", b"main", b"code", right.as_bytes());
                (first, base, [(b"code".to_vec(), right.into_bytes())].into())
            }
            Case::Conflict => {
                let ours = forge.advance(b"forge/temper", b"topic", b"code", left.as_bytes());
                let base = forge.advance(b"forge/temper", b"main", b"code", right.as_bytes());
                (ours, base, [(b"code".to_vec(), b"resolved\n".to_vec())].into())
            }
            Case::Clean | Case::Resumed | Case::Ambiguous | Case::Saved => {
                let ours = forge.advance(b"forge/temper", b"topic", b"ours", left.as_bytes());
                let base = forge.advance(b"forge/temper", b"main", b"theirs", right.as_bytes());
                (
                    ours,
                    base,
                    [
                        (b"code".to_vec(), b"original\n".to_vec()),
                        (b"ours".to_vec(), left.into_bytes()),
                        (b"theirs".to_vec(), right.into_bytes()),
                    ]
                    .into(),
                )
            }
        };
        Self {
            domain: Domain::new(&limits),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            out: Queue::uncapped(),
            forge,
            disk: Checkout::new(),
            hold: None,
            trace: Vec::new(),
            commits: Vec::new(),
            expected,
            ours,
            base,
        }
    }

    fn finish(&mut self, mut event: Event, mut ambiguous: bool) -> Request {
        for _ in 0..64 {
            let preparing = matches!(event, Event::Prepare { .. });
            step(&mut self.domain, &self.env, event, &mut self.out);
            let mut pending = None;
            let mut terminal = None;
            let mut held = false;
            while let Some(request) = self.out.pop() {
                self.trace.push(format!("{request:?}"));
                match request {
                    Request::Held { hold, .. } => {
                        assert!(!held, "one hold per prepare");
                        held = true;
                        assert!(self.hold.replace(hold).is_none());
                    }
                    Request::Io { owner, op, deadline } => {
                        assert_eq!(Some(owner), self.hold);
                        let timeout =
                            if op.is_remote() { self.env.limits.remote_timeout } else { self.env.limits.local_timeout };
                        assert_eq!(deadline, self.env.now.saturating_add(timeout));
                        assert!(pending.replace((owner, op)).is_none(), "one IO operation per hold");
                    }
                    answer @ (Request::Prepared { .. }
                    | Request::Pushed { .. }
                    | Request::Saved { .. }
                    | Request::Released { .. }) => {
                        assert!(terminal.replace(answer).is_none(), "one terminal per caller operation");
                    }
                    Request::Cancel { .. } => panic!("these scenarios have no outstanding cancellation"),
                }
            }
            assert_eq!(held, preparing, "only a prepare emits a new hold");
            while self.domain.pop_fact().is_some() {}
            self.domain.reclaim();
            if let Some(terminal) = terminal {
                assert!(pending.is_none());
                return terminal;
            }
            let (owner, op) = pending.expect("caller waits for one IO operation");
            if let Op::Commit { parent, merging: Some(second), identity, title, .. } = &op {
                self.commits.push((*parent, *second, *identity, title.to_vec()));
            }
            let pushing = matches!(op, Op::Push { .. });
            let mut done = translate::perform(&mut self.forge, &mut self.disk, op);
            if ambiguous && pushing {
                assert_eq!(done, Done::Succeeded, "the timed-out push actually landed");
                done = Done::Failed { fault: Fault::TimedOut };
                ambiguous = false;
            }
            event = Event::Done { owner, done };
        }
        panic!("bounded scenario settles");
    }

    fn push(&mut self, saved: bool, ambiguous: bool) -> Box<[Landing]> {
        let hold = self.hold.expect("prepared");
        let message =
            Message { title: b"Merge checked tree".as_slice().into(), body: b"Keep both parents.".as_slice().into() };
        let event = if saved { Event::Save { hold, message } } else { Event::Push { hold, message } };
        match self.finish(event, ambiguous) {
            Request::Pushed { outcome: Outcome::Pushed { landings }, .. } if !saved => landings,
            Request::Saved { outcome: Outcome::Pushed { landings }, .. } if saved => landings,
            other @ (Request::Held { .. }
            | Request::Prepared { .. }
            | Request::Pushed { .. }
            | Request::Saved { .. }
            | Request::Released { .. }
            | Request::Io { .. }
            | Request::Cancel { .. }) => panic!("push/save terminal: {other:?}"),
        }
    }
}

/// Run one complete scenario, checking every promised terminal and observable
/// tree/ref against its independent prediction. Seed changes file content.
#[must_use]
pub fn run(seed: u64, case: Case, facts: u32) -> Report {
    let mut world = World::new(seed, case, facts);
    let spec = Spec {
        key: b"merge".as_slice().into(),
        repositories: Box::new([Repository {
            save: Some(b"saved".as_slice().into()),
            git: None,
            name: b"temper".as_slice().into(),
            remote: b"forge/temper".as_slice().into(),
            start: Start::Merge { branch: b"topic".as_slice().into(), base: translate::commit(world.base) },
            identity: 0,
            account: None,
            push: Some(b"topic".as_slice().into()),
            expected: Some(translate::commit(world.ours)),
            landing: if matches!(case, Case::Contained | Case::Resumed) {
                Some(b"main".as_slice().into())
            } else {
                None
            },
        }]),
    };
    world.forge.moves();
    let Request::Prepared { prepared: Prepared::Ready { workspace, conflicts }, .. } =
        world.finish(Event::Prepare { client: Token::new(9), spec }, false)
    else {
        panic!("prepared")
    };
    assert!(world.forge.moves().is_empty(), "preparation moves no remote refs");
    let at = translate::path(&temper_worker_domain_checkout::git::Place {
        git: None,
        workspace,
        repository: b"temper".as_slice().into(),
    });
    if matches!(case, Case::Conflict) {
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].repository, 0);
        assert_eq!(&*conflicts[0].files, &[Box::<[u8]>::from(&b"code"[..])]);
        assert_eq!(&*world.push(false, false), &[Landing::Conflicted { files: Box::new([b"code".as_slice().into()]) }]);
        assert!(world.forge.moves().is_empty(), "remaining markers make no remote effect");
        world.disk.write(&[&at[..], b"/code"].concat(), b"resolved\n");
    } else {
        assert!(conflicts.is_empty());
    }
    let prepared_merge = if matches!(case, Case::Resumed) { Some(resumed_merge(&mut world, &at)) } else { None };
    if matches!(case, Case::Moved) {
        assert_eq!(
            world.forge.push(b"forge/temper", b"topic", world.base, None).expect("peer fast-forward"),
            temper_fake_checkout::git::Pushed::Pushed
        );
    }
    if matches!(case, Case::Saved) {
        let saved = world.push(true, false);
        let Landing::Landed { commit } = saved[0] else { panic!("saved") };
        assert_eq!(world.forge.branch(b"forge/temper", b"saved"), Some(translate::fake(commit)));
        assert_eq!(world.forge.branch(b"forge/temper", b"topic"), Some(world.ours));
    }
    let landed = world.push(false, matches!(case, Case::Ambiguous));
    let commit = if matches!(case, Case::Moved) {
        assert_eq!(&*landed, &[Landing::Moved]);
        assert_eq!(world.forge.branch(b"forge/temper", b"topic"), Some(world.base));
        None
    } else {
        let Landing::Landed { commit } = landed[0] else { panic!("landed: {landed:?}") };
        Some(translate::fake(commit))
    };
    if let Some(commit) = commit {
        if let Some(prepared) = prepared_merge {
            assert_eq!(commit, prepared, "finish delivers the merge prepared before the run, without another commit");
        }
        if matches!(case, Case::Contained) {
            assert_eq!(commit, world.ours, "resume with nothing new keeps the reported head");
            assert_eq!(world.forge.merge_parent(commit), None, "resume adds no merge parent");
        } else {
            assert_eq!(world.forge.parent(commit), Some(world.ours));
            assert_eq!(world.forge.merge_parent(commit), Some(world.base));
        }
        assert_eq!(world.forge.tree(commit), world.expected);
        assert_eq!(world.forge.branch(b"forge/temper", b"topic"), Some(commit));
        let again = world.push(false, false);
        assert_eq!(again, landed, "a repeat is the same landing");
        assert_eq!(world.forge.branch(b"forge/temper", b"topic"), Some(commit), "repeat creates no new commit or push");
    }
    if matches!(case, Case::Contained) {
        resume_saved_work(&mut world, workspace);
    }
    if let Some(merge) = prepared_merge {
        new_work_on_merge(&mut world, &at, merge);
    }
    let hold = world.hold.expect("held");
    assert!(matches!(world.finish(Event::Release { hold }, false), Request::Released { .. }));
    assert_eq!(world.domain.holds(), 0);
    Report { trace: world.trace, tree: world.expected, parents: (world.ours, world.base) }
}

/// Read the fake's checkout and merge metadata before the run edits anything.
/// Saving must compare with the producer's original head, so this merge lands.
fn resumed_merge(world: &mut World, at: &[u8]) -> u64 {
    let [(ours, base, identity, title)] = world.commits.as_slice() else {
        panic!("prepare commits exactly one merge before ready")
    };
    assert_eq!((*ours, *base, *identity), (translate::commit(world.ours), translate::commit(world.base), 0));
    assert_eq!(title, expected_merge_message(world.base).as_bytes());
    assert!(
        world.trace.windows(3).any(|steps| {
            steps[0].contains("op: Commit {") && steps[1].contains("op: CheckOut {") && steps[2].contains("Ready {")
        }),
        "the trace commits and checks out before ready"
    );
    assert!(!world.disk.exists(&[at, b"/.git/MERGE_HEAD"].concat()), "no merge is left in progress");
    let head = world.disk.content(&[at, b"/.git/temper-head"].concat()).expect("checked-out head");
    let head = u64::from_le_bytes(head.try_into().expect("fake head spelling"));
    assert_eq!(world.forge.parent(head), Some(world.ours));
    assert_eq!(world.forge.merge_parent(head), Some(world.base));
    assert_eq!(world.forge.tree(head), world.expected);
    let mut tree = world.disk.tree(at);
    tree.retain(|path, _| !path.starts_with(b".git/"));
    assert_eq!(tree, world.expected, "the run sees the committed merge tree");
    assert_eq!(world.forge.branch(b"forge/temper", b"topic"), Some(world.ours));
    assert_eq!(world.forge.branch(b"forge/temper", b"main"), Some(world.base));
    assert_eq!(world.forge.branch(b"forge/temper", b"saved"), None);
    assert_eq!(&*world.push(true, false), &[Landing::Landed { commit: translate::commit(head) }]);
    assert_eq!(world.forge.branch(b"forge/temper", b"saved"), Some(head));
    assert_eq!(world.forge.branch(b"forge/temper", b"topic"), Some(world.ours));
    head
}

fn new_work_on_merge(world: &mut World, at: &[u8], merge: u64) {
    world.disk.write(&[at, b"/new-work"].concat(), b"after the clean merge\n");
    let landed = world.push(false, false);
    let [Landing::Landed { commit }] = &*landed else { panic!("new work delivered") };
    let head = translate::fake(*commit);
    assert_ne!(head, merge);
    assert_eq!(world.forge.parent(head), Some(merge));
    assert_eq!(world.forge.merge_parent(head), None, "new work has one parent");
    let mut expected = world.expected.clone();
    expected.insert(b"new-work".to_vec(), b"after the clean merge\n".to_vec());
    assert_eq!(world.forge.tree(head), expected);
}

/// A contained producer resumes the saved tree, then delivers a new ordinary commit.
fn resume_saved_work(world: &mut World, workspace: Token) {
    let at = translate::path(&temper_worker_domain_checkout::git::Place {
        git: None,
        workspace,
        repository: b"temper".as_slice().into(),
    });
    world.disk.write(&[at.as_slice(), b"/saved-only"].concat(), b"unfinished work\n");
    let saved = world.push(true, false);
    let Landing::Landed { commit: saved } = saved[0] else { panic!("saved tree") };
    let saved_head = translate::fake(saved);
    let expected = world.forge.tree(saved_head);
    assert_eq!(world.forge.parent(saved_head), Some(world.ours));
    assert_eq!(world.forge.merge_parent(saved_head), None);
    let hold = world.hold.take().expect("first held");
    world.finish(Event::Release { hold }, false);
    let spec = Spec {
        key: b"merge".as_slice().into(),
        repositories: Box::new([Repository {
            name: b"temper".as_slice().into(),
            remote: b"forge/temper".as_slice().into(),
            git: None,
            start: Start::Merge { branch: b"saved".as_slice().into(), base: translate::commit(world.base) },
            identity: 0,
            account: None,
            push: Some(b"topic".as_slice().into()),
            expected: None,
            landing: Some(b"main".as_slice().into()),
            save: Some(b"saved".as_slice().into()),
        }]),
    };
    let ready = world.finish(Event::Prepare { client: Token::new(10), spec }, false);
    assert!(matches!(ready, Request::Prepared { prepared: Prepared::Ready { .. }, .. }));
    assert_eq!(&*world.push(false, false), &[Landing::Landed { commit: saved }]);
    assert_eq!(world.forge.branch(b"forge/temper", b"topic"), Some(saved_head));
    assert_eq!(world.forge.tree(saved_head), expected, "saved tree survives the resume");
    assert_eq!(world.forge.merge_parent(saved_head), None);
    world.disk.write(&[at.as_slice(), b"/new-work"].concat(), b"after resume\n");
    let edited = world.push(false, false);
    let Landing::Landed { commit: edited } = edited[0] else { panic!("new work delivered") };
    let edited = translate::fake(edited);
    assert_ne!(edited, saved_head);
    assert_eq!(world.forge.parent(edited), Some(saved_head));
    assert_eq!(world.forge.merge_parent(edited), None, "new work has one parent");
    assert_eq!(world.forge.tree(edited).get(b"saved-only".as_slice()), Some(&b"unfinished work\n".to_vec()));
}

/// A separate activation starts from the pushed topic, without editing it.
/// Move is applied after preparation: an advance or an ancestor rewind.
pub fn retry_existing_change(movement: Option<bool>, ambiguous: bool) {
    let mut world = World::new(91, Case::Unchanged, 64);
    let base = world.forge.advance(b"forge/temper", b"main", b"README.md", b"# app\n");
    let spec = |retry| Spec {
        key: b"retry".as_slice().into(),
        repositories: Box::new([Repository {
            save: Some(b"saved".as_slice().into()),
            git: None,
            name: b"temper".as_slice().into(),
            remote: b"forge/temper".as_slice().into(),
            start: Start::Branch { branch: if retry { b"topic".as_slice().into() } else { b"main".as_slice().into() } },
            identity: 0,
            account: None,
            push: Some(b"topic".as_slice().into()),
            expected: None,
            landing: if retry { Some(b"main".as_slice().into()) } else { None },
        }]),
    };
    let Request::Prepared { prepared: Prepared::Ready { workspace, .. }, .. } =
        world.finish(Event::Prepare { client: Token::new(1), spec: spec(false) }, false)
    else {
        panic!("first ready")
    };
    let path = translate::path(&temper_worker_domain_checkout::git::Place {
        git: None,
        workspace,
        repository: b"temper".as_slice().into(),
    });
    world.disk.write(&[path.as_slice(), b"/README.md"].concat(), b"# app\nChanged by temper.\n");
    let first = world.push(false, false);
    let Landing::Landed { commit: head } = first[0] else { panic!("first landed") };
    let first_hold = world.hold.take().expect("first held");
    world.finish(Event::Release { hold: first_hold }, false);
    assert!(matches!(
        world.finish(Event::Prepare { client: Token::new(2), spec: spec(true) }, false),
        Request::Prepared { prepared: Prepared::Ready { .. }, .. }
    ));
    assert_ne!(world.hold, Some(first_hold));
    let fake = translate::fake(head);
    assert_eq!(world.forge.parent(fake), Some(base));
    assert_eq!(world.forge.merge_parent(fake), None);
    assert_eq!(
        world.forge.tree(fake).get(b"README.md".as_slice()).expect("preserved README"),
        b"# app\nChanged by temper.\n"
    );
    let remote = match movement {
        Some(true) => world.forge.advance(b"forge/temper", b"topic", b"peer", b"advanced\n"),
        Some(false) => {
            use skein_fake_forge_domain::api as raw;
            world
                .forge
                .api(
                    b"forge/temper",
                    2,
                    raw::Op::Write(raw::Write::DeleteBranch { branch: b"topic".as_slice().into() }),
                )
                .expect("peer deletes topic");
            world.forge.create_branch(b"forge/temper", b"topic", base).expect("peer rewinds topic");
            base
        }
        None => fake,
    };
    let retry = world.push(false, ambiguous);
    assert_eq!(&*retry, if movement.is_some() { &[Landing::Moved][..] } else { &*first });
    assert_eq!(world.forge.branch(b"forge/temper", b"topic"), Some(remote));
    if movement.is_none() {
        assert_eq!(world.forge.parent(remote), Some(base), "retry created no extra commit");
        assert_eq!(world.forge.merge_parent(remote), None);
        assert_eq!(&*world.push(true, false), &[Landing::Unchanged], "save still compares actual start");
    }
    let hold = world.hold.take().expect("second held");
    world.finish(Event::Release { hold }, false);
    assert_eq!(world.domain.holds(), 0);
}

fn heads_spec(heads: &[u64]) -> Spec {
    Spec {
        key: b"heads".as_slice().into(),
        repositories: Box::new([Repository {
            name: b"temper".as_slice().into(),
            remote: b"forge/temper".as_slice().into(),
            git: None,
            start: Start::Merged {
                branch: b"main".as_slice().into(),
                heads: heads.iter().copied().map(translate::commit).collect(),
            },
            identity: 17,
            account: None,
            push: Some(b"early".as_slice().into()),
            expected: None,
            landing: None,
            save: Some(b"saved".as_slice().into()),
        }]),
    }
}

fn prepare_heads(world: &mut World, heads: &[u64]) -> (Token, Vec<u8>) {
    let Request::Prepared { prepared: Prepared::Ready { workspace, conflicts }, .. } =
        world.finish(Event::Prepare { client: Token::new(9), spec: heads_spec(heads) }, false)
    else {
        panic!("merged heads prepared")
    };
    assert!(conflicts.is_empty());
    let at = translate::path(&temper_worker_domain_checkout::git::Place {
        workspace,
        repository: b"temper".as_slice().into(),
        git: None,
    });
    (workspace, at)
}

fn release_heads(world: &mut World) {
    let hold = world.hold.take().expect("held until release");
    assert!(matches!(world.finish(Event::Release { hold }, false), Request::Released { .. }));
    assert_eq!(world.domain.holds(), 0);
}

/// Each prepare merge commits locally; delivery creates one branch holding all heads.
pub fn merged_heads() {
    let mut world = World::new(73, Case::Clean, 64);
    let root = world.forge.parent(world.ours).expect("fork point");
    world.forge.create_branch(b"forge/temper", b"second", root).expect("second head");
    let second = world.forge.advance(b"forge/temper", b"second", b"second", b"head two\n");
    world.expected.insert(b"second".to_vec(), b"head two\n".to_vec());
    world.forge.moves();
    let heads = [world.ours, second];
    prepare_heads(&mut world, &heads);
    assert!(world.forge.moves().is_empty(), "prepare moves no remote branch");
    assert_eq!(&*world.push(true, false), &[Landing::Unchanged], "merged tree is the save baseline");
    let pushed = world.push(false, false);
    let Landing::Landed { commit } = pushed[0] else { panic!("first delivery lands") };
    let last = translate::fake(commit);
    let first = world.forge.parent(last).expect("first merge");
    assert_eq!(world.forge.parent(first), Some(world.base));
    assert_eq!(world.forge.merge_parent(first), Some(world.ours));
    assert_eq!(world.forge.merge_parent(last), Some(second));
    assert_eq!(world.forge.tree(last), world.expected);
    assert_eq!(world.forge.branch(b"forge/temper", b"early"), Some(last));
    assert_eq!(world.forge.branch(b"forge/temper", b"saved"), None);
    // The fake store records full messages and both parents; the operation names its author.
    let expected_first = expected_merge_message(world.ours);
    let expected_second = expected_merge_message(second);
    assert_eq!(world.forge.message(first), expected_first.as_bytes());
    assert_eq!(world.forge.message(last), expected_second.as_bytes());
    assert_eq!(
        world.commits,
        vec![
            (translate::commit(world.base), translate::commit(world.ours), 17, expected_first.into_bytes()),
            (translate::commit(first), translate::commit(second), 17, expected_second.into_bytes()),
        ]
    );
    release_heads(&mut world);
}

/// A second head conflicts; release and retry discard every partial merge and marker.
pub fn conflicting_head() {
    let mut world = World::new(79, Case::Clean, 64);
    let root = world.forge.parent(world.ours).expect("fork point");
    world.base = world.forge.advance(b"forge/temper", b"main", b"code", b"base edit\n");
    world.forge.create_branch(b"forge/temper", b"second", root).expect("second head");
    let second = world.forge.advance(b"forge/temper", b"second", b"code", b"conflicting edit\n");
    world.forge.moves();
    let failed = world.finish(Event::Prepare { client: Token::new(9), spec: heads_spec(&[world.ours, second]) }, false);
    assert!(matches!(failed, Request::Prepared { prepared: Prepared::Failed {
        failure: Failure::Conflicted { repository: 0, head },
    }, .. } if head == translate::commit(second)));
    assert_eq!(world.domain.holds(), 1);
    let hold = world.hold.expect("failed hold stays");
    for event in [
        Event::Push { hold, message: Message { title: b"push".as_slice().into(), body: Box::new([]) } },
        Event::Save { hold, message: Message { title: b"save".as_slice().into(), body: Box::new([]) } },
    ] {
        assert!(matches!(
            world.finish(event, false),
            Request::Pushed { outcome: Outcome::Refused { .. }, .. }
                | Request::Saved { outcome: Outcome::Refused { .. }, .. }
        ));
    }
    assert!(world.forge.moves().is_empty(), "failed preparation neither pushes nor saves");
    assert_eq!(world.forge.branch(b"forge/temper", b"early"), None);
    assert_eq!(world.forge.branch(b"forge/temper", b"saved"), None);
    release_heads(&mut world);
    let heads = [world.base];
    let (_, at) = prepare_heads(&mut world, &heads);
    let mut expected = world.forge.tree(world.base);
    expected.insert(b"code".to_vec(), b"base edit\n".to_vec());
    let mut tree = world.disk.tree(&at);
    tree.retain(|path, _| !path.starts_with(b".git/"));
    assert_eq!(tree, expected, "next prepare resets partial merges and markers");
    assert_eq!(&*world.push(true, false), &[Landing::Unchanged]);
    assert_eq!(&*world.push(false, false), &[Landing::Unchanged]);
    assert!(world.forge.moves().is_empty());
    release_heads(&mut world);
}

fn expected_merge_message(head: u64) -> String {
    use std::fmt::Write as _;
    let mut message = String::from("Merge ");
    for byte in skein_fake_forge_domain::object_bytes(head) {
        write!(&mut message, "{byte:02x}").expect("write to string");
    }
    message
}
