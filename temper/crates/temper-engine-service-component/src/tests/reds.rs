//! Application asks use the admitted run and make no decision (domain/forge/reds.md, 5.1 and 5.4).
use super::fixtures;
use jig_domain_host as host;
use skein_lib::{Queue, Token};
use temper_engine_domain as root;
use temper_engine_domain_forge as forge;
use temper_protocol_worker_wire::{self as payload, v1};

pub(super) fn seed_reds(service: &mut crate::Service, task: u64) {
    let repository = forge::Repository {
        contexts: Box::new([]),
        workflows: Box::new([]),
        project: 1,
        home: true,
        provider: skein_forge::api::Repository { forge: 1, repository: 41 },
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
    };
    let hold = forge::Hold {
        name: forge::Name {
            forge: 1,
            repository: 41,
            what: forge::What::Branch(Box::new([
                b"temper".as_slice().into(),
                b"1".as_slice().into(),
                b"g7".as_slice().into(),
            ])),
        },
        task,
        writer: None,
        design_writers: Box::new([]),
        drift: None,
    };
    let mut rows = skein_lib::List::with_capacity(4);
    rows.push(forge::Stored::Repository(repository)).unwrap();
    rows.push(forge::Stored::Hold(hold)).unwrap();
    // Reverse insertion distinguishes the ordered snapshot from insertion order.
    for (generation, key, head) in
        [(1, b"test a::t x".as_slice(), [2; 32].as_slice()), (2, b"lint p c f", [1; 20].as_slice())]
    {
        rows.push(forge::Stored::Red(forge::Red {
            goal: task,
            key: key.into(),
            head: head.into(),
            why: b"base failure".as_slice().into(),
            by: forge::Recorder::Task(task),
            at: 9,
            generation,
        }))
        .unwrap();
    }
    for row in rows.into_boxed() {
        service.engine.as_mut().unwrap().step(root::Event::Forge(Box::new(forge::Event::Restore { record: row })));
    }
    fixtures::settle_root(service, &mut Queue::uncapped());
}

pub(super) fn answer(
    service: &mut crate::Service,
    run: &root::Assignment,
    bytes: &[u8],
    number: u64,
) -> host::AskAnswer {
    service.engine.as_mut().unwrap().step(root::Event::Worker(root::WorkerInput::Link {
        channel: Token::new(7),
        link: root::Link::Up {
            parts: root::Parts::none(),
            up: host::LinkUp::Ask {
                run: Token::new(run.task),
                attempt: Token::new(run.attempt),
                number,
                bytes: bytes.into(),
            },
        },
    }));
    let mut delivered = Queue::uncapped();
    fixtures::settle_root(service, &mut delivered);
    let mut found = None;
    while let Some(delivery) = delivered.pop() {
        if let root::Delivery::Link {
            channel,
            down: host::LinkDown::Asked { run: task, attempt, number: answered, answer },
        } = delivery
        {
            assert_eq!((channel.raw(), task.raw(), attempt.raw(), answered), (7, run.task, run.attempt, number));
            assert!(found.replace(answer).is_none(), "one answer");
        }
    }
    found.expect("the admitted ask is answered")
}

#[test]
fn an_admitted_reds_ask_reads_the_goal_in_key_order_without_a_commit() {
    let (mut service, run) = fixtures::claimed_chat();
    seed_reds(&mut service, run.task);
    let before = service.engine.as_ref().unwrap().domain.deployment().commits;
    let host::AskAnswer::Bytes(bytes) = answer(&mut service, &run, &payload::ask(&v1::Ask::Reds), 19) else {
        panic!("snapshot")
    };
    let v1::Answer::Reds(reds) =
        v1::Answer::decode(&fixtures::wire_limits(), &mut skein_lib::Reader::new(&bytes)).unwrap();
    let rows = reds.reds();
    assert_eq!(rows.len(), 2);
    assert_eq!(
        (rows.get(0).unwrap().key(), rows.get(0).unwrap().head()),
        (b"lint p c f".as_slice(), [1; 20].as_slice())
    );
    assert_eq!(
        (rows.get(1).unwrap().key(), rows.get(1).unwrap().head()),
        (b"test a::t x".as_slice(), [2; 32].as_slice())
    );
    assert_eq!(service.engine.as_ref().unwrap().domain.deployment().commits, before);
}

#[test]
fn a_run_without_a_goal_gets_empty_reds_and_malformed_asks_are_refused() {
    let (mut service, run) = fixtures::claimed_chat();
    let before = service.engine.as_ref().unwrap().domain.deployment().commits;
    assert_eq!(answer(&mut service, &run, &[0], 1), host::AskAnswer::Bytes(Box::new([0; 5])));
    for bytes in [b"".as_slice(), &[1], &[0, 0]] {
        assert_eq!(answer(&mut service, &run, bytes, 2), host::AskAnswer::Refused, "malformed {bytes:?}");
    }
    assert_eq!(service.engine.as_ref().unwrap().domain.deployment().commits, before);
}
