//! Merge scenarios against the real checkout child, fake disk and fake forge.
//! The referee predicts complete trees, parent pairs and remote refs from
//! the scenario; it never reads the domain's private state.
use crate::{forge::Forge, translate};
use skein_lib::{Duration, Env, Queue, Time, Token, Wall};
use temper_fake_checkout::{Checkout, git::Remote};
use temper_worker_domain_checkout::git::{Commit, Done, Fault, Op};
use temper_worker_domain_checkout::{
    Conflicts, Domain, Event, Failure, Landing, Limits, Message, Outcome, Prepared, Repository, Request, Spec, Start,
    step,
};

#[derive(Clone, Copy, Debug)]
pub enum Case {
    Clean,
    Resumed,
    Contained,
    Conflict,
    PastTheBound,
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
    pins: usize,
    branch_fetches: Vec<Vec<u8>>,
    push_races: usize,
    push_attempts: usize,
    expected: temper_fake_checkout::git::Tree,
    ours: u64,
    base: u64,
}

impl World {
    fn new(seed: u64, case: Case, facts: u32) -> Self {
        let limits = Limits {
            workspaces: 1,
            preparations: 1,
            repositories: 1,
            name_bytes: 64,
            message_bytes: 128,
            conflicts: 4,
            path_bytes: 128,
            remote_timeout: Duration::from_secs(10),
            reds_read_ms: 5_000,
            rerun_processes: 4,
            local_timeout: Duration::from_secs(5),
            facts,
        };
        let mut forge = Forge::new(seed);
        let initial = if matches!(case, Case::PastTheBound) {
            (0..=limits.conflicts).map(|n| (format!("code{n}").into_bytes(), b"original\n".to_vec())).collect()
        } else {
            [(b"code".to_vec(), b"original\n".to_vec())].into()
        };
        let first = forge.repository(b"forge/temper", b"main", initial);
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
            Case::PastTheBound => {
                let mut ours = first;
                let mut base = first;
                let mut expected = temper_fake_checkout::git::Tree::new();
                for n in 0..=limits.conflicts {
                    let path = format!("code{n}").into_bytes();
                    ours = forge.advance(b"forge/temper", b"topic", &path, left.as_bytes());
                    base = forge.advance(b"forge/temper", b"main", &path, right.as_bytes());
                    expected.insert(path, b"resolved\n".to_vec());
                }
                (ours, base, expected)
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
            pins: 0,
            branch_fetches: Vec::new(),
            push_races: 0,
            push_attempts: 0,
            expected,
            ours,
            base,
        }
    }

    fn finish(&mut self, mut event: Event, mut ambiguous: bool) -> Request {
        let merge_starts: Vec<_> = if let Event::Prepare { spec, .. } = &event {
            spec.repositories.iter().map(|repo| matches!(repo.start, Start::Merge { .. })).collect()
        } else {
            Vec::new()
        };
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
                    Request::Results { .. } | Request::ResultsEnd { .. } | Request::Cancel { .. } => {
                        panic!("these scenarios have no outstanding cancellation")
                    }
                }
            }
            assert_eq!(held, preparing, "only a prepare emits a new hold");
            while self.domain.pop_fact().is_some() {}
            self.domain.reclaim();
            if let Some(terminal) = terminal {
                assert!(pending.is_none());
                if let Request::Prepared {
                    prepared: Prepared::Failed { failure: Failure::Conflicted { repository, .. }, .. },
                    ..
                } = &terminal
                {
                    assert!(
                        !merge_starts[usize::try_from(*repository).expect("bounded repository")],
                        "a Start::Merge prepare never ends Failure::Conflicted"
                    );
                }
                return terminal;
            }
            let (owner, op) = pending.expect("caller waits for one IO operation");
            if let Op::Commit { parent, merging: Some(second), identity, title, .. } = &op {
                self.commits.push((*parent, *second, *identity, title.to_vec()));
            }
            if matches!(
                op,
                Op::Fetch { into: temper_worker_domain_checkout::git::Into::Landing, .. }
                    | Op::Point { into: temper_worker_domain_checkout::git::Into::Landing, .. }
            ) {
                self.pins += 1;
            }
            if let Op::Fetch { want: temper_worker_domain_checkout::git::Want::Branch { branch }, .. } = &op {
                self.branch_fetches.push(branch.to_vec());
            }
            let pushing = matches!(op, Op::Push { .. });
            if pushing {
                if let Op::Push { branch, expected, commit, .. } = &op
                    && branch.as_ref() == b"g"
                {
                    let tip = self.forge.branch(b"forge/temper", b"g").expect("goal exists");
                    assert_eq!(*expected, Some(translate::commit(tip)), "each design push leases the fetched tip");
                    assert!(
                        self.forge.is_ancestor(tip, translate::fake(*commit)),
                        "every design push is a fast-forward"
                    );
                }
                self.push_attempts += 1;
                if self.push_races > 0 {
                    self.push_races -= 1;
                    self.forge.advance(b"forge/temper", b"g", b"src/race.rs", self.push_races.to_string().as_bytes());
                }
            }
            let mut done = translate::perform_within(&mut self.forge, &mut self.disk, op, self.env.limits.conflicts);
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
            other @ (Request::Results { .. }
            | Request::ResultsEnd { .. }
            | Request::Held { .. }
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
#[expect(clippy::too_many_lines, reason = "the merge scenario checks its prepare pin and resulting tree together")]
pub fn run(seed: u64, case: Case, facts: u32) -> Report {
    let mut world = World::new(seed, case, facts);
    let spec = Spec {
        key: b"merge".as_slice().into(),
        repositories: Box::new([Repository {
            prefixes: Box::new([]),
            save: Some(b"saved".as_slice().into()),
            git: None,
            name: b"temper".as_slice().into(),
            remote: b"forge/temper".as_slice().into(),
            start: Start::Merge { branch: b"topic".as_slice().into(), base: translate::commit(world.base) },
            identity: 0,
            account: None,
            push: Some(b"topic".as_slice().into()),
            expected: Some(translate::commit(world.ours)),
            landing: if matches!(case, Case::Contained | Case::Resumed | Case::Conflict) {
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
    if matches!(case, Case::Contained | Case::Resumed | Case::Conflict) {
        assert_eq!(world.pins, 1, "one pin during prepare, contained starts included");
        assert_eq!(
            world.disk.content(&[at.as_slice(), b"/.git/refs/temper/landing"].concat()),
            Some(world.base.to_le_bytes().as_slice()),
            "prepare pins the landing tip"
        );
    } else {
        assert_eq!(world.pins, 1, "a resolution pins its replaced head once");
        assert_eq!(
            world.disk.content(&[at.as_slice(), b"/.git/refs/temper/landing"].concat()),
            Some(world.ours.to_le_bytes().as_slice()),
            "a resolution pins its branch before the merge"
        );
    }
    if matches!(case, Case::Conflict) {
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].repository, 0);
        assert_eq!(&*conflicts[0].files, &[Box::<[u8]>::from(&b"code"[..])]);
        assert_eq!(&*world.push(false, false), &[Landing::Conflicted { files: Box::new([b"code".as_slice().into()]) }]);
        assert!(world.forge.moves().is_empty(), "remaining markers make no remote effect");
        world.disk.write(&[&at[..], b"/code"].concat(), b"resolved\n");
    } else if matches!(case, Case::PastTheBound) {
        settle_past_the_bound(&mut world, &at, &conflicts, seed);
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
    assert_eq!(world.pins, 1, "pushes and verification never move the pin");
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

#[derive(Clone, Copy, Debug)]
pub enum PinCase {
    Repair,
    Saved,
    Resolution,
    NewBranch,
    ReadOnly,
}

/// Reuse a producer's checkout, then check the replacement pin through two deliveries.
pub fn replaced_pin(case: PinCase) {
    let mut world = World::new(73, Case::Clean, 64);
    let repository = || Repository {
        prefixes: Box::new([]),
        name: b"temper".as_slice().into(),
        remote: b"forge/temper".as_slice().into(),
        git: None,
        start: Start::Branch { branch: b"topic".as_slice().into() },
        identity: 0,
        account: None,
        push: Some(b"topic".as_slice().into()),
        expected: None,
        landing: Some(b"main".as_slice().into()),
        save: Some(b"saved".as_slice().into()),
    };
    let spec = |repository| Spec { key: b"merge".as_slice().into(), repositories: Box::new([repository]) };
    let Request::Prepared { prepared: Prepared::Ready { workspace, .. }, .. } =
        world.finish(Event::Prepare { client: Token::new(1), spec: spec(repository()) }, false)
    else {
        panic!("producer ready")
    };
    let at = translate::path(&temper_worker_domain_checkout::git::Place {
        git: None,
        workspace,
        repository: b"temper".as_slice().into(),
    });
    let pin = [at.as_slice(), b"/.git/refs/temper/landing"].concat();
    assert_eq!(world.disk.content(&pin), Some(world.base.to_le_bytes().as_slice()));
    let hold = world.hold.take().expect("producer held");
    world.finish(Event::Release { hold }, false);
    world.pins = 0;
    world.branch_fetches.clear();
    let mut repository = repository();
    repository.landing = None;
    let mut expected = world.ours;
    match case {
        PinCase::Repair => {}
        PinCase::Saved => {
            world.forge.create_branch(b"forge/temper", b"saved", world.ours).expect("saved branch");
            let saved = world.forge.advance(b"forge/temper", b"saved", b"saved-only", b"unfinished\n");
            assert_ne!(saved, expected);
            repository.start = Start::Saved { branch: b"saved".as_slice().into() };
        }
        PinCase::Resolution => {
            repository.start = Start::Merge { branch: b"topic".as_slice().into(), base: translate::commit(world.base) };
        }
        PinCase::NewBranch => {
            repository.start = Start::Commit { commit: translate::commit(world.ours) };
            repository.push = Some(b"new".as_slice().into());
        }
        PinCase::ReadOnly => {
            repository.push = None;
            repository.landing = Some(b"main".as_slice().into());
            expected = world.base;
        }
    }
    let Request::Prepared { prepared: Prepared::Ready { workspace: reused, .. }, .. } =
        world.finish(Event::Prepare { client: Token::new(2), spec: spec(repository) }, false)
    else {
        panic!("replacement ready")
    };
    assert_eq!(reused, workspace, "the producer's cached checkout is reused");
    if matches!(case, PinCase::Repair) {
        assert_eq!(world.branch_fetches, [b"topic".to_vec()], "one fetch serves start, baseline and pin");
    }
    assert_eq!(world.pins, usize::from(!matches!(case, PinCase::ReadOnly)), "one pin per writable hold");
    assert_eq!(
        world.disk.content(&pin),
        Some(expected.to_le_bytes().as_slice()),
        "prepare replaces the cached producer pin, not the saved start"
    );
    if !matches!(case, PinCase::ReadOnly) {
        for content in [b"first\n".as_slice(), b"second\n".as_slice()] {
            world.disk.write(&[at.as_slice(), b"/repair"].concat(), content);
            let landings = world.push(false, false);
            assert!(matches!(landings.as_ref(), [Landing::Landed { .. }]));
            assert_eq!(
                world.disk.content(&pin),
                Some(expected.to_le_bytes().as_slice()),
                "deliveries keep the prepare pin"
            );
            assert_eq!(world.pins, 1, "deliveries never fetch a new pin");
        }
    }
    let hold = world.hold.take().expect("replacement held");
    world.finish(Event::Release { hold }, false);
}

/// A resolution receives the conflict whether or not its change has pushed a head.
pub fn resolution_conflict(pushed: bool) {
    let mut world = World::new(73, Case::Conflict, 64);
    let (branch, push, ours, base) = if pushed {
        (b"topic".as_slice(), b"topic".as_slice(), world.ours, world.base)
    } else {
        // The base branch is one side of the conflict; the incoming base is the other.
        (b"main".as_slice(), b"resolution".as_slice(), world.base, world.ours)
    };
    let expected = if pushed { Some(translate::commit(ours)) } else { None };
    assert_eq!(world.forge.branch(b"forge/temper", push), expected.map(translate::fake));
    let spec = Spec {
        key: b"resolution".as_slice().into(),
        repositories: Box::new([Repository {
            save: None,
            git: None,
            name: b"temper".as_slice().into(),
            remote: b"forge/temper".as_slice().into(),
            start: Start::Merge { branch: branch.into(), base: translate::commit(base) },
            identity: 0,
            account: None,
            push: Some(push.into()),
            prefixes: Box::new([]),
            expected,
            landing: None,
        }]),
    };
    world.forge.moves();
    let ready = world.finish(Event::Prepare { client: Token::new(9), spec }, false);
    let Request::Prepared { prepared: Prepared::Ready { workspace, conflicts }, .. } = ready else {
        panic!("resolution ready with conflicts: {ready:?}")
    };
    assert_eq!(&*conflicts, &[Conflicts { repository: 0, files: Box::new([b"code".as_slice().into()]) }]);
    let at = translate::path(&temper_worker_domain_checkout::git::Place {
        git: None,
        workspace,
        repository: b"temper".as_slice().into(),
    });
    let pin = [at.as_slice(), b"/.git/refs/temper/landing"].concat();
    assert_eq!(world.pins, 1, "one replaced-head pin per hold");
    assert_eq!(world.disk.content(&pin), Some(ours.to_le_bytes().as_slice()));
    assert_eq!(
        world.disk.content(&[at.as_slice(), b"/.git/temper-head"].concat()),
        Some(ours.to_le_bytes().as_slice())
    );
    assert_eq!(world.disk.content(&[at.as_slice(), b"/.git/MERGE_HEAD"].concat()), Some(base.to_le_bytes().as_slice()));
    assert!(world.commits.is_empty(), "prepare leaves the merge in progress");
    assert_eq!(&*world.push(false, false), &[Landing::Conflicted { files: Box::new([b"code".as_slice().into()]) }]);
    assert!(world.forge.moves().is_empty(), "unsettled files neither move nor create a branch");
    world.disk.write(&[at.as_slice(), b"/code"].concat(), b"resolved\n");
    let landings = world.push(false, false);
    let [Landing::Landed { commit }] = landings.as_ref() else { panic!("resolved merge lands: {landings:?}") };
    let landed = translate::fake(*commit);
    assert_eq!(world.forge.parent(landed), Some(ours));
    assert_eq!(world.forge.merge_parent(landed), Some(base));
    assert_eq!(world.forge.tree(landed), world.expected);
    assert_eq!(world.forge.branch(b"forge/temper", push), Some(landed));
    assert_eq!(world.forge.moves().len(), 1, "one push creates or replaces the change's branch");
    assert_eq!(world.commits.len(), 2, "the refused commit and the single settled merge commit");
    assert_eq!(world.pins, 1, "push does not move the pin");
    assert_eq!(world.disk.content(&pin), Some(ours.to_le_bytes().as_slice()));
    let hold = world.hold.take().expect("resolution held");
    assert!(matches!(world.finish(Event::Release { hold }, false), Request::Released { .. }));
    assert_eq!(world.domain.holds(), 0);
}

/// The named prefix leaves every conflicted file and the merge state on disk.
fn settle_past_the_bound(world: &mut World, at: &[u8], conflicts: &[Conflicts], seed: u64) {
    let named: Box<[Box<[u8]>]> =
        (0..world.env.limits.conflicts).map(|n| format!("code{n}").into_bytes().into()).collect();
    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0].repository, 0);
    assert_eq!(conflicts[0].files, named);
    assert_eq!(
        world.disk.content(&[at, b"/.git/MERGE_HEAD"].concat()),
        Some(world.base.to_le_bytes().as_slice()),
        "the merge remains in progress at the base"
    );
    let marked = format!("<<<<<<< ours\nours {seed}\n=======\nbase {seed}\n>>>>>>> theirs\n");
    let unmerged = world.disk.tree(&[at, b"/.git/temper-conflicts"].concat());
    assert_eq!(unmerged.into_values().collect::<Vec<_>>(), world.expected.keys().cloned().collect::<Vec<_>>());
    for file in world.expected.keys() {
        assert_eq!(world.disk.content(&[at, b"/", file].concat()), Some(marked.as_bytes()));
    }
    for (file, content) in &world.expected {
        world.disk.write(&[at, b"/", file].concat(), content);
    }
}

/// Read the fake's checkout and merge metadata before the run edits anything.
/// Saving must compare with the producer's original head, so this merge lands.
fn resumed_merge(world: &mut World, at: &[u8]) -> u64 {
    let [(ours, base, identity, title)] = world.commits.as_slice() else {
        panic!("prepare commits exactly one merge before ready")
    };
    assert_eq!((*ours, *base, *identity), (translate::commit(world.ours), translate::commit(world.base), 0));
    assert_eq!(title, expected_merge_message(world.base).as_bytes());
    let trace: Vec<_> = world.trace.iter().filter(|line| !line.contains("op: Reds {")).collect();
    assert!(
        trace.windows(3).any(|steps| {
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
            prefixes: Box::new([]),
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
            prefixes: Box::new([]),
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
            prefixes: Box::new([]),
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
    world.forge.create_branch(b"forge/temper", b"early/base", world.base).expect("early base branch");
    world.forge.moves();
    let mut spec = heads_spec(&[world.ours, second]);
    let Start::Merged { branch, .. } = &mut spec.repositories[0].start else { panic!("merged start") };
    *branch = b"early/base".as_slice().into();
    let failed = world.finish(Event::Prepare { client: Token::new(9), spec }, false);
    let Request::Prepared { prepared: Prepared::Failed { failure, detail }, .. } = failed else {
        panic!("a conflicting merged head fails: {failed:?}")
    };
    assert_eq!(failure, Failure::Conflicted { repository: 0, head: translate::commit(second) });
    let title = expected_merge_message(second);
    let hex = title.strip_prefix("Merge ").expect("merge title prefix");
    assert_eq!(detail.as_ref(), format!("merge: Conflicted: head {hex} does not merge into early/base").as_bytes());
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
    assert_eq!(world.forge.moves().len(), 0);
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

/// A goal starts on main (g), brings its run head in, and writes only topic (r)
/// (temper/docs/design/domain/worker/checkouts.md).
#[expect(clippy::too_many_lines, reason = "one scenario holds the goal start, its pin and delivery together")]
pub fn goal_later_run(case: Case) {
    let mut world = World::new(73, case, 64);
    assert!(matches!(case, Case::Contained | Case::Resumed | Case::Conflict));
    if matches!(case, Case::Contained) {
        assert_eq!(
            world.forge.push(b"forge/temper", b"main", world.ours, None).expect("g holds r"),
            temper_fake_checkout::git::Pushed::Pushed
        );
        world.base = world.forge.advance(b"forge/temper", b"main", b"goal-only", b"landed in g\n");
        world.expected = world.forge.tree(world.base);
    }
    world.forge.moves();
    let spec = Spec {
        key: b"goal".as_slice().into(),
        repositories: Box::new([Repository {
            prefixes: Box::new([]),
            name: b"temper".as_slice().into(),
            remote: b"forge/temper".as_slice().into(),
            git: None,
            start: Start::Merge { branch: b"main".as_slice().into(), base: translate::commit(world.ours) },
            identity: 17,
            account: None,
            push: Some(b"topic".as_slice().into()),
            expected: None,
            landing: Some(b"main".as_slice().into()),
            save: Some(b"saved".as_slice().into()),
        }]),
    };
    let Request::Prepared { prepared: Prepared::Ready { workspace, conflicts }, .. } =
        world.finish(Event::Prepare { client: Token::new(9), spec }, false)
    else {
        panic!("goal start prepared")
    };
    let at = translate::path(&temper_worker_domain_checkout::git::Place {
        git: None,
        workspace,
        repository: b"temper".as_slice().into(),
    });
    assert!(world.forge.moves().is_empty(), "prepare moves neither g nor r");
    assert_eq!(
        world.branch_fetches.iter().filter(|branch| branch.as_slice() == b"main").count(),
        1,
        "g is fetched once"
    );
    assert_eq!(
        world.disk.content(&[at.as_slice(), b"/.git/refs/temper/landing"].concat()),
        Some(world.base.to_le_bytes().as_slice()),
        "the pin stays at g's tip"
    );
    let prepared = match case {
        Case::Contained => {
            assert!(conflicts.is_empty());
            assert!(world.commits.is_empty(), "g already contains r: prepare adds no merge");
            assert!(!world.disk.exists(&[at.as_slice(), b"/.git/MERGE_HEAD"].concat()));
            Some(world.base)
        }
        Case::Resumed => {
            assert!(conflicts.is_empty());
            assert_eq!(
                world.commits,
                vec![(
                    translate::commit(world.base),
                    translate::commit(world.ours),
                    17,
                    expected_merge_message(world.ours).into_bytes(),
                )],
                "the clean merge into g is committed before the run"
            );
            assert!(!world.disk.exists(&[at.as_slice(), b"/.git/MERGE_HEAD"].concat()));
            let head = world.disk.content(&[at.as_slice(), b"/.git/temper-head"].concat()).expect("prepared head");
            Some(u64::from_le_bytes(head.try_into().expect("fake head spelling")))
        }
        Case::Conflict => {
            assert_eq!(conflicts.len(), 1);
            assert_eq!(conflicts[0].repository, 0);
            assert_eq!(&*conflicts[0].files, &[Box::<[u8]>::from(b"code".as_slice())]);
            assert!(world.commits.is_empty(), "the conflicting merge stays in progress");
            assert!(world.disk.exists(&[at.as_slice(), b"/.git/MERGE_HEAD"].concat()));
            assert_eq!(
                &*world.push(false, false),
                &[Landing::Conflicted { files: Box::new([b"code".as_slice().into()]) }]
            );
            assert!(world.forge.moves().is_empty(), "markers prevent delivery");
            world.disk.write(&[at.as_slice(), b"/code"].concat(), b"resolved\n");
            None
        }
        Case::Clean | Case::PastTheBound | Case::Unchanged | Case::Moved | Case::Ambiguous | Case::Saved => {
            unreachable!("goal cases")
        }
    };
    let mut tree = world.disk.tree(&at);
    tree.retain(|path, _| !path.starts_with(b".git/"));
    assert_eq!(tree, world.expected, "the run sees g with r's work");
    assert_eq!(world.forge.branch(b"forge/temper", b"topic"), Some(world.ours));
    let pushed = world.push(false, false);
    let [Landing::Landed { commit }] = &*pushed else { panic!("delivery fast-forwards r: {pushed:?}") };
    let head = translate::fake(*commit);
    if let Some(prepared) = prepared {
        assert_eq!(head, prepared, "empty delivery pushes the prepared head without another commit");
    }
    if matches!(case, Case::Contained) {
        assert_eq!(world.forge.parent(head), Some(world.ours));
        assert_eq!(world.forge.merge_parent(head), None);
    } else {
        assert_eq!(world.forge.parent(head), Some(world.base));
        assert_eq!(world.forge.merge_parent(head), Some(world.ours));
    }
    assert_eq!(world.forge.tree(head), world.expected);
    assert_eq!(world.forge.branch(b"forge/temper", b"topic"), Some(head));
    assert_eq!(world.forge.branch(b"forge/temper", b"main"), Some(world.base), "g is never pushed");
    assert_eq!(world.push(false, false), pushed, "a repeated delivery keeps the same head");
    release_heads(&mut world);
}

fn design_start() -> (World, Vec<u8>, u64) {
    let mut world = World::new(73, Case::Clean, 64);
    // Use the same forge and checkout driver as the merge scenarios.
    world.forge.create_branch(b"forge/temper", b"g", world.ours).expect("g created");
    let spec = Spec {
        key: b"design".as_slice().into(),
        repositories: Box::new([Repository {
            prefixes: Box::new([]),
            name: b"temper".as_slice().into(),
            remote: b"forge/temper".as_slice().into(),
            git: None,
            start: Start::Branch { branch: b"g".as_slice().into() },
            identity: 17,
            account: None,
            push: Some(b"g".as_slice().into()),
            expected: Some(translate::commit(world.ours)),
            landing: Some(b"g".as_slice().into()),
            save: Some(b"saved".as_slice().into()),
        }]),
    };
    let Request::Prepared { prepared: Prepared::Ready { workspace, conflicts }, .. } =
        world.finish(Event::Prepare { client: Token::new(9), spec }, false)
    else {
        panic!("design prepared")
    };
    assert!(conflicts.is_empty());
    let at = translate::path(&temper_worker_domain_checkout::git::Place {
        git: None,
        workspace,
        repository: b"temper".as_slice().into(),
    });
    assert_eq!(world.branch_fetches, [b"g".to_vec()], "g is fetched once during prepare");
    assert_eq!(
        world.disk.content(&[at.as_slice(), b"/.git/refs/temper/landing"].concat()),
        Some(world.ours.to_le_bytes().as_slice())
    );
    let mut tree = world.disk.tree(&at);
    tree.retain(|path, _| !path.starts_with(b".git/"));
    assert_eq!(tree, world.forge.tree(world.ours), "the run starts on g's tree");
    world.forge.moves();
    let tip = world.ours;
    (world, at, tip)
}

pub fn design_fast_forward() {
    let (mut world, at, tip) = design_start();
    world.disk.write(&[at.as_slice(), b"/docs/a.md"].concat(), b"design\n");
    let landed = world.push(false, false);
    let [Landing::Landed { commit }] = &*landed else { panic!("design landed: {landed:?}") };
    let head = translate::fake(*commit);
    assert_eq!(world.forge.branch(b"forge/temper", b"g"), Some(head));
    assert_eq!(world.forge.parent(head), Some(tip));
    assert_eq!(world.forge.merge_parent(head), None);
    assert_eq!(
        world.disk.content(&[at.as_slice(), b"/.git/temper-head"].concat()),
        Some(head.to_le_bytes().as_slice())
    );
    release_heads(&mut world);
}

pub fn design_clean_move() {
    let (mut world, at, tip) = design_start();
    let moved = world.forge.advance(b"forge/temper", b"g", b"src/x.rs", b"landed code\n");
    world.disk.write(&[at.as_slice(), b"/docs/a.md"].concat(), b"design\n");
    let landed = world.push(false, false);
    let head = world.forge.branch(b"forge/temper", b"g").expect("g exists");
    assert_ne!(head, moved, "delivery merges the moved tip and pushes");
    assert_eq!(&*landed, &[Landing::DesignMerged { commit: translate::commit(head), tip: translate::commit(moved) }]);
    let delivered = world.forge.parent(head).expect("delivery parent");
    assert_eq!(world.forge.parent(delivered), Some(tip));
    assert_eq!(world.forge.merge_parent(head), Some(moved));
    assert_eq!(world.forge.message(head), b"Merge g");
    assert_eq!(world.commits, vec![(translate::commit(delivered), translate::commit(moved), 17, b"Merge g".to_vec())]);
    assert_eq!(
        world.disk.content(&[at.as_slice(), b"/.git/temper-head"].concat()),
        Some(head.to_le_bytes().as_slice())
    );
    assert!(!world.disk.exists(&[at.as_slice(), b"/.git/MERGE_HEAD"].concat()));
    assert_eq!(world.disk.content(&[at.as_slice(), b"/src/x.rs"].concat()), Some(b"landed code\n".as_slice()));
    assert_eq!(world.push(false, false), landed, "repeat keeps the landing");
    world.disk.write(&[at.as_slice(), b"/docs/b.md"].concat(), b"next\n");
    world.push(false, false);
    let next = world.forge.branch(b"forge/temper", b"g").expect("g exists");
    assert_eq!(world.forge.parent(next), Some(head), "next delivery builds on pushed merge");
    assert_eq!(world.pins, 1, "design deliveries never move the prepare pin");
    assert_eq!(
        world.disk.content(&[at.as_slice(), b"/.git/refs/temper/landing"].concat()),
        Some(tip.to_le_bytes().as_slice())
    );
    release_heads(&mut world);
}

pub fn design_conflicting_move() {
    let (mut world, at, _) = design_start();
    let moved = world.forge.advance(b"forge/temper", b"g", b"docs/a.md", b"other design\n");
    world.disk.write(&[at.as_slice(), b"/docs/a.md"].concat(), b"our design\n");
    let refused = world.push(false, false);
    assert_eq!(
        &*refused,
        &[Landing::DesignConflict { files: Box::new([b"docs/a.md".as_slice().into()]) }],
        "design conflicts never end stale"
    );
    assert_eq!(world.forge.branch(b"forge/temper", b"g"), Some(moved));
    assert_eq!(
        world.disk.content(&[at.as_slice(), b"/.git/MERGE_HEAD"].concat()),
        Some(moved.to_le_bytes().as_slice())
    );
    let ours = world.disk.content(&[at.as_slice(), b"/.git/temper-head"].concat()).expect("delivery head");
    let ours = u64::from_le_bytes(ours.try_into().expect("fake id"));
    assert!(
        world.disk.content(&[at.as_slice(), b"/docs/a.md"].concat()).expect("conflicted file").starts_with(b"<<<<<<< ")
    );
    assert_eq!(
        &*world.push(false, false),
        &[Landing::Conflicted { files: Box::new([b"docs/a.md".as_slice().into()]) }]
    );
    world.disk.write(&[at.as_slice(), b"/docs/a.md"].concat(), b"resolved\n");
    world.push(false, false);
    let head = world.forge.branch(b"forge/temper", b"g").expect("g exists");
    assert_eq!(world.forge.parent(head), Some(ours));
    assert_eq!(world.forge.merge_parent(head), Some(moved));
    release_heads(&mut world);
}

pub fn design_markdown_only() {
    for deletion in [false, true] {
        let (mut world, at, tip) = design_start();
        world.disk.write(&[at.as_slice(), b"/docs/a.md"].concat(), b"design\n");
        if deletion {
            world.disk.remove(&[at.as_slice(), b"/code"].concat());
        } else {
            world.disk.write(&[at.as_slice(), b"/src/z.rs"].concat(), b"more code\n");
            world.disk.write(&[at.as_slice(), b"/src/lib.rs"].concat(), b"code\n");
        }
        let landings = world.push(false, false);
        let path = if deletion { b"code".as_slice() } else { b"src/lib.rs".as_slice() };
        assert_eq!(&*landings, &[Landing::DesignCode { path: path.into() }], "first non-Markdown path, in git order");
        assert_eq!(world.forge.branch(b"forge/temper", b"g"), Some(tip), "code additions and deletions are refused");
        assert_eq!(world.push_attempts, 0);
        release_heads(&mut world);
    }
}

pub fn design_moving_again(races: usize) {
    let (mut world, at, _) = design_start();
    world.disk.write(&[at.as_slice(), b"/docs/a.md"].concat(), b"design\n");
    world.push_races = races;
    let landed = world.push(false, false);
    assert_eq!(world.push_attempts, 3, "three leased tries in all");
    assert!(!matches!(&*landed, [Landing::Moved]), "design delivery is never stale");
    let head = world.forge.branch(b"forge/temper", b"g").expect("g exists");
    assert_eq!(world.forge.tree(head).contains_key(b"docs/a.md".as_slice()), races < 3);
    if races == 3 {
        assert_eq!(&*landed, &[Landing::DesignMoving]);
        world.push(false, false);
        let head = world.forge.branch(b"forge/temper", b"g").expect("g exists");
        assert!(
            world.forge.tree(head).contains_key(b"docs/a.md".as_slice()),
            "a refusal leaves the run able to deliver again"
        );
    }
    release_heads(&mut world);
}

pub fn design_no_unchecked_save() {
    let (mut world, at, tip) = design_start();
    world.disk.write(&[at.as_slice(), b"/docs/a.md"].concat(), b"design\n");
    let hold = world.hold.expect("prepared");
    let refused = world.finish(Event::PushUnchecked { hold }, false);
    assert!(
        matches!(refused, Request::Pushed { outcome: Outcome::Pushed { ref landings }, .. } if landings.as_ref() == [Landing::DesignUnchecked])
    );
    assert_eq!(world.forge.branch(b"forge/temper", b"g"), Some(tip), "unchecked save never touches g");
    assert_eq!(world.push_attempts, 0);
    assert_eq!(
        world.disk.content(&[at.as_slice(), b"/.git/temper-head"].concat()),
        Some(tip.to_le_bytes().as_slice()),
        "refused before committing"
    );
    let saved = world.push(true, false);
    assert!(matches!(&*saved, [Landing::Landed { .. }]), "saved-work branch remains writable");
    assert_eq!(world.forge.branch(b"forge/temper", b"g"), Some(tip));
    release_heads(&mut world);
}
