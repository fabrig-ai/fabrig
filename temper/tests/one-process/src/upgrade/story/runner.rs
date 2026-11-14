//! Production setup, calibration, cuts and the next start on the same store.
use super::{
    HarnessProcess, Links, Machine, Maps, Person, Proc, STATE, STORE, SignalHost, State, Story, Upgrade, ctl, cuts,
    fixtures, forge_peer, golden, image, judge::Judge, machine, mutations, programs, referee, stories, world,
};
use std::{cell::RefCell, rc::Rc};
/// The service stops settled, then starts with the mark-only fixture sets.
pub fn settled() {
    run(None, None);
}
/// Kill the service with its sent write still in flight.
pub fn unsettled() {
    run(Some(Selection::MeasureKill), None);
    let before = STATE.with(|s| s.borrow().write_at.expect("write in flight"));
    run(None, Some(before));
}

/// A fixed completed operation during the copy-to-wires interval.
pub fn power_loss(point: usize) {
    run(Some(Selection::Fixed(point)), None);
}
/// A seed draws one service submission across the whole upgrade start.
pub fn drawn_power_loss(seed: u64) {
    run(Some(Selection::Drawn(seed)), None);
}
enum Selection {
    Fixed(usize),
    Drawn(u64),
    MeasureKill,
}

fn run(selection: Option<Selection>, kill: Option<u32>) {
    STATE.with(|s| *s.borrow_mut() = State { end_after_kill: kill.is_some(), ..State::default() });
    stories::set(Story::PlaygroundA);
    stories::setup_settings(world::Phase::Story);
    let start = temper_playground::stories::A.start(fixtures::SEED, skein_sim::Config::calm().wall);
    let mut configuration: serde_json::Value =
        serde_json::from_slice(&golden::configuration(&start.deployment)).expect("upgrade configuration");
    // The lost-write recovery and settling fit this story's sixty-second bound.
    configuration["application"]["limits"]["overrides"]["forge.client.lifetime_ms"] = 10_000.into();
    let configuration = serde_json::to_vec(&configuration).expect("upgrade configuration");
    let mut disk = machine::fresh();
    temper_playground::story::prepare(&mut disk, &start.deployment, start.seed);
    let store = Rc::new(RefCell::new(referee::Store::new()));
    let ledger = Rc::new(RefCell::new(Vec::new()));
    let disk = phase(
        disk,
        &configuration,
        &start.fakes.forge,
        &store,
        &ledger,
        false,
        kill.map(|before| (before, skein_world::Cut::Kill)),
    );
    if kill.is_none() {
        assert!(store.borrow().pending.is_empty(), "settled stop has no pending commit");
    } else {
        assert!(STATE.with(|s| s.borrow().killed), "the service was killed, not drained");
    }
    STATE.with(|s| s.borrow_mut().end_after_kill = false);
    if matches!(selection, Some(Selection::MeasureKill)) {
        world::lower_store(None);
        STORE.with(|s| *s.borrow_mut() = None);
        return;
    }
    let mut disk = disk;
    let ledger_before = mutations(&ledger.borrow());
    let before = Maps::read(&mut disk);
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        s.before = Some(before);
        s.upgrade = true;
        s.points = cuts::Points::default();
    });
    store.borrow_mut().recovering();
    if let Some(selection) = selection {
        let image = image::Image::read(&mut disk);
        let _calibration = phase(disk, &configuration, &start.fakes.forge, &store, &ledger, true, None);
        let before = STATE.with(|s| {
            let s = s.borrow();
            let points = s.points.fixed();
            assert!(
                points.windows(2).all(|pair| pair[0] <= pair[1]),
                "copy, patch, switch, wires in order: {points:?}"
            );
            match selection {
                Selection::MeasureKill => unreachable!("kill measured before the upgrade"),
                Selection::Fixed(index) => points[index],
                Selection::Drawn(seed) => {
                    let mut draw = skein_lib::Rng::new(seed);
                    points[0] + u32::try_from(draw.below(u64::from(points[3] - points[0] + 1))).expect("bounded cut")
                }
            }
        });
        eprintln!("upgrade power cut before service submission {before}");
        store.borrow_mut().recovering();
        store.borrow_mut().agent_records.clear();
        let disk = image.restore(&mut store.borrow_mut());
        STATE.with(|s| {
            let mut s = s.borrow_mut();
            s.checked = false;
            s.points = cuts::Points::default();
        });
        let _disk = phase(
            disk,
            &configuration,
            &start.fakes.forge,
            &store,
            &ledger,
            true,
            Some((before, skein_world::Cut::PowerLoss)),
        );
        assert_eq!(STATE.with(|s| s.borrow().epoch), 2, "power loss restarted the service");
    } else {
        let _disk = phase(disk, &configuration, &start.fakes.forge, &store, &ledger, true, None);
    }
    assert!(STATE.with(|s| s.borrow().checked), "history checked before the root acts");
    assert_eq!(
        mutations(&ledger.borrow()),
        ledger_before,
        "no forge mutation is repeated while recovering or continuing"
    );
    assert_eq!(ledger.borrow().iter().filter(|o| matches!(o, skein_fake_forge_domain::Observation::Opened {title,..} if title.as_ref() == b"Upgrade once")).count(), 1, "uncertain write made once across starts");
    world::lower_store(None);
    STORE.with(|s| *s.borrow_mut() = None);
}

fn phase(
    disk: skein_fake_machine::Machine,
    configuration: &[u8],
    transport: &temper_fake_git_hosted::Typed,
    store: &Rc<RefCell<referee::Store>>,
    ledger: &Rc<RefCell<Vec<skein_fake_forge_domain::Observation>>>,
    upgrade: bool,
    cut: Option<(u32, skein_world::Cut)>,
) -> skein_fake_machine::Machine {
    world::lower_store(Some(store.clone()));
    STORE.with(|s| *s.borrow_mut() = Some(store.clone()));
    let mut disk = disk;
    machine::lay_handover(&mut disk, b"client/private/address", format!("http://{}\n", fixtures::CLIENTS).as_bytes());
    let client = (!upgrade).then(|| disk.reopen_root(2));

    let machine = Machine::new(disk);
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        s.disk = Some(machine.0.clone());
        s.epoch = 0;
        s.epoch_floor = None;
        s.transport = Some(transport.clone());
    });
    let document = serde_json::from_slice(configuration).expect("configuration");
    let params = programs::Parameters {
        deployment: configuration.into(),
        seed: fixtures::SEED,
        links: Links::Keep,
        forge: transport.clone(),
        route: temper_playground::story::route(&document),
        disk: machine.0.clone(),
    };
    let mut config = skein_sim::Config::calm();
    if upgrade {
        config.wall = STATE.with(|s| s.borrow().stopped_wall.expect("wall at stop"));
    }
    let mut outcome = programs::with(params, || {
        let mut world =
            skein_world::World::new_controlled(fixtures::SEED, config, skein_world::Memory::Unchecked, |controls| {
                Judge::new(controls, store.clone(), transport.clone(), upgrade, ledger.clone())
            })
            .with_machine(machine);
        for (program, roots) in programs::programs::<Upgrade>() {
            world.host_roots(program, roots);
        }
        world.host_roots(
            ctl::program(|spawn, inherited| Proc::Other(HarnessProcess::Ctl(Box::new(ctl::host(spawn, inherited))))),
            ctl::roots,
        );
        world.spawn_restartable(programs::service_roots(), |inherited| {
            if STATE.with(|s| s.borrow().end_after_kill && s.borrow().epoch > 0) {
                STATE.with(|s| s.borrow_mut().killed = true);
                let mut closer = ctl::Caller::scripted_restartable(inherited);
                closer.closed = true;
                Proc::Other(HarnessProcess::Calls(Box::new(closer)))
            } else {
                programs::service::<Upgrade>(inherited)
            }
        });
        world.spawn_restartable(
            vec![skein_world::StartupRoot { name: b"private".as_slice().into(), path: b"client".as_slice().into() }],
            |inherited| Proc::Other(HarnessProcess::Calls(Box::new(ctl::Caller::scripted_restartable(inherited)))),
        );
        if !upgrade {
            world.spawn_root(skein_sim::Handle::new(client.expect("first client").raw()), |root| {
                Proc::Other(HarnessProcess::Client(Box::new(SignalHost::new(
                    Person::welcome(root, golden::person_steps()).with_account_after(7),
                    None,
                ))))
            });
        }
        world.spawn_restartable(vec![], programs::llm::<Upgrade>);
        world.spawn_restartable(vec![], |inherited| {
            let transport = STATE.with(|s| s.borrow().transport.as_ref().expect("forge").clone());
            programs::fake_forge(forge_peer(&transport), inherited)
        });
        if let Some((before, kind)) = cut {
            world.cut(0, before, kind);
        }
        world.run()
    });
    for proc in &mut outcome.procs {
        if !upgrade && let Proc::Forge(peer) = proc {
            STATE.with(|s| {
                let mut s = s.borrow_mut();
                s.tokens = peer.setup_mut().issued().iter().map(|t| (t.secret.clone(), t.user)).collect();
                s.users = [b"temper".as_slice(), b"owner"]
                    .into_iter()
                    .filter_map(|login| peer.setup_mut().find(login).map(|a| (a.id, a.login.clone())))
                    .collect();
            });
        }
    }
    if let Proc::Service(service) = &outcome.procs[0] {
        assert!(service.service.is_empty(), "settled service stop");
    } else {
        assert!(STATE.with(|s| s.borrow().killed));
    }
    std::mem::replace(&mut outcome.machine.0.borrow_mut(), machine::fresh())
}
