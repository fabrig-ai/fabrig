//! The inspection adapter over real checkout transitions and the fake disk
//! (temper/docs/design/testing/worker.md, 2).

use std::collections::VecDeque;

use skein_lib::{Env, Queue, Time, Token, Wall};
use skein_world::inspect::{Inspection, Moment, Release, Table, Wake, check_tables, check_waits};
use temper_fake_checkout::{Checkout, git::Tree, layout};
use temper_worker_checkout_world::{LIMITS, forge::Forge, translate};
use temper_worker_domain_checkout::{
    Domain, Event, Limits, Message, Repository, Request, Spec, Start,
    git::{Done, Fault, Op},
};

struct Fixture {
    domain: Domain,
    env: Env<Limits>,
    disk: Checkout,
    forge: Forge,
    operations: VecDeque<(Token, Op, Time)>,
}

impl Fixture {
    fn new(preparations: u32) -> Self {
        let limits = Limits { preparations, ..LIMITS };
        let mut forge = Forge::new(17);
        forge.repository(b"forge/app", b"main", Tree::from([(b"file".to_vec(), b"contents".to_vec())]));
        Self {
            domain: Domain::new(&limits),
            env: Env { now: Time::from_nanos(23), wall: Wall::EPOCH, limits },
            disk: Checkout::new(),
            forge,
            operations: VecDeque::new(),
        }
    }

    fn step(&mut self, event: Event) -> Option<Token> {
        let mut out = Queue::uncapped();
        temper_worker_domain_checkout::step(&mut self.domain, &self.env, event, &mut out);
        let mut held = None;
        while let Some(request) = out.pop() {
            match request {
                Request::Held { hold, .. } => held = Some(hold),
                Request::Io { owner, op, deadline } => self.operations.push_back((owner, op, deadline)),
                _ => {}
            }
        }
        held
    }

    fn prepare(&mut self, key: &[u8]) -> Token {
        self.prepare_remote(key, b"forge/app")
    }

    fn prepare_remote(&mut self, key: &[u8], remote: &[u8]) -> Token {
        self.step(Event::Prepare {
            client: Token::new(1),
            spec: Spec {
                rerun: None,
                key: key.into(),
                repositories: Box::new([Repository {
                    admit: false,
                    name: b"app".as_slice().into(),
                    remote: remote.into(),
                    git: None,
                    start: Start::Branch { branch: b"main".as_slice().into() },
                    identity: 0,
                    account: None,
                    save: None,
                    push: Some(b"topic".as_slice().into()),
                    prefixes: Box::new([]),
                    expected: None,
                    delivered: None,
                    landing: None,
                    pin: None,
                }]),
            },
        })
        .expect("the prepare holds a workspace")
    }

    fn settle(&mut self) {
        while let Some((owner, op, _)) = self.operations.pop_front() {
            let done = translate::perform(&mut self.forge, &mut self.disk, op);
            self.step(Event::Done { owner, done });
        }
    }

    fn inspect(&self) -> Inspection {
        let mut found = temper_worker_inspect::inspection();
        temper_worker_inspect::inspect_checkout(&self.domain, &self.env.limits, &mut found);
        temper_worker_inspect::inspect_disk(&self.domain, &self.env.limits, &self.disk, &mut found);
        found
    }

    fn moment(&self) -> Moment {
        Moment { seed: 17, at: self.env.now, iteration: 4 }
    }
}

fn table<'a>(found: &'a Inspection, name: &str) -> &'a Table {
    found.tables().iter().find(|table| table.name == name).expect("the adapter supplies the table")
}

#[test]
fn records_name_operations_permits_and_idle_workspaces() {
    let mut f = Fixture::new(1);
    let idle = f.prepare(b"idle");
    f.settle();
    let idle_workspace = f.domain.held().iter().find(|(id, _)| id.token() == idle).expect("held").1.workspace();
    f.step(Event::Release { hold: idle });
    f.domain.reclaim();
    let first = f.prepare(b"first");
    let second = f.prepare(b"second");
    let deadline = f.operations.front().expect("first operation").2;
    let found = f.inspect();
    let operation = found.waits().iter().find(|wait| wait.kind == "an operation in flight").expect("operation wait");
    assert_eq!(operation.waiter, format!("hold {}", first.raw()));
    assert!(operation.wakes.contains(&Wake::Timer(deadline)));
    assert!(operation.wakes.contains(&Wake::Party("git".into())));
    let queued = found.waits().iter().find(|wait| wait.kind == "a prepare queued for a permit").expect("queued wait");
    assert_eq!(queued.waiter, format!("hold {}", second.raw()));
    assert_eq!(queued.wakes, [Wake::Waiter(format!("hold {}", first.raw()))]);
    let workspaces = table(&found, "workspaces");
    assert_eq!(workspaces.entries.len(), 3);
    let idle_entry = workspaces
        .entries
        .iter()
        .find(|entry| entry.key == format!("workspace {}", idle_workspace.raw()))
        .expect("idle entry");
    assert_eq!(idle_entry.release, Release::Kept);
    assert_eq!(workspaces.entries.iter().filter(|entry| entry.release == Release::Pending).count(), 2);
    let permits = table(&found, "prepare permits");
    assert_eq!(permits.capacity, 1);
    assert_eq!(permits.entries.len(), 1);
    assert_eq!(permits.entries[0].key, format!("hold {}", first.raw()));
    assert_eq!(permits.entries[0].release, Release::Pending);
    assert_eq!(permits.refused.len(), 1);
    assert_eq!(permits.refused[0].key, format!("hold {}", second.raw()));
    assert_eq!(permits.refused[0].since, f.env.now);
    assert_eq!(permits.refused[0].bound, None);
    assert_eq!(check_waits(&[&found], f.moment()), Ok(()));
    assert_eq!(check_tables(&[&found], f.moment()), Ok(()));
}

#[test]
fn a_store_waiter_names_the_hold_making_it() {
    let mut f = Fixture::new(2);
    let maker = f.prepare(b"first");
    let waiter = f.prepare(b"second");
    let found = f.inspect();
    let wait =
        found.waits().iter().find(|wait| wait.kind == "a prepare waiting for another's store").expect("store waiter");
    assert_eq!(wait.waiter, format!("hold {}", waiter.raw()));
    assert_eq!(wait.wakes, [Wake::Waiter(format!("hold {}", maker.raw()))]);
    let store = f.domain.stores().iter().next().expect("shared store").1;
    assert_eq!(store.maker(), Some(maker));
    assert!(!store.is_made());
    assert_eq!(check_waits(&[&found], f.moment()), Ok(()));
    f.settle();
    assert!(f.domain.stores().iter().next().expect("made store").1.is_made());
    assert_eq!(f.inspect().waits(), []);
}

#[test]
fn locks_find_trusted_workspaces_but_pass_after_a_timeout() {
    let mut f = Fixture::new(1);
    let hold = f.prepare(b"first");
    f.settle();
    let workspace = f.domain.held().iter().next().expect("hold").1.workspace();
    let index = [layout::path(workspace, b"app"), b"/.git/index.lock".to_vec()].concat();
    let store = f.domain.stores().iter().next().expect("store").1.token();
    let reference =
        [layout::store_path(store), format!("/refs/temper/{}/fetched.lock", workspace.raw()).into_bytes()].concat();
    for path in [&index, &reference] {
        f.disk.write(path, b"lock");
        let found = f.inspect();
        assert_eq!(
            check_tables(&[&found], f.moment()),
            Err(format!(
                "seed 17: at 23 ns, iteration 4: worker git's locks {}: still held after release rule: the workspace's making again",
                String::from_utf8_lossy(path),
            ))
        );
        f.disk.remove(path);
    }
    f.step(Event::Push { hold, message: Message { title: b"Change".as_slice().into(), body: Box::new([]) } });
    let (owner, _, deadline) = f.operations.pop_front().expect("commit in flight");
    f.step(Event::Abort { hold });
    assert!(
        f.inspect().waits().iter().any(|wait| wait.wakes.contains(&Wake::Timer(deadline))),
        "an interrupted push still waits for its terminal"
    );
    f.disk.write(&index, b"lock");
    f.disk.write(&reference, b"lock");
    let busy = f.inspect();
    assert_eq!(check_tables(&[&busy], f.moment()), Ok(()), "in-flight operations have not reached rest");
    f.step(Event::Done { owner, done: Done::Failed { fault: Fault::TimedOut } });
    let found = f.inspect();
    assert!(table(&found, "git's locks").entries.iter().all(|entry| entry.release == Release::Pending));
    assert_eq!(check_tables(&[&found], f.moment()), Ok(()));
}

#[test]
fn a_store_lock_is_due_only_while_the_workspace_uses_that_store() {
    let mut f = Fixture::new(1);
    f.env.limits.workspaces = 1;
    f.domain = Domain::new(&f.env.limits);
    f.forge.repository(b"forge/other", b"main", Tree::from([(b"other".to_vec(), b"contents".to_vec())]));
    let first = f.prepare(b"first");
    let workspace = f.domain.held().iter().next().expect("hold").1.workspace();
    // A cut fetch leaves its store's ref lock and spoils the workspace.
    while let Some((owner, op, _)) = f.operations.pop_front() {
        let cut = matches!(op, Op::Fetch { .. });
        let done = if cut {
            translate::cut(&mut f.forge, &mut f.disk, op)
        } else {
            translate::perform(&mut f.forge, &mut f.disk, op)
        };
        f.step(Event::Done { owner, done });
        if cut {
            break;
        }
    }
    let old_lock = f.disk.files().keys().find(|path| path.ends_with(b".lock")).expect("cut ref lock").to_vec();
    f.step(Event::Release { hold: first });
    f.domain.reclaim();
    let second = f.prepare_remote(b"second", b"forge/other");
    f.settle();
    assert_eq!(f.domain.held().iter().next().expect("hold").1.workspace(), workspace, "eviction keeps the token");
    assert!(f.disk.exists(&old_lock), "Make names only the new repositories' stores");
    let found = f.inspect();
    assert_eq!(check_tables(&[&found], f.moment()), Ok(()), "the old store is not trusted by this workspace");
    assert_eq!(table(&found, "git's locks").entries[0].release, Release::Pending);

    let store = f
        .domain
        .stores()
        .iter()
        .find(|(_, store)| f.domain.workspace_uses_store(workspace, store.token()))
        .expect("current store")
        .1
        .token();
    let new_lock =
        [layout::store_path(store), format!("/refs/temper/{}/fetched.lock", workspace.raw()).into_bytes()].concat();
    f.disk.write(&new_lock, b"lock");
    let found = f.inspect();
    let finding = check_tables(&[&found], f.moment()).expect_err("a lock in the current store is due");
    assert!(finding.contains(&String::from_utf8_lossy(&new_lock).to_string()), "{finding}");
    f.disk.remove(&new_lock);
    f.step(Event::Release { hold: second });
    f.domain.reclaim();
    f.prepare(b"third");
    f.settle();
    assert!(!f.disk.exists(&old_lock), "returning to the old store makes its refs afresh");
    assert_eq!(check_tables(&[&f.inspect()], f.moment()), Ok(()));
}

#[test]
fn pack_counts_pass_at_seventy_five_and_find_seventy_six() {
    let mut f = Fixture::new(1);
    f.prepare(b"first");
    f.settle();
    let store = f.domain.stores().iter().next().expect("store").1.token();
    let workspace = f.domain.held().iter().next().expect("hold").1.workspace();
    for (place, key) in [
        (layout::store_path(store), format!("store {}", store.raw())),
        ([layout::path(workspace, b"app"), b"/.git".to_vec()].concat(), format!("workspace {} app", workspace.raw())),
    ] {
        for n in 0..75 {
            f.disk.write(&[place.as_slice(), format!("/objects/pack/pack-{n}.pack").as_bytes()].concat(), b"pack");
        }
        let found = f.inspect();
        let packs = table(&found, "Git stores' packs");
        assert_eq!(packs.capacity, 18);
        assert!(packs.entries.iter().any(|entry| entry.key == key && entry.release == Release::Pending));
        assert_eq!(check_tables(&[&found], f.moment()), Ok(()));
        let extra = [place.as_slice(), b"/objects/pack/pack-75.pack"].concat();
        f.disk.write(&extra, b"pack");
        let found = f.inspect();
        let finding = check_tables(&[&found], f.moment()).expect_err("76 packs must trigger packing");
        assert!(finding.contains(&format!("worker Git stores' packs {key}:")), "{finding}");
        assert!(finding.contains("the packing its own count triggers"), "{finding}");
        f.disk.remove(&extra);
    }
}

#[test]
fn scratch_keeps_leftovers_and_restart_directories_until_the_slot_is_taken() {
    let mut f = Fixture::new(1);
    let directory = layout::path(Token::new(0), b"app");
    f.disk.write(&[directory.as_slice(), b"/.git/HEAD"].concat(), b"leftover");
    let leftover = f.inspect();
    assert_eq!(table(&leftover, "per-run scratch").entries[0].release, Release::Kept);
    assert_eq!(check_tables(&[&leftover], f.moment()), Ok(()));
    let hold = f.prepare(b"first");
    f.settle();
    f.step(Event::Release { hold });
    f.domain = Domain::new(&f.env.limits);
    let restarted = f.inspect();
    assert_eq!(table(&restarted, "per-run scratch").entries[0].release, Release::Kept);
    assert_eq!(check_tables(&[&restarted], f.moment()), Ok(()));
    f.prepare(b"second");
    f.settle();
    let taken = f.inspect();
    assert_eq!(table(&taken, "per-run scratch").entries[0].release, Release::Pending);
    assert_eq!(check_tables(&[&taken], f.moment()), Ok(()));
}

#[test]
fn scratch_finds_a_repository_without_a_cached_workspace() {
    let mut f = Fixture::new(1);
    f.prepare(b"first");
    f.settle();
    let found = f.inspect();
    let scratch = table(&found, "per-run scratch");
    assert_eq!(scratch.capacity, 9);
    assert_eq!(scratch.entries.len(), 1);
    assert_eq!(scratch.entries[0].release, Release::Pending);
    for orphan in [Token::new(999), Token::new(u64::from(f.env.limits.workspaces) << 32)] {
        let path = [layout::path(orphan, b"app"), b"/.git/HEAD".to_vec()].concat();
        f.disk.write(&path, b"head");
        let found = f.inspect();
        let finding = check_tables(&[&found], f.moment()).expect_err("taken or out-of-range slots are due");
        assert!(finding.contains(&format!("worker per-run scratch workspace {} app:", orphan.raw())), "{finding}");
        assert!(finding.contains("the workspace's release, or its slot's next taking"), "{finding}");
        f.disk.remove(&path);
    }
}
