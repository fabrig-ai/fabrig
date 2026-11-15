//! The open's drop job precedes the strict restart link check (protocol/store.md, 4).
use crate::tests::fixtures::{
    self,
    directory::{Directory, JOB},
};
use crate::{End, Stop};
use jig_protocol_store::{Owner, Row};
use skein_forge::api::{Ci, Repository, State};
use temper_engine_domain::{Record, Write};
use temper_engine_domain_forge::{self as forge, Key, MissingLink, Need, RecordKind, Stored};

const A: Repository = Repository { forge: 1, repository: 7 };
const B: Repository = Repository { forge: 1, repository: 20 };
const CALL: forge::NamedCall = forge::NamedCall { task: 999, attempt: 1, completion: 2, position: 3 };
const PASSES: u32 = 10_000; // Calm directory open and generation switch, then the ordinary startup phases.

fn pull(repository: Repository) -> Stored {
    Stored::PullState(forge::PullState {
        name: forge::Name { forge: repository.forge, repository: repository.repository, what: forge::What::Pull(1) },
        head: [1; 32],
        ci: Ci::Passed,
        state: State::Closed,
    })
}
fn rows() -> Vec<Stored> {
    vec![
        Stored::Forge(fixtures::forge_record(1)),
        Stored::ForgeNumber { last: 1 },
        Stored::Repository(forge::Repository {
            contexts: Box::new([]),
            workflows: Box::new([]),
            project: 1,
            home: true,
            provider: A,
            host: b"forge.example".as_slice().into(),
            owner: b"demo".as_slice().into(),
            name: b"widget".as_slice().into(),
            prefix: b"temper/".as_slice().into(),
            role: forge::Role::Owned,
            ci: false,
            checks: Box::new([]),
            kinds: forge::Kinds {
                read: true,
                push: true,
                open: true,
                land: true,
                review: true,
                status: true,
                comment: true,
                issue: true,
                branch: true,
            },
            protection: forge::Protection::Absent,
            settings: skein_forge::api::Settings {
                default_branch: b"main".as_slice().into(),
                merge: true,
                squash: true,
                rebase: false,
            },
        }),
        pull(A),
        pull(B),
    ]
}
fn restarted(rows: Vec<Stored>) -> (Directory, crate::Service) {
    let mut disk = Directory::new();
    disk.seed(rows.into_iter().map(|row| Write::Save(Record::Forge { row: Box::new(row) })).collect());
    let mut service = disk.service();
    for _ in 0..PASSES {
        disk.turn(&mut service);
        assert_eq!(service.end(), None, "open before link check");
        if service.engine.as_ref().expect("engine").store.opened {
            break;
        }
    }
    let engine = service.engine.as_mut().expect("engine");
    assert!(engine.store.opened, "production open completed");
    let ran: Vec<_> = engine
        .notes
        .iter()
        .filter_map(|note| {
            if let crate::Note::Ran { of, job, counts, .. } = note { Some((*of, job.as_ref(), counts)) } else { None }
        })
        .collect();
    assert_eq!(ran.len(), 1, "only the pending job ran");
    let (of, job, counts) = ran[0];
    assert_eq!(of, Owner::Application);
    assert_eq!(job, JOB);
    for name in ["PullState", "dropped"] {
        assert_eq!(counts.iter().find(|count| count.name.as_ref() == name).expect("job count").count, 1, "{name}");
    }
    for (repository, expected) in [(A, Some(pull(A))), (B, None)] {
        let key = forge::stored_key(&pull(repository));
        let got = disk.get(&mut engine.store, &key).map(|row| {
            let Row::Application { key: found, value } = row else { panic!("application row") };
            assert_eq!(found, temper_protocol_store::keys::encode(&key, u32::MAX).expect("pull key"));
            value
        });
        let expected = expected.map(|row| temper_protocol_store::records::encode(&row));
        assert_eq!(got, expected, "pull state for {repository:?} after open");
    }
    (disk, service)
}

#[test]
fn deleted_repository_job_at_open_allows_the_restart_to_send_startup_wires() {
    let (mut disk, mut service) = restarted(rows());
    for _ in 0..PASSES {
        disk.turn(&mut service);
        assert_eq!(service.end(), None, "no forge link refusal after the drop job");
        if service.engine.as_ref().expect("engine").domain.ready() {
            break;
        }
    }
    let engine = service.engine.as_ref().expect("engine");
    assert_eq!(engine.domain.forge_missing_link(), None);
    assert_eq!(engine.domain.start_stop(), None);
    assert!(engine.store.host.mark_durable(), "startup wires sent after link check");
    assert!(engine.domain.ready(), "ordinary restart completed");
}

#[test]
fn deleted_repository_job_at_open_keeps_missing_call_tasks_strict_before_startup_wires() {
    let mut rows = rows();
    rows.push(Stored::Call { key: CALL, answer: forge::CallPayload::Refused(skein_forge::api::Error::Missing) });
    let (mut disk, mut service) = restarted(rows);
    for _ in 0..PASSES {
        disk.turn(&mut service);
        if service.end().is_some() {
            break;
        }
    }
    let link = MissingLink {
        kind: RecordKind::Call,
        key: Key::Call(CALL),
        need: Need::Task { task: CALL.task, retired: None },
    };
    let phase = Some(jig_domain_core::start::Phase::Link);
    assert_eq!(service.end(), Some(&End::Stopped(Stop::ForgeLink { phase, link: link.clone() })));
    let engine = service.engine.as_ref().expect("linking engine");
    assert_eq!(engine.domain.forge_missing_link(), Some(link.clone()));
    assert_eq!(engine.domain.start_stop(), Some(temper_engine_domain::StartStop::ForgeLink { phase, link }));
    assert!(!engine.store.host.mark_durable(), "no start wires before link");
    assert_eq!(engine.domain.start_phase(), phase, "no adoption before link");
    assert!(!engine.domain.ready(), "no decisions before link");
}
