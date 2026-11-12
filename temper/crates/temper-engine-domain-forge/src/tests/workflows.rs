//! Workflow read and invalidation rows (temper/docs/design/domain/forge/adoption.md, 7).
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "ordinary workflow tables construct answers and inspect requests outside step code"
)]
use super::*;
use skein_forge::api::{self, Answer, Entry, EntryKind, Error, Op, Read};

const TIP: api::Commit = [4; 32];

fn restored(kind: Kind) -> Domain {
    let mut d = domain();
    d.configured.get_mut(&1).unwrap().kind = kind;
    let mut row = adopted();
    row.workflows = Box::new([b"old.yml".as_slice().into()]);
    outputs(&mut d, Event::Restore { record: Stored::Repository(row) });
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::ReadAfresh);
    outputs(&mut d, Event::SettleOutbox);
    d
}

fn directory(path: &[u8], names: &[&[u8]], truncated: bool) -> Answer {
    Answer::Directory {
        head: TIP,
        path: path.into(),
        entries: names.iter().map(|name| Entry { name: (*name).into(), kind: EntryKind::File }).collect(),
        truncated,
    }
}

fn file_read(path: &[u8], head: api::Commit) -> Read {
    Read::File { head, path: path.into(), max_bytes: LIMITS.client.answer_bytes }
}

fn expect_read(d: &mut Domain, read: Read) -> Token {
    let (call, op) = fixtures::next_call(d);
    assert_eq!(op, Op::Read(read));
    call
}

fn saved(requests: &[Request]) -> bool {
    for request in requests {
        if let Request::Save { record: Stored::Repository(_), .. } = request {
            return true;
        }
    }
    false
}

#[test]
#[expect(clippy::too_many_lines, reason = "one table holds the workflow rule and its answer or trigger rows")]
fn the_workflows_are_read_at_the_landing_tip_and_kept_when_refused() {
    struct Row {
        name: &'static str,
        kind: Kind,
        tip: Result<Answer, Error>,
        listings: Vec<(&'static [u8], Result<Answer, Error>)>,
        expected: Vec<Box<[u8]>>,
        saved: bool,
    }
    let github = b".github/workflows".as_slice();
    let forgejo = b".forgejo/workflows".as_slice();
    let mut entries = vec![
        Entry { name: b"ci.yml".as_slice().into(), kind: EntryKind::File },
        Entry { name: b"lint.yaml".as_slice().into(), kind: EntryKind::File },
        Entry { name: b"README.md".as_slice().into(), kind: EntryKind::File },
        Entry { name: b"dir.yml".as_slice().into(), kind: EntryKind::Directory },
        Entry { name: b"module.yml".as_slice().into(), kind: EntryKind::Submodule },
    ];
    let mut long = vec![b'a'; 252];
    long.extend_from_slice(b".yml");
    entries.push(Entry { name: long.into(), kind: EntryKind::File });
    entries.push(Entry { name: vec![255, b'.', b'y', b'm', b'l'].into(), kind: EntryKind::File });
    let many: Vec<Box<[u8]>> = (0_u32..17).map(|i| format!("{i}.yml").into_bytes().into()).collect();
    let many_refs: Vec<&[u8]> = many.iter().map(AsRef::as_ref).collect();
    let rows = [
        Row {
            name: "only whole workflow files",
            kind: Kind::GitHub,
            tip: Ok(Answer::Commit(TIP)),
            listings: vec![(
                github,
                Ok(Answer::Directory { head: TIP, path: github.into(), entries: entries.into(), truncated: false }),
            )],
            expected: vec![b"ci.yml".as_slice().into(), b"lint.yaml".as_slice().into()],
            saved: true,
        },
        Row {
            name: "Forgejo first",
            kind: Kind::Forgejo,
            tip: Ok(Answer::Commit(TIP)),
            listings: vec![(forgejo, Ok(directory(forgejo, &[b"checks.yml"], false)))],
            expected: vec![b"checks.yml".as_slice().into()],
            saved: true,
        },
        Row {
            name: "Forgejo fallback",
            kind: Kind::Forgejo,
            tip: Ok(Answer::Commit(TIP)),
            listings: vec![(forgejo, Err(Error::Missing)), (github, Ok(directory(github, &[b"ci.yml"], false)))],
            expected: vec![b"ci.yml".as_slice().into()],
            saved: true,
        },
        Row {
            name: "both absent",
            kind: Kind::Forgejo,
            tip: Ok(Answer::Commit(TIP)),
            listings: vec![(forgejo, Err(Error::Missing)), (github, Err(Error::Missing))],
            expected: vec![],
            saved: true,
        },
        Row {
            name: "a file is no directory",
            kind: Kind::GitHub,
            tip: Ok(Answer::Commit(TIP)),
            listings: vec![(
                github,
                Ok(Answer::File { head: TIP, path: github.into(), bytes: Box::new([]), truncated: false }),
            )],
            expected: vec![],
            saved: true,
        },
        Row {
            name: "listing refused",
            kind: Kind::GitHub,
            tip: Ok(Answer::Commit(TIP)),
            listings: vec![(github, Err(Error::Unavailable))],
            expected: vec![b"old.yml".as_slice().into()],
            saved: false,
        },
        Row {
            name: "tip refused",
            kind: Kind::GitHub,
            tip: Err(Error::Forbidden),
            listings: vec![],
            expected: vec![b"old.yml".as_slice().into()],
            saved: false,
        },
        Row {
            name: "first sixteen",
            kind: Kind::GitHub,
            tip: Ok(Answer::Commit(TIP)),
            listings: vec![(github, Ok(directory(github, &many_refs, false)))],
            expected: many[..16].into(),
            saved: true,
        },
        Row {
            name: "truncated keeps names",
            kind: Kind::GitHub,
            tip: Ok(Answer::Commit(TIP)),
            listings: vec![(github, Ok(directory(github, &[b"a.yml"], true)))],
            expected: vec![b"a.yml".as_slice().into()],
            saved: true,
        },
        Row {
            name: "unchanged is not saved",
            kind: Kind::GitHub,
            tip: Ok(Answer::Commit(TIP)),
            listings: vec![(github, Ok(directory(github, &[b"old.yml"], false)))],
            expected: vec![b"old.yml".as_slice().into()],
            saved: false,
        },
        Row {
            name: "Forgejo file falls back",
            kind: Kind::Forgejo,
            tip: Ok(Answer::Commit(TIP)),
            listings: vec![
                (forgejo, Ok(Answer::File { head: TIP, path: forgejo.into(), bytes: Box::new([]), truncated: false })),
                (github, Ok(directory(github, &[b"ci.yml"], false))),
            ],
            expected: vec![b"ci.yml".as_slice().into()],
            saved: true,
        },
        Row {
            name: "wrong answer keeps names",
            kind: Kind::GitHub,
            tip: Ok(Answer::Commit(TIP)),
            listings: vec![(github, Ok(Answer::Done))],
            expected: vec![b"old.yml".as_slice().into()],
            saved: false,
        },
    ];
    for row in rows {
        let mut d = restored(row.kind);
        crate::workflows::read(&mut d, &env(), REPO, None, &mut Queue::uncapped());
        let call = expect_read(&mut d, Read::Branch { branch: b"main".as_slice().into() });
        let mut did_save = saved(&fixtures::answer(&mut d, call, row.tip));
        for (path, answer) in row.listings {
            let call = expect_read(&mut d, file_read(path, TIP));
            if answer == Err(Error::Unavailable) {
                // Deliver the terminal failure directly: the client retries Unavailable.
                let owner = Token::new(d.sequence | (1_u64 << 60_u32));
                let mut out = Queue::uncapped();
                crate::workflows::answered(&mut d, &env(), owner, answer, &mut out);
                assert!(out.is_empty(), "a failed listing saves nothing");
            } else {
                did_save |= saved(&fixtures::answer(&mut d, call, answer));
            }
        }
        assert_eq!(d.repository(REPO).unwrap().workflows.as_ref(), row.expected, "{}", row.name);
        assert_eq!(did_save, row.saved, "{}", row.name);
        assert!(ready(&mut d).is_empty(), "no extra listing: {}", row.name);
    }
}

fn observe(d: &mut Domain, branch: &[u8], previous: Option<api::Commit>) {
    if let Some(commit) = previous {
        let name =
            Name { forge: REPO.forge, repository: REPO.repository, what: What::Branch(Box::new([branch.into()])) };
        d.heads.insert(name.clone(), BranchHead { name, commit, owned: false }).unwrap();
    }
    outputs(
        d,
        Event::Subscribe {
            subscription: Subscriber {
                task: 7,
                number: 1,
                topic: Topic::Landings { repository: REPO, branch: branch.into() },
                own_change: None,
                goal_tasks: None,
                paths: Box::new([]),
            },
        },
    );
    let call = expect_read(d, Read::Branch { branch: branch.into() });
    fixtures::answer(d, call, Ok(Answer::Commit(TIP)));
}

#[test]
#[expect(clippy::too_many_lines, reason = "one table holds the workflow rule and its answer or trigger rows")]
fn the_landing_branch_reads_its_workflows_again_only_when_they_may_have_changed() {
    enum Trigger {
        First,
        Land { branch: &'static [u8], files: Option<&'static [u8]>, changed: u64 },
        Failed,
        InFlight,
        Adoption,
        Context,
        Refresh,
        Released,
        NoRoom,
    }
    for (name, trigger, expected) in [
        ("first tip", Trigger::First, Some(file_read(b".github/workflows", TIP))),
        ("ordinary landing", Trigger::Land { branch: b"main", files: Some(b"src/lib.rs"), changed: 1 }, None),
        (
            "GitHub workflow",
            Trigger::Land { branch: b"main", files: Some(b".github/workflows/ci.yml"), changed: 1 },
            Some(file_read(b".github/workflows", TIP)),
        ),
        (
            "Forgejo workflow",
            Trigger::Land { branch: b"main", files: Some(b".forgejo/workflows/ci.yml"), changed: 1 },
            Some(file_read(b".github/workflows", TIP)),
        ),
        (
            "comparison refused",
            Trigger::Land { branch: b"main", files: None, changed: 1 },
            Some(file_read(b".github/workflows", TIP)),
        ),
        (
            "partial comparison",
            Trigger::Land { branch: b"main", files: Some(b"src/lib.rs"), changed: 2 },
            Some(file_read(b".github/workflows", TIP)),
        ),
        (
            "other branch",
            Trigger::Land { branch: b"release", files: Some(b".github/workflows/ci.yml"), changed: 1 },
            None,
        ),
        ("failed read marks next move", Trigger::Failed, Some(file_read(b".github/workflows", TIP))),
        ("one read in flight", Trigger::InFlight, Some(Read::Branch { branch: b"main".as_slice().into() })),
        ("adoption", Trigger::Adoption, Some(Read::Branch { branch: b"main".as_slice().into() })),
        ("context beside writer", Trigger::Context, None),
        ("capability refresh", Trigger::Refresh, Some(Read::Branch { branch: b"main".as_slice().into() })),
        ("released row takes no answer", Trigger::Released, None),
        ("no room marks next move", Trigger::NoRoom, Some(file_read(b".github/workflows", TIP))),
    ] {
        let mut d = restored(Kind::GitHub);
        let mut out = Queue::uncapped();
        match trigger {
            Trigger::First => observe(&mut d, b"main", None),
            Trigger::Land { branch, files, changed } => {
                observe(&mut d, branch, Some([3; 32]));
                let call = expect_read(&mut d, Read::Compare { before: [3; 32], after: TIP });
                let result = match files {
                    Some(path) => Ok(Answer::Compare {
                        before: [3; 32],
                        after: TIP,
                        files: Box::new([path.into()]),
                        ahead: 0,
                        behind: 0,
                        changed,
                    }),
                    None => Err(Error::Forbidden),
                };
                fixtures::answer(&mut d, call, result);
            }
            Trigger::Failed | Trigger::InFlight | Trigger::Released => {
                crate::workflows::read(&mut d, &env(), REPO, Some([3; 32]), &mut out);
                let call = expect_read(&mut d, file_read(b".github/workflows", [3; 32]));
                match trigger {
                    Trigger::Failed => {
                        fixtures::answer(&mut d, call, Err(Error::Forbidden));
                        crate::workflows::landed(
                            &mut d,
                            &env(),
                            REPO,
                            b"main",
                            TIP,
                            Some(&[b"src/lib.rs".as_slice().into()]),
                            &mut out,
                        );
                    }
                    Trigger::InFlight => {
                        crate::workflows::landed(&mut d, &env(), REPO, b"main", TIP, None, &mut out);
                        crate::workflows::landed(&mut d, &env(), REPO, b"main", TIP, None, &mut out);
                        assert!(ready(&mut d).is_empty(), "one read at a time");
                        fixtures::answer(&mut d, call, Err(Error::Forbidden));
                    }
                    Trigger::Released => {
                        outputs(&mut d, Event::ForgetAdoption { repository: REPO, restore: None });
                        assert!(!saved(&fixtures::answer(&mut d, call, Err(Error::Missing))));
                        assert!(d.repository(REPO).is_none());
                    }
                    Trigger::First
                    | Trigger::Land { .. }
                    | Trigger::Adoption
                    | Trigger::Context
                    | Trigger::Refresh
                    | Trigger::NoRoom => unreachable!(),
                }
            }
            Trigger::Adoption | Trigger::Context => {
                let mut context = false;
                if let Trigger::Context = trigger {
                    context = true;
                }
                let mut adoption = fixtures::adoption();
                if context {
                    adoption.project += 1;
                    adoption.role = Role::Context;
                    adoption.home = false;
                }
                fixtures::begin_adoption(&mut d, Token::new(4), adoption);
                let answers = [
                    Answer::Permission(api::UserPermission {
                        user: 1,
                        login: Box::new([]),
                        permission: if context { api::Permission::Read } else { api::Permission::Admin },
                    }),
                    Answer::Settings(adopted().settings),
                    Answer::Protection(None),
                    Answer::Collaborators { collaborators: Box::new([]), more: false },
                ];
                let mut requests: Box<[Request]> = Box::new([]);
                for answer in answers {
                    requests = fixtures::answer_next(&mut d, Ok(answer));
                }
                let Some(Request::Adopted { result: Ok(_), .. }) = requests.last() else {
                    panic!("adopted is last");
                };
                assert!(saved(&requests), "writer facts or the additional context project are saved");
                assert_eq!(d.repository(REPO).unwrap().workflows.as_ref(), &[Box::from(b"old.yml".as_slice())]);
            }
            Trigger::Refresh => {
                outputs(&mut d, Event::Enqueue { entry: status(19, 8) });
                outputs(&mut d, Event::Committed { entry: 19 });
                let (poll, op) = fixtures::next_call(&mut d);
                let Op::Read(Read::Items { .. }) = op else { panic!("repository poll: {op:?}") };
                fixtures::answer(
                    &mut d,
                    poll,
                    Ok(Answer::Items { cut: false, items: Box::new([]), more: false, now: env().now }),
                );
                let (call, op) = fixtures::next_call(&mut d);
                let Op::Write(_) = op else { panic!("status write: {op:?}") };
                fixtures::answer(&mut d, call, Err(Error::Forbidden));
                for (read, answer) in [
                    (
                        Read::Permission { user: 7 },
                        Answer::Permission(api::UserPermission {
                            user: 7,
                            login: Box::new([]),
                            permission: api::Permission::Write,
                        }),
                    ),
                    (Read::Settings, Answer::Settings(adopted().settings)),
                    (Read::Protection { branch: b"main".as_slice().into() }, Answer::Protection(None)),
                ] {
                    let call = expect_read(&mut d, read);
                    fixtures::answer(&mut d, call, Ok(answer));
                }
            }
            Trigger::NoRoom => {
                let mut full = env();
                full.limits.repositories = 0;
                crate::workflows::read(&mut d, &full, REPO, None, &mut out);
                assert!(ready(&mut d).is_empty());
                crate::workflows::landed(
                    &mut d,
                    &env(),
                    REPO,
                    b"main",
                    TIP,
                    Some(&[b"src/lib.rs".as_slice().into()]),
                    &mut out,
                );
            }
        }
        match expected {
            Some(read) => {
                expect_read(&mut d, read);
            }
            None => assert!(ready(&mut d).is_empty(), "{name}"),
        }
    }
}
