//! The home reader over the fake forge, with the connector's own client and store requests.
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "ordinary Rust fake forge harness and assertions"
)]

use super::*;
use skein_fake_forge_domain::{self as fake, api as raw};
use skein_forge::api::{self, Answer, Commit, EntryKind, Error, Op, Read};
use skein_lib::ReplyTo;

pub(crate) const PROJECT: u32 = 5;
const REPOSITORY: &[u8] = b"org/repo";
const BRANCH: &[u8] = b"staging";
pub(crate) const WAYS: &[u8] = b".temper/ways/build.toml";
pub(crate) const GOOD: &[u8] = b"format = 1\n";

pub(crate) struct World {
    pub(crate) d: Domain,
    env: Env<Limits>,
    forge: fake::Domain,
    forge_env: Env<fake::Config>,
    calls: Vec<(Token, api::Repository, Read)>,
    seen: Vec<Read>,
    pub(crate) saved: Vec<TemperRevision>,
    erased: Vec<Key>,
}

impl World {
    pub(crate) fn new(seed: u64, files: Vec<raw::File>) -> Self {
        let mut config = fake::Config::test();
        config.latency_min = Duration::ZERO;
        config.latency_max = Duration::ZERO;
        config.limits.branches = 8;
        config.limits.files = 128;
        config.limits.commits = 128;
        config.limits.content_bytes = 65_536;
        config.limits.name_bytes = 4_096;
        let forge_env = env().with(config);
        let mut forge = fake::Domain::new(&forge_env.limits, seed);
        fake::repository(
            &mut forge,
            &forge_env.limits,
            raw::Setup {
                id: REPO.repository,
                name: REPOSITORY.into(),
                default: BRANCH.into(),
                tree: files.into(),
                labels: Box::new([]),
                checks: forge_env.limits.made_checks.clone(),
                workflows: Box::new([]),
                protection: None,
                hooked: false,
            },
        );
        fake::grant(&mut forge, REPOSITORY, 7, raw::Permission::Admin);
        let mut env = env();
        env.limits.client.answer_bytes = 131_072;
        let mut d = active_domain_with_limits(&env.limits);
        d.restore_begin(env.wall);
        Self { d, env, forge, forge_env, calls: Vec::new(), seen: Vec::new(), saved: Vec::new(), erased: Vec::new() }
    }

    fn take(&mut self, mut out: Queue<Request>) {
        while let Some(request) = out.pop() {
            if let Request::Call { call, repository, op: Op::Read(read) } = request {
                self.seen.push(read.clone());
                self.calls.push((call, repository, read));
            } else if let Request::Save { record: Stored::TemperRevision(row), .. } = request {
                self.saved.push(row);
            } else if let Request::Erase { key } = request {
                self.erased.push(key);
            }
        }
    }

    fn event(&mut self, event: Event) {
        let mut out = Queue::uncapped();
        step(&mut self.d, &self.env, event, &mut out);
        self.take(out);
        self.d.reclaim();
    }

    pub(crate) fn restart(&mut self, previous: Option<TemperRevision>) {
        self.calls.clear();
        self.d = domain_with_limits(&self.env.limits);
        self.d.restore_begin(self.env.wall);
        self.restore_home();
        if let Some(row) = previous {
            self.event(Event::Restore { record: Stored::TemperRevision(row) });
        }
        self.event(Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
        self.event(Event::CatchUp);
    }

    pub(crate) fn restore_home(&mut self) {
        self.event(Event::Restore { record: Stored::Repository(home()) });
    }

    fn tick(&mut self, by: Duration) {
        self.env.now = self.env.now.saturating_add(by);
        self.forge_env.now = self.env.now;
        let mut out = Queue::uncapped();
        fire(&mut self.d, &self.env, &mut out);
        self.take(out);
    }

    pub(crate) fn next(&mut self) -> Option<(Token, api::Repository, Read)> {
        for _ in 0..100_u32 {
            if !self.calls.is_empty() {
                return Some(self.calls.remove(0));
            }
            if !self.d.is_ready() {
                return None;
            }
            let mut out = Queue::uncapped();
            resume(&mut self.d, &self.env, &mut out);
            self.take(out);
        }
        panic!("the connector did not yield");
    }

    pub(crate) fn answer(&mut self, call: Token, result: Result<Answer, Error>) {
        self.event(Event::Client(skein_forge_client::Event::Answered { call, cost: 1, free: 0, rate: None, result }));
    }

    fn settle(&mut self) {
        for _ in 0..240_u32 {
            let Some((call, repository, read)) = self.next() else { return };
            let answer = self.answer_for(repository, &read);
            self.answer(call, answer);
        }
        panic!("the connector did not settle");
    }

    pub(crate) fn answer_for(&mut self, _repository: api::Repository, read: &Read) -> Result<Answer, Error> {
        let raw_read = if let Read::Branch { branch } = read {
            raw::Read::Branch { branch: branch.clone() }
        } else if let Read::File { head, path, .. } = read {
            raw::Read::File { commit: fake::commit_count(head).unwrap(), path: path.clone() }
        } else if let Read::Compare { before, after } = read {
            raw::Read::Compare {
                base: fake::commit_count(before).unwrap(),
                head: fake::commit_count(after).unwrap(),
                page: 1,
                limit: 100,
            }
        } else if let Read::Items { .. } = read {
            // The world's inbox is empty; only the trees and their changes vary.
            return Ok(Answer::Items { cut: false, items: Box::new([]), more: false, now: self.env.now });
        } else {
            panic!("unexpected read: {read:?}");
        };
        let answer = match self.call(raw::Op::Read(raw_read)) {
            Ok(answer) => answer,
            Err(raw::Error::Missing(_)) => return Err(Error::Missing),
            Err(raw::Error::Unavailable | raw::Error::Unreachable) => return Err(Error::Unavailable),
            Err(raw::Error::Forbidden) => return Err(Error::Forbidden),
            Err(raw::Error::Refused) => return Err(Error::Refused),
            Err(raw::Error::Timeout) => return Err(Error::Timeout),
            Err(
                error @ (raw::Error::RateLimited { .. }
                | raw::Error::TooLarge
                | raw::Error::Full
                | raw::Error::Exists
                | raw::Error::AlreadyQueued(_)
                | raw::Error::Circular
                | raw::Error::NothingToMerge
                | raw::Error::Empty
                | raw::Error::Closed
                | raw::Error::Stale
                | raw::Error::Conflict
                | raw::Error::Protected),
            ) => panic!("{read:?}: {error:?}"),
        };
        if let raw::Answer::Commit(tip) = answer {
            Ok(Answer::Commit(fake::commit_bytes(tip)))
        } else if let raw::Answer::Comparison { base, head, ahead, behind, files, .. } = answer {
            let changed = u64::try_from(files.len()).unwrap();
            Ok(Answer::Compare {
                before: fake::commit_bytes(base),
                after: fake::commit_bytes(head),
                ahead,
                behind,
                changed,
                files: files.into_vec().into_iter().map(|f| f.path).collect(),
            })
        } else if let Read::File { head, path, max_bytes } = read {
            if let raw::Answer::File(mut bytes) = answer {
                let max = usize::try_from(*max_bytes).unwrap();
                let truncated = bytes.len() > max;
                if truncated {
                    bytes = bytes[..max].into();
                }
                Ok(Answer::File { head: *head, path: path.clone(), bytes, truncated })
            } else if let raw::Answer::Directory(entries) = answer {
                let entries = entries
                    .into_vec()
                    .into_iter()
                    .map(|entry| api::Entry {
                        name: entry.name,
                        kind: match entry.kind {
                            raw::EntryKind::File => EntryKind::File,
                            raw::EntryKind::Directory => EntryKind::Directory,
                            raw::EntryKind::Link => EntryKind::Link,
                        },
                    })
                    .collect();
                Ok(Answer::Directory { head: *head, path: path.clone(), entries, truncated: false })
            } else {
                panic!("{read:?}: {answer:?}")
            }
        } else {
            panic!("{read:?}: {answer:?}")
        }
    }

    pub(crate) fn call(&mut self, op: raw::Op) -> Result<raw::Answer, raw::Error> {
        let mut out = Queue::uncapped();
        fake::step(
            &mut self.forge,
            &self.forge_env,
            fake::Event::Call { reply_to: ReplyTo::new(Token::new(7)), user: 7, repository: REPOSITORY.into(), op },
            &mut out,
        );
        fake::fire(&mut self.forge, &self.forge_env, &mut out);
        self.forge.reclaim();
        let Some(fake::Request::Reply { result, .. }) = out.pop() else { panic!("fake forge did not answer") };
        result
    }

    pub(crate) fn tip(&self) -> Commit {
        self.branch_tip(BRANCH)
    }

    pub(crate) fn branch_tip(&self, branch: &[u8]) -> Commit {
        fake::commit_bytes(self.forge.branch(REPOSITORY, branch).unwrap())
    }

    pub(crate) fn push(&mut self, branch: &[u8], path: &[u8], words: &[u8]) -> Commit {
        fake::commit_bytes(fake::advance(&mut self.forge, &self.forge_env, REPOSITORY, branch, path, words, 7).unwrap())
    }

    pub(crate) fn advance(&mut self, path: &[u8], words: &[u8]) -> Commit {
        self.push(BRANCH, path, words)
    }

    fn observe(&mut self, repository: api::Repository, branch: &[u8], tip: Commit) {
        let mut out = Queue::uncapped();
        let name = crate::tiers::name(repository, branch);
        let resource =
            skein_forge_client::Resource { repository, what: skein_forge_client::What::Branch(branch.into()) };
        domain::record_branch_tip(&mut self.d, &self.env, name, &resource, tip, &mut out);
        self.take(out);
    }

    pub(crate) fn trigger(&mut self, tip: Option<Commit>) {
        let mut out = Queue::uncapped();
        crate::temper_revision::landed(&mut self.d, &self.env, REPO, BRANCH, tip, &mut out);
        self.take(out);
    }

    fn revision(&self) -> &TemperRevision {
        self.d.temper_revision(PROJECT).unwrap()
    }

    fn root_reads(&self) -> usize {
        self.seen.iter().filter(|read| matches!(read, Read::File { path, .. } if path.as_ref() == b".temper")).count()
    }
}

pub(crate) fn file(path: &[u8], bytes: &[u8]) -> raw::File {
    raw::File { path: path.into(), content: bytes.into() }
}

fn home() -> Repository {
    Repository { landing: BRANCH.into(), ..adopted() }
}

#[test]
fn only_a_taken_home_adoption_reads_the_revision() {
    let request = Token::new(4);
    for (role, is_home, taken, beside_home) in [
        (Role::Adopted, true, true, false),
        (Role::Adopted, true, false, false),
        (Role::Context, false, true, false),
        (Role::Context, false, true, true),
        (Role::Adopted, false, true, false),
    ] {
        let mut w = World::new(1, vec![file(WAYS, GOOD)]);
        if beside_home {
            w.restore_home();
        }
        let project = if beside_home { PROJECT + 1 } else { PROJECT };
        let adoption = Adoption { project, role, home: is_home, landing: BRANCH.into(), ..fixtures::adoption() };
        fixtures::begin_adoption(&mut w.d, request, adoption);
        fixtures::answer_next(
            &mut w.d,
            Ok(Answer::Permission(api::UserPermission {
                user: 7,
                login: b"owner".as_slice().into(),
                permission: api::Permission::Admin,
            })),
        );
        if role != Role::Context {
            fixtures::answer_next(&mut w.d, Ok(Answer::Branches(Box::new([]))));
        }
        for answer in [Answer::Settings(adopted().settings), Answer::Protection(None)] {
            fixtures::answer_next(&mut w.d, Ok(answer));
        }
        let requests =
            fixtures::answer_next(&mut w.d, Ok(Answer::Collaborators { collaborators: Box::new([]), more: false }));
        assert!(requests.iter().any(|r| matches!(r, Request::Save { record: Stored::Repository(row), .. } if row.home == (is_home || beside_home))));
        assert!(requests.iter().any(|r| matches!(r, Request::Adopted { result: Ok(_), .. })));
        w.settle();
        assert_eq!(w.root_reads(), 0, "before root takes or forgets the adoption");
        w.seen.clear();
        if taken {
            w.event(Event::TakeAdoption { repository: REPO, request });
        } else {
            w.event(Event::ForgetAdoption { repository: REPO, restore: None });
        }
        w.settle();
        if is_home && taken {
            assert!(
                matches!(w.seen.iter().find(|read| matches!(read, Read::Branch { .. } | Read::File { .. })), Some(Read::Branch { branch }) if branch.as_ref() == BRANCH)
            );
            assert_eq!(w.root_reads(), 1);
            assert_eq!(
                (w.revision().number, w.revision().home, w.revision().commit, w.revision().method),
                (1, REPO, Some(w.tip()), true)
            );
            assert_eq!(w.revision().files.as_ref(), &[TemperFile { path: WAYS.into(), bytes: GOOD.into() }]);
            assert_eq!(w.saved.len(), 1);
        } else {
            assert_eq!(w.root_reads(), 0);
            assert_eq!(w.saved, []);
        }
    }
}

#[test]
fn only_relevant_home_landing_comparisons_read_the_revision() {
    enum Move {
        Temper,
        Unknown,
        Rewind,
        Unrelated,
        OtherBranch,
        OtherRepo,
    }
    for (movement, reads) in [
        (Move::Temper, 1),
        (Move::Unknown, 1),
        (Move::Rewind, 1),
        (Move::Unrelated, 0),
        (Move::OtherBranch, 0),
        (Move::OtherRepo, 0),
    ] {
        let mut w = World::new(9, vec![file(WAYS, GOOD)]);
        w.restore_home();
        let before = w.tip();
        let (repository, branch, after) = match movement {
            Move::Temper => (REPO, BRANCH, w.advance(WAYS, b"format = 1\n# changed\n")),
            Move::Rewind => {
                let ahead = w.advance(WAYS, b"format = 1\n# removed by rewind\n");
                fake::reset_branch(
                    &mut w.forge,
                    &w.forge_env,
                    REPOSITORY,
                    BRANCH,
                    fake::commit_count(&before).unwrap(),
                    7,
                )
                .unwrap();
                w.d.heads
                    .insert(
                        crate::tiers::name(REPO, BRANCH),
                        BranchHead { name: crate::tiers::name(REPO, BRANCH), commit: ahead, owned: false },
                    )
                    .unwrap();
                (REPO, BRANCH, before)
            }
            Move::Unknown | Move::Unrelated => (REPO, BRANCH, w.advance(b"README", b"unrelated")),
            Move::OtherBranch => (REPO, b"other".as_slice(), w.advance(WAYS, b"format = 1\n# other\n")),
            Move::OtherRepo => {
                let other = api::Repository { repository: REPO.repository + 1, ..REPO };
                w.event(Event::Restore {
                    record: Stored::Repository(Repository { provider: other, home: false, ..home() }),
                });
                (other, BRANCH, w.advance(WAYS, b"format = 1\n# other\n"))
            }
        };
        let name = crate::tiers::name(repository, branch);
        if !w.d.heads.contains_key(&name) {
            w.d.heads.insert(name.clone(), BranchHead { name, commit: before, owned: false }).unwrap();
        }
        w.observe(repository, branch, after);
        if let Move::Unknown = movement {
            let (call, _, read) = w.next().unwrap();
            assert!(matches!(read, Read::Compare { .. }));
            w.answer(call, Err(Error::Forbidden));
        }
        w.settle();
        assert_eq!(w.root_reads(), reads, "{branch:?} {repository:?} {:?}", w.seen);
    }
}

#[test]
fn revision_keeps_last_good_and_retries_only_unread_trees() {
    let mut w = World::new(5, vec![file(WAYS, GOOD)]);
    w.restore_home();
    w.trigger(None);
    let (call, _, read) = w.next().unwrap();
    assert!(matches!(read, Read::Branch { .. }));
    w.answer(call, Err(Error::Forbidden));
    assert_eq!((w.revision().number, w.revision().commit, w.revision().observed), (0, None, 1));
    assert_eq!(w.revision().refused.as_ref().unwrap().words.as_ref(), b"forge refused permission");
    assert_eq!(w.revision().refused.as_ref().unwrap().commit, None);
    w.settle();
    assert_eq!(w.root_reads(), 0);
    w.tick(w.env.limits.client.backoff);
    w.settle();
    let good = w.revision().clone();
    assert_eq!((good.number, good.commit, good.observed, good.refused), (1, Some(w.tip()), 2, None));
    let equal = w.advance(b"README", b"not in the method");
    w.trigger(Some(equal));
    w.settle();
    assert_eq!((w.revision().number, w.revision().commit, w.revision().observed), (1, good.commit, 3));
    w.advance(WAYS, b"format = 2\n");
    let bad = w.advance(b".temper/ways/bad.txt", b"fault beside ways.toml");
    w.trigger(Some(bad));
    w.settle();
    let refused = w.revision().refused.as_ref().unwrap();
    assert_eq!((refused.commit, refused.observed), (Some(bad), 4));
    assert!(refused.words.starts_with(b".temper/ways/"), "{:?}", refused.words);
    assert!(refused.words.ends_with(b"(and 1 more)"));
    assert_eq!(w.revision().files, good.files);
    let reads = w.root_reads();
    w.tick(w.env.limits.client.backoff);
    w.settle();
    assert_eq!(w.root_reads(), reads, "a judgement fault does not arm a retry");
    fake::reset_branch(&mut w.forge, &w.forge_env, REPOSITORY, BRANCH, fake::commit_count(&equal).unwrap(), 7).unwrap();
    let changed = w.advance(WAYS, b"format = 1\n# changed from the last good read\n");
    w.trigger(Some(changed));
    w.settle();
    assert_eq!((w.revision().number, w.revision().commit, w.revision().refused.as_ref()), (2, Some(changed), None));

    let mut hooks_only = World::new(6, vec![file(b".temper/hooks/config.toml", b"not toml")]);
    hooks_only.restore_home();
    hooks_only.trigger(None);
    hooks_only.settle();
    assert_eq!(
        (hooks_only.revision().number, hooks_only.revision().method, hooks_only.revision().refused.as_ref()),
        (1, false, None)
    );
}

#[test]
fn restart_coalesces_triggers_and_drops_stale_or_retired_home_answers() {
    let mut w = World::new(21, vec![file(WAYS, GOOD)]);
    let first = w.tip();
    let row = TemperRevision {
        project: PROJECT,
        number: 7,
        home: REPO,
        commit: Some(first),
        method: true,
        files: Box::new([TemperFile { path: WAYS.into(), bytes: GOOD.into() }]),
        observed: 83,
        refused: None,
    };
    w.restart(Some(row.clone()));
    w.settle();
    assert_eq!(w.root_reads(), 1);
    assert_eq!(w.revision(), &TemperRevision { observed: 84, ..row });
    w.trigger(Some(first));
    let (call, repository, read) = w.next().unwrap();
    assert!(matches!(&read, Read::File { head, path, .. } if *head == first && path.as_ref() == b".temper"));
    let result = w.answer_for(repository, &read);
    let latest = w.advance(WAYS, b"format = 1\n# new tip\n");
    w.trigger(Some(latest));
    w.trigger(Some(latest));
    assert!(w.next().is_none());
    w.answer(call, result);
    w.settle();
    assert_eq!(w.root_reads(), 3);
    assert_eq!((w.revision().commit, w.revision().number, w.revision().observed), (Some(latest), 8, 86));

    w.trigger(Some(latest));
    let (call, repository, read) = w.next().unwrap();
    let result = w.answer_for(repository, &read);
    let protected = TemperRevision { observed: 87, ..w.revision().clone() };
    w.event(Event::Restore { record: Stored::TemperRevision(protected.clone()) });
    let count = w.saved.len();
    w.answer(call, result);
    w.settle();
    assert_eq!(w.saved.len(), count);
    assert_eq!(w.revision(), &protected);

    w.trigger(Some(latest));
    let (call, repository, read) = w.next().unwrap();
    let result = w.answer_for(repository, &read);
    // The previous home is still adopted when an answer from it arrives.
    let other = api::Repository { repository: REPO.repository + 1, ..REPO };
    w.event(Event::Restore { record: Stored::Repository(Repository { home: false, ..home() }) });
    w.event(Event::Restore { record: Stored::Repository(Repository { provider: other, ..home() }) });
    w.answer(call, result);
    assert_eq!(w.revision(), &protected);

    w.event(Event::ForgetAdoption { repository: other, restore: None });
    w.trigger(None);
    w.settle();
    assert_eq!(w.saved.len(), count);
    assert_eq!(w.revision(), &protected);

    // Ending the forge cancels retries, not the project's revision.
    w.restore_home();
    w.event(Event::Restore { record: Stored::Forge(removal::record(REPO.forge, Mode::Fake)) });
    w.trigger(None);
    let (call, _, _) = w.next().unwrap();
    w.answer(call, Err(Error::Forbidden));
    let refused = w.revision().clone();
    let reads = w.root_reads();
    let mut out = Queue::uncapped();
    assert!(crate::removal::begin(&mut w.d, REPO.forge, &mut out));
    w.take(out);
    w.tick(w.env.limits.client.backoff);
    w.settle();
    assert_eq!(w.root_reads(), reads);
    assert_eq!(w.revision(), &refused);
    w.event(Event::ReleaseProject { project: PROJECT, request: Token::new(9) });
    assert!(w.erased.contains(&Key::TemperRevision { project: PROJECT }));
    assert!(w.d.temper_revision(PROJECT).is_none());
}

#[test]
fn refusal_and_tree_size_bounds_leave_the_last_good_revision() {
    let mut files = vec![file(WAYS, GOOD)];
    for n in 0..64_u32 {
        files.push(file(format!(".temper/ways/a{n:02}.toml").as_bytes(), b"format = 1\n"));
    }
    let mut w = World::new(6, files);
    w.restore_home();
    w.trigger(None);
    w.settle();
    let row = w.revision();
    assert_eq!(row.number, 0);
    assert!(row.files.is_empty());
    let words = &row.refused.as_ref().unwrap().words;
    assert!(words.starts_with(b".temper/: past what temper reads: 64 files"), "{:?}", String::from_utf8_lossy(words));

    let mut body = GOOD.to_vec();
    body.push(b'#');
    body.resize(65_536, b'x');
    let files = (0..9_u32).map(|n| file(format!(".temper/ways/b{n}.toml").as_bytes(), &body)).collect();
    let mut w = World::new(7, files);
    w.restore_home();
    w.trigger(None);
    w.settle();
    assert_eq!(w.revision().number, 0);
    assert!(w.revision().refused.as_ref().unwrap().words.starts_with(b".temper/: past what temper reads: 64 files"));

    let long = vec![b'x'; 1500];
    let path = [b".temper/ways/".as_slice(), long.as_slice()].concat();
    let mut w = World::new(6, vec![file(&path, b"invalid file name")]);
    w.restore_home();
    w.trigger(None);
    w.settle();
    assert_eq!(w.revision().refused.as_ref().unwrap().words.len(), 1024);
}
