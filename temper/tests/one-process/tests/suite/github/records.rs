//! The GitHub fixture's records through the deployment store (temper/docs/design/protocol/credentials.md, 2.1).
use jig_domain_core::{CoreKey, CoreRecord, Deployment, Key, Record};
use jig_protocol_store::{Event, Get, Request, Row, SecretOwner, SecretValue, Secrets, Write};
use skein_fake_machine::Machine;
use skein_lib::{Secret, Token};
use temper_engine_service::store::Store;
use temper_github_app::{APP, records};
use temper_one_process_world::testing::{Disk, USER};
use temper_protocol_records::Item;
use temper_protocol_records_forge as forge_records;

const OWNER: Token = Token::new(1);

fn open(disk: &mut Disk) -> Store {
    let jig = jig_protocol_config_fixtures::small();
    let own = temper_limits_fixtures::small();
    let root = temper_engine_service::limits::engine(&jig, &own).expect("engine limits");
    let limits = temper_engine_service::store::limits(&root, &root.forge, &jig.store).expect("store limits");
    let seeds = temper_engine_service::settings::Seeds {
        deployment: [1; 16],
        installation: [0; 16],
        seed: 0,
        crypto: Secret::new(Box::new([7; 32])),
        link_secret: None,
        local_secret: None,
    };
    let mut store = Store::new(&root, &limits, &[], &seeds);
    let root = disk.root(0);
    store.adopt(root, USER).expect("adopt deployment root");
    let Event::Opened { last, pending, .. } = disk.event(&mut store) else { panic!("fixture store opened") };
    if pending {
        store.ask(Request::Switch { owner: OWNER });
        assert!(matches!(disk.event(&mut store), Event::Switched { .. }));
    }
    store.ask(Request::Get { owner: OWNER, key: Get::Jig(Key::Core(CoreKey::Deployment)) });
    let mut header = match disk.event(&mut store) {
        Event::Got { row: Some(Row::Jig(Record::Core(CoreRecord::Deployment(header)))), .. } => header,
        Event::Got { row: None, .. } => Deployment {
            id: seeds.deployment,
            restart_wait_ms: None,
            wait_floors: None,
            tasks: 0,
            people: 0,
            sign_ins: 0,
            messages: 0,
            runs: 0,
            calls: 0,
            connector_rows: 0,
            applications: 0,
            commits: 0,
            projects: 0,
        },
        other => panic!("fixture deployment lookup: {other:?}"),
    };
    header.commits = last + 1;
    store.ask(Request::Commit {
        number: header.commits,
        writes: Box::new([Write::Save(Record::Core(CoreRecord::Deployment(header))), Write::Wires]),
    });
    assert!(matches!(disk.event(&mut store), Event::Committed { number, .. } if number == last + 1));
    store
}

#[test]
fn the_seeded_records_are_what_temper_loads_at_start() {
    let origin: std::net::SocketAddr = "127.0.0.1:4321".parse().expect("fixture forge address");
    let records = records(&APP, 17, &[3, 1, 3], origin);
    let mut machine = Machine::for_user(USER);
    let root = machine.lay(&[]);
    machine.close(root);
    let mut disk = Disk::new(machine);
    let mut store = open(&mut disk);
    for (number, bytes) in &records {
        store.ask(Request::Secrets {
            owner: OWNER,
            request: Secrets::Keep {
                owner: SecretOwner::Application,
                number: *number,
                value: SecretValue::Application(Secret::new(bytes.clone())),
            },
        });
        assert!(matches!(disk.event(&mut store), Event::Kept { .. }));
    }
    disk.close(store);
    let mut store = open(&mut disk);
    let mut loaded = Vec::new();
    let mut after = None;
    loop {
        store.ask(Request::Secrets { owner: OWNER, request: Secrets::Load { owner: SecretOwner::Application, after } });
        let Event::SecretsLoaded { rows, next, .. } = disk.event(&mut store) else { panic!("durable fixture records") };
        for row in rows {
            if let SecretValue::Application(value) = row.value {
                loaded.push((row.number, value));
            }
        }
        after = next;
        if after.is_none() {
            break;
        }
    }
    disk.close(store);
    assert_eq!(loaded.len(), 6);
    let mut rows = forge_records::Rows::new();
    for (number, value) in loaded {
        assert!(records.iter().any(|(expected, bytes)| (*expected, bytes.as_ref()) == (number, value.expose())));
        rows.push(number, value, 4096);
    }
    let rows = rows.forge(17).expect("fixture forge records");
    assert!(rows.get(Item::AppKey).expect("fixture app key") == skein_fake_forge_github::keys::KEYS[0]);
    let start = forge_records::access(
        skein_forge_connection::settings::Kind::GitHub,
        &forge_records::Origin { kind: skein_forge_connection::settings::Kind::GitHub, api: origin.to_string().into() },
        rows,
        &skein_forge_setup::item::Bounds { token_bytes: 512, name_bytes: 256, key_bytes: 4096, installations: 8 },
        false,
    )
    .expect("fixture records grant app access");
    assert_eq!(start.access.user, APP.bot);
    assert_eq!(start.slug.as_deref(), Some(APP.slug.as_bytes()));
    assert_eq!(&*start.accepted, &[1, 3]);
    match start.access.credential {
        skein_forge::credential::Credential::App { id, key } => {
            assert_eq!(id, APP.id);
            assert!(key.expose() == skein_fake_forge_github::keys::KEYS[0]);
        }
        skein_forge::credential::Credential::Token(_) => panic!("App access expected"),
    }
}
