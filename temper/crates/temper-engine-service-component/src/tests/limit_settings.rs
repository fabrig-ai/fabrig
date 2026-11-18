//! A restart reads persisted settings before constructing the root.
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "ordinary focused tests edit startup documents and restart a memory store"
)]
mod admission;
mod reading;
use super::fixtures;
use crate::{
    limit_settings::{LimitInputs, LimitSettings},
    startup,
    store::Store,
};
use jig_domain_core::{self as core, settings as s};
use skein_lib::{List, Token};
use temper_engine_domain_forge::{self as forge, settings as f};

fn inputs() -> LimitInputs {
    use ::core::num::NonZeroU16;
    let generated = temper_init::files(
        &temper_init::Facts {
            sign_in: false,
            binary: b"/bin/temper",
            git: Some(b"/bin/git"),
            claude: None,
            login: Some(b"ada"),
            ports: temper_init::Ports {
                clients: NonZeroU16::new(8367).unwrap(),
                llm: NonZeroU16::new(8370).unwrap(),
                webhooks: NonZeroU16::new(8371).unwrap(),
            },
        },
        &temper_init::Directories { config: b"/config", data: b"/data" },
        &[0; 32],
        &[1; 32],
    )
    .unwrap();
    let file = generated.files.into_iter().find(|file| file.name == b"temper.json").unwrap();
    let temper_init::Content::Bytes { bytes, .. } = file.content else {
        panic!("document");
    };
    let mut document: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    for provider in document["engine"]["llm"]["providers"].as_array_mut().unwrap() {
        provider["sources"].as_object_mut().unwrap().remove("sign_in");
        provider["signed_in_plan"] = false.into();
    }
    document["workers"] = serde_json::json!([]);
    document["engine"]["hosts"]["listener"] = serde_json::json!([{"socket":"127.0.0.1:0"}]);
    let bytes = serde_json::to_vec(&document).unwrap();
    let mut files = crate::settings::Files { entries: List::with_capacity(1) };
    files.entries.push((b"worker.digest".as_slice().into(), skein_lib::Secret::new(Box::new([b'0'; 64])))).unwrap();
    LimitInputs::new(
        &bytes,
        &jig_protocol_config::Directories::working(),
        &files,
        None,
        &fixtures::seeds(),
        &crate::stock::Given::Stock,
        false,
        None,
    )
}
fn item(name: &[u8], value: u64) -> s::Override {
    s::Override { name: name.into(), value, reason: b"restart room".as_slice().into() }
}
fn opening(inputs: &LimitInputs, kept: Option<jig_protocol_store::Stores>) -> Store {
    let mut store = Store::for_limits(inputs, &[]).unwrap();
    let limits = store.opening_limits();
    let (jig, own) = inputs.stock().unwrap();
    let root = crate::limits::engine(&jig, &own).unwrap();
    let application = temper_protocol_store::application(&root.forge, root.tasks.spec_bytes).unwrap();
    let kept = kept.unwrap_or_else(|| jig_protocol_store::Stores {
        store: skein_kv::Store::memory(&limits.store),
        secrets: skein_kv::Store::memory(&limits.secrets),
    });
    store.host.component = jig_protocol_store::Store::over(&limits, application, &[], kept);
    store.host.ask(jig_protocol_store::Request::Open {
        purpose: jig_protocol_store::Purpose::Serve,
        owner: Token::new(1),
        root: jig_protocol_store::Root { token: Token::new(2), fd: skein_io::kernel::Fd::new(3) },
        installation: inputs.seeds.installation,
    });
    let Some(jig_protocol_store::Event::Opened { installation, pending, .. }) = store.events().pop() else {
        panic!("memory open");
    };
    store.opened_limits(installation, pending);
    store
}
fn build(store: Store, state: LimitSettings) -> crate::Service {
    let input = &state.inputs;
    let start = startup::checked_start_with_limits(
        &input.document,
        &input.directories,
        &input.files,
        None,
        &input.seeds,
        List::with_capacity(0),
        None,
        &input.given,
        &state.in_force.jig,
        &state.in_force.temper,
    )
    .unwrap();
    let mut service =
        crate::Service::prepared(start.engine, &input.seeds, None, start.forges, start.io, None, None).unwrap();
    service.engine.as_mut().unwrap().take_limit_store(store, state);
    service
}
fn changed(jig: Box<[s::Override]>, temper: Box<[s::Override]>) -> Box<[jig_protocol_store::Write]> {
    let value = f::Value::Limits(temper);
    let size = temper_engine_smith::kinds::size(&value);
    let row = forge::Stored::Setting(f::SettingRow {
        key: f::SettingKey { kind: 22, scope: 0, number: 1, slot: f::SettingSlot::Change },
        edit: f::Edit::Set(f::SizedValue { value, size }),
    });
    Box::new([
        jig_protocol_store::Write::Save(core::Record::Core(core::CoreRecord::Settings(s::Record::Changed {
            setting: s::Setting::Limits,
            changed: s::Changed { value: Some(s::Value::Limits(jig)), revision: 1 },
        }))),
        jig_protocol_store::Write::Put {
            key: temper_protocol_store::keys::encode(&forge::stored_key(&row), u32::MAX).unwrap(),
            value: temper_protocol_store::records::encode(&row),
        },
    ])
}
fn save(service: &mut crate::Service, jig: Box<[s::Override]>, temper: Box<[s::Override]>) {
    save_rows(service, jig, temper, &[]);
}
fn save_rows(
    service: &mut crate::Service,
    jig: Box<[s::Override]>,
    temper: Box<[s::Override]>,
    rows: &[forge::Stored],
) {
    save_all(service, jig, temper, rows, &[]);
}
fn save_all(
    service: &mut crate::Service,
    jig: Box<[s::Override]>,
    temper: Box<[s::Override]>,
    rows: &[forge::Stored],
    core_rows: &[core::Record],
) {
    let engine = service.engine.as_mut().unwrap();
    let mut header = engine.domain.deployment();
    header.commits = header.commits.checked_add(1).unwrap();
    let mut writes = List::with_capacity(
        3_u32.checked_add(u32::try_from(rows.len().checked_add(core_rows.len()).unwrap()).unwrap()).unwrap(),
    );
    writes.push(jig_protocol_store::Write::Save(core::Record::Core(core::CoreRecord::Deployment(header)))).unwrap();
    for write in changed(jig, temper) {
        writes.push(write).unwrap();
    }
    for row in core_rows {
        writes.push(jig_protocol_store::Write::Save(row.clone())).unwrap();
    }
    for row in rows {
        writes
            .push(jig_protocol_store::Write::Put {
                key: temper_protocol_store::keys::encode(&forge::stored_key(row), u32::MAX).unwrap(),
                value: temper_protocol_store::records::encode(row),
            })
            .unwrap();
    }
    engine.store.host.ask(jig_protocol_store::Request::Commit { number: header.commits, writes: writes.into_boxed() });
    let Some(jig_protocol_store::Event::Committed { .. }) = engine.store.events().pop() else {
        panic!("settings commit");
    };
    // The fixture's direct commit is durable before this process is crashed.
}
fn kept(service: crate::Service) -> jig_protocol_store::Stores {
    service.engine.unwrap().store.host.component.into_stores()
}
fn repository(number: u32) -> forge::Stored {
    forge::Stored::Repository(forge::Repository {
        contexts: Box::new([]),
        workflows: Box::new([]),
        project: 1,
        home: true,
        provider: skein_forge::api::Repository { forge: 1, repository: number },
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
    })
}
#[test]
fn lowered_codec_settings_refuse_restart_and_held_stock_recovers_the_same_store() {
    for temper in [false, true] {
        let input = inputs();
        let (stock_jig, stock_own) = input.stock().unwrap();
        let mut store = opening(&input, None);
        let state = store.resolve_settings(input, None).unwrap();
        let mut service = build(store, state);
        fixtures::restart(&mut service);
        let (name, configured, held, stock, kind, row_key) = if temper {
            let mut row = repository(1);
            let forge::Stored::Repository(value) = &mut row else { unreachable!() };
            value.host = Box::new([b'x'; 60]);
            let key = temper_protocol_store::keys::encode(&forge::stored_key(&row), u32::MAX).unwrap();
            save_rows(
                &mut service,
                Box::new([]),
                Box::new([item(b"forge.name_bytes", 40)]),
                &[forge::Stored::Forge(fixtures::forge_record(1)), row],
            );
            (
                b"forge.name_bytes".as_slice(),
                40,
                60,
                u64::from(stock_own.forge.name_bytes),
                startup::LimitKind::Temper,
                jig_protocol_store::keys::application(&key, u32::MAX).unwrap(),
            )
        } else {
            let row = core::Record::People(jig_domain_people::Stored::Person {
                number: 77,
                identity: jig_domain_people::Identity {
                    key: jig_domain_people::IdentityKey { provider: 1, subject: b"a person".as_slice().into() },
                    login: b"ada".as_slice().into(),
                    name: Box::new([b'x'; 300]),
                },
            });
            let key = jig_protocol_store::keys::encode(&row.key(), u32::MAX).unwrap();
            save_all(&mut service, Box::new([item(b"people.identity_bytes", 200)]), Box::new([]), &[], &[row]);
            (
                b"people.identity_bytes".as_slice(),
                200,
                300,
                u64::from(stock_jig.core.people.identity_bytes),
                startup::LimitKind::Jig,
                key,
            )
        };
        let input = inputs();
        let mut store = opening(&input, Some(kept(service)));
        let state = store.resolve_settings(input, None).unwrap();
        let mut service = build(store, state);
        for _ in 0_u32..512 {
            fixtures::turn(&mut service);
            if service.end().is_some() {
                break;
            }
        }
        let Some(crate::End::Stopped(crate::Stop::CodecRestore { bound, kind: found, row: Some(site) })) =
            service.end()
        else {
            panic!("codec lowering: {:?}", service.end());
        };
        assert_eq!(*bound, jig_protocol_store::codec::Lowered { name, configured, held });
        assert_eq!(*found, kind);
        assert_eq!(site.key, row_key);
        assert!(!format!("{:?}", service.end()).contains("xxxxxxxx"));
        let input = inputs();
        let mut store = opening(&input, Some(kept(service)));
        let over = [item(name, stock)];
        let held: (&[s::Override], &[s::Override]) = if temper { (&[], &over) } else { (&over, &[]) };
        let state = store.resolve_settings(input, Some(held)).unwrap();
        assert!(state.held);
        let mut service = build(store, state);
        for _ in 0_u32..512 {
            fixtures::turn(&mut service);
            if !service.work_pending(skein_lib::Time::from_nanos(1)) {
                break;
            }
        }
        assert!(service.engine.as_ref().unwrap().domain.ready(), "held recovery: {:?}", service.end());
    }
}

#[test]
fn a_lowered_setting_names_the_figure_and_the_work_the_store_holds() {
    let input = inputs();
    let mut store = opening(&input, None);
    let state = store.resolve_settings(input, None).unwrap();
    let mut service = build(store, state);
    fixtures::restart(&mut service);
    save_rows(&mut service, Box::new([]), Box::new([item(b"forge.repositories", 1)]), &[repository(1), repository(2)]);
    let input = inputs();
    let mut store = opening(&input, Some(kept(service)));
    let state = store.resolve_settings(input, None).unwrap();
    let mut service = build(store, state);
    for _ in 0_u32..512 {
        fixtures::turn(&mut service);
        if service.end().is_some() {
            break;
        }
    }
    assert_eq!(
        service.end(),
        Some(&crate::End::Stopped(crate::Stop::LimitRestore {
            name: b"forge.repositories",
            kind: startup::LimitKind::Temper,
            configured: 1,
            held: 2,
        })),
        "phase={:?}, limit={}, pending={:?}",
        service.engine.as_ref().unwrap().domain.start_phase(),
        service.engine.as_ref().unwrap().limits().forge.repositories,
        service.pending(skein_lib::Time::from_nanos(1))
    );
    assert!(!service.engine.as_ref().unwrap().domain.ready());
}

#[test]
fn a_written_store_resolves_both_kinds_before_a_restarted_engine_restores() {
    let input = inputs();
    let mut store = opening(&input, None);
    let stock = store.opening_limits();
    let state = store.resolve_settings(input, None).unwrap();
    let mut service = build(store, state);
    fixtures::restart(&mut service);
    let input = inputs();
    let mut store = opening(&input, Some(kept(service)));
    let state = store.resolve_settings(input, None).unwrap();
    assert!(state.jig.is_empty() && state.temper.is_empty(), "defaults are not changes");
    assert_eq!(state.in_force.temper.live_goals, 40);
    let mut service = build(store, state);
    fixtures::restart(&mut service);
    save(
        &mut service,
        Box::new([item(b"tasks.tasks", 1024)]),
        Box::new([
            item(b"forge.client.entries", 16),
            item(b"forge.entries", 16),
            item(b"forge.gone", 1),
            item(b"live_goals", 20),
        ]),
    );
    let input = inputs();
    let mut store = opening(&input, Some(kept(service)));
    let state = store.resolve_settings(input, None).unwrap();
    assert_eq!(state.jig[0].name.as_ref(), b"tasks.tasks");
    assert_eq!(state.in_force.jig.core.tasks.tasks, 1024);
    assert_eq!(state.in_force.jig.core.tasks.project_tasks, 384);
    assert_eq!(state.in_force.temper.forge.entries, 16);
    assert_eq!(state.in_force.left_out.len(), 1);
    assert_eq!(state.in_force.left_out.get(0).unwrap().name.as_ref(), b"forge.gone");
    assert_eq!(store.opening_limits().store, stock.store);
    let mut service = build(store, state);
    fixtures::restart(&mut service);
    assert_eq!(service.engine.as_ref().unwrap().limits().tasks.tasks, 1024);
    assert_eq!(service.engine.as_ref().unwrap().limits().forge.entries, 16);
    save(
        &mut service,
        Box::new([item(b"tasks.tasks", 512)]),
        Box::new([item(b"forge.client.entries", 16), item(b"forge.entries", 16), item(b"live_goals", 20)]),
    );
    let input = inputs();
    let mut store = opening(&input, Some(kept(service)));
    let state = store.resolve_settings(input, None).unwrap();
    assert_eq!(state.in_force.jig.core.tasks.tasks, 512, "a jig setting lowers its count below the stock");
    assert_eq!(store.opening_limits().store, stock.store);
    assert_eq!(store.opening_limits().secrets, stock.secrets);
    assert_eq!(store.opening_limits().commits, stock.commits);
    let mut service = build(store, state);
    fixtures::restart(&mut service);
    assert_eq!(service.engine.as_ref().unwrap().limits().tasks.tasks, 512);
    save(
        &mut service,
        Box::new([item(b"tasks.tasks", 1024)]),
        Box::new([item(b"forge.entries", 16), item(b"live_goals", 80)]),
    );
    let input = inputs();
    let mut store = opening(&input, Some(kept(service)));
    let found = store.resolve_settings(input, None).unwrap_err();
    assert_eq!(startup::lines(&found).as_ref(), b"live_goals (setting temper-limits): the store's map value, 199008107 bytes, past its bound of 100881496 bytes; put limits-held in the data directory to start at the stock\n");
    let state = store.resolve_settings(inputs(), Some((&[], &[]))).unwrap();
    assert!(state.held);
    assert_eq!(state.temper[1].value, 80);
    assert_eq!(state.in_force.temper.live_goals, 40);
    let mut service = build(store, state);
    fixtures::restart(&mut service);
    assert_eq!(service.engine.as_ref().unwrap().limits().tasks.tasks, 768);
}

fn agents_with_payload(payload: u32) -> f::AgentFigures {
    use temper_client_protocol_wire::v1 as w;
    let mut parts = crate::stock::agents().into_parts();
    parts.profile = List::from_boxed(Box::new([w::AgentFigure::new(
        &w::CEILINGS,
        w::AgentFigureParts { name: b"declared.tool_payload".as_slice().into(), value: u64::from(payload) },
    )
    .unwrap()]));
    w::AgentFigures::new(&w::CEILINGS, parts).unwrap()
}

#[test]
fn restart_derives_carriers_from_the_larger_of_the_limit_and_stored_agents_payload() {
    for (floor, expected) in [(8192, 32_000), (48_000, 48_000)] {
        let input = inputs();
        let mut store = opening(&input, None);
        let state = store.resolve_settings(input, None).unwrap();
        let mut service = build(store, state);
        fixtures::restart(&mut service);
        let value = f::Value::AgentFigures(agents_with_payload(32_000));
        let size = temper_engine_smith::kinds::size(&value);
        let row = forge::Stored::Setting(f::SettingRow {
            key: f::SettingKey { kind: 21, scope: 0, number: 1, slot: f::SettingSlot::Change },
            edit: f::Edit::Set(f::SizedValue { value, size }),
        });
        save_rows(&mut service, Box::new([]), Box::new([item(b"tool_payload", floor)]), &[row]);
        let input = inputs();
        let mut store = opening(&input, Some(kept(service)));
        let state = store.resolve_settings(input, None).unwrap();
        assert_eq!(
            state.in_force.temper.tool_payload,
            u32::try_from(floor).unwrap(),
            "the limit retains its own meaning"
        );
        assert_eq!(state.in_force.payload, expected);
        assert_eq!(state.in_force.jig.core.tasks.message_bytes, expected - 2);
        assert_eq!(state.in_force.temper.checkout.message_bytes, expected);
    }
}
