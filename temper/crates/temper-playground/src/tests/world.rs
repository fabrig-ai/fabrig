//! The calm composition's startup and replay (temper/docs/design/playground.md, 4).
use crate::{
    documents, fakes,
    programs::Process,
    story::{self, Fakes, Links, Start},
};
use skein_lib::Wall;
use skein_shell::Host;
use skein_sim_world::{StateDigests, World};
use std::{cell::RefCell, rc::Rc};

fn start(links: Links) -> Start {
    let seed = 7;
    let seeds = documents::seeds(seed);
    let llm = fakes::scripted_provider(
        Box::new([skein_fake_llm_domain::api::Script {
            cue: b"Hello".as_slice().into(),
            turns: Box::new([skein_fake_llm_domain::api::Turn {
                lines: Box::new([skein_fake_llm_domain::api::Line::Text { text: b"Hello back".as_slice().into() }]),
                finish: skein_fake_llm_domain::api::Finish::Stop,
                tokens: 4,
            }]),
        }]),
        seeds.llm,
    );
    let forge = Rc::new(RefCell::new(skein_fake_peers::forge::Forge::new(
        skein_fake_forge_domain::Domain::new(&documents::hosted_git::CONFIG, seeds.forge),
        documents::hosted_git::CONFIG,
    )));
    let forge = temper_fake_git::Typed::new(forge, &fakes::forge_table());
    Start {
        seed,
        wall: Wall::EPOCH,
        deployment: documents::document().into(),
        fakes: Fakes::new(llm, forge, seeds.forge),
        links,
    }
}
fn idle(world: &mut World<Process<()>, crate::machine::Machine>) {
    for step in 0..10_000 {
        story::step(world);
        if !world.busy() && world.next_due().is_none_or(|due| due > world.now()) {
            return;
        }
        if !world.busy()
            && let Some(due) = world.next_due()
        {
            world.advance(due);
        }
        if step == 9_999 {
            let states: Vec<_> = world
                .processes()
                .iter()
                .map(|process| (process.work_pending(world.now()), process.next_deadline(), process.exit()))
                .collect();
            let Process::Service(service) = &world.processes()[0] else { panic!("service first") };
            panic!(
                "world did not wait idle with nothing due at step {step}, now {:?}, due {:?}, states {states:?}, service {:?}, pending {:?}",
                world.now(),
                world.next_due(),
                service.service.wait_state(),
                service.service.pending(world.now())
            );
        }
    }
}
fn digests(world: &mut World<Process<()>, crate::machine::Machine>) -> StateDigests {
    use skein_sim_world::Machine;
    let procs = world.processes().iter().map(Host::state_digest).collect();
    let (sim, _, _) = world.parts();
    assert!(sim.trace().is_empty(), "a playground keeps its trace off");
    // The simulator has no pending call at this idle boundary.
    // Read the disk through the service's shared startup parameters.
    let Process::Service(service) = &world.processes()[0] else { panic!("service first") };
    let disk = crate::machine::Machine(service.parameters.disk.clone());
    StateDigests { procs, machine: disk.state_digest() }
}
#[test]
fn the_world_starts_calm_and_prints_its_sign_in_link() {
    let mut previous = None;
    for _ in 0..2 {
        let mut world = story::world(start(Links::Print));
        assert_eq!(world.seed(), 7);
        assert_eq!(world.wall(), Wall::EPOCH);
        idle(&mut world);
        let lines = world.printed();
        assert_eq!(lines.len(), 1, "one sign-in link");
        assert_eq!(lines[0].process.as_ref(), b"temper serve");
        assert_eq!(lines[0].stream, skein_sim_world::Stream::Output);
        let link = std::str::from_utf8(&lines[0].line).expect("sign-in URL");
        let prefix = format!("http://{}/jig/v1/sign-in/link?secret=", documents::CLIENTS);
        let secret = link.strip_prefix(&prefix).expect("serve's link form");
        assert!(!secret.is_empty() && !secret.contains(['\r', '\n']), "one whole link line");
        assert!(
            world
                .processes()
                .iter()
                .all(|process| process.exit().is_none_or(|exit| exit == skein_io::kernel::Exit::Code(0)))
        );
        let Process::Service(service) = &world.processes()[0] else { panic!("service first") };
        assert!(service.service.end().is_none(), "service waiting for its visitor");
        let current = digests(&mut world);
        assert!(current.machine.is_some(), "replay includes the disk's state");
        if let Some(previous) = previous {
            assert_eq!(previous, current, "same seed, start and inputs replay");
        }
        previous = Some(current);
    }
}
#[test]
fn keeping_links_leaves_them_for_the_harness() {
    let mut world = story::world(start(Links::Keep));
    idle(&mut world);
    assert!(world.printed().is_empty());
    let Process::Service(service) = world.process(0) else { panic!("service first") };
    assert!(service.service.links().pop().is_some());
    assert!(service.service.links().is_empty());
}

#[test]
fn story_a_lays_the_playground_forges_handover_before_starting() {
    use crate::{service::read_handover, stories::A};
    use skein_lib::Token;
    let world = story::world(A.start(7, Wall::EPOCH));
    let parameters = story::parameters(&world);
    let expected = crate::stories::admin_token(7);
    let read = temper_service::HandoverRead {
        forge: 1,
        run: Token::new(1),
        path: b"playground-1/admin_token".as_slice().into(),
        max: u32::try_from(expected.len()).expect("synthetic handover's length"),
    };
    let handover = read_handover(&mut parameters.disk.borrow_mut(), &read).expect("playground handover read");
    assert_eq!(handover.expect("present before the service starts").expose(), expected);
}

#[test]
fn handover_reads_its_path_and_bounds_the_file_before_reading() {
    use crate::{
        machine::{fresh, lay_handover},
        service::read_handover,
    };
    use skein_lib::Token;
    use temper_service::{HandoverRead, HandoverUnread};
    let mut disk = fresh();
    crate::machine::setup(&mut disk);
    let at = b"demo/custom/llm.key";
    lay_handover(&mut disk, at, b"scripted-key\n\n");
    let read = HandoverRead { forge: 0, run: Token::new(1), path: at.as_slice().into(), max: 14 };
    let before = disk.open_handles();
    let bytes = read_handover(&mut disk, &read).expect("read path").expect("file present");
    assert_eq!(bytes.expose(), b"scripted-key\n", "one final LF removed");
    assert_eq!(disk.open_handles(), before, "handover retains no handle");
    let too_small = HandoverRead { max: 13, ..read };
    assert_eq!(read_handover(&mut disk, &too_small), Err(HandoverUnread::TooLarge));
    let missing = HandoverRead { path: b"demo/custom/absent".as_slice().into(), ..too_small };
    assert_eq!(read_handover(&mut disk, &missing), Ok(None));
    let admin = HandoverRead { path: b"/setup/admin.token".as_slice().into(), max: 100, ..missing };
    assert_eq!(read_handover(&mut disk, &admin).expect("admin path").expect("present").expose(), fakes::ADMIN_TOKEN);
    assert_eq!(disk.open_handles(), before, "refusals retain no handle");
}

#[test]
fn story_a_start_makes_its_project() {
    // The native page's inputs through Try the playground, before its reply.
    let log = skein_ui_web::log::read(include_bytes!("fixtures/story-a-start.log")).expect("start's session log");
    let world = story::world(crate::stories::A.start(log.seed, log.start));
    let parameters = story::parameters(&world);
    let mut world =
        crate::programs::with(parameters, || skein_playground::replay::replay(world, story::reach(), &log, &mut ()))
            .unwrap_or_else(|differs| panic!("{differs:#?}"));
    for _ in 0..10_000 {
        story::step(&mut world);
        let Process::Service(service) = &world.processes()[0] else { panic!("service first") };
        let engine = service.service.engine_domain().expect("running engine");
        if let Some(playground) = engine.playground()
            && let Some(project) = playground.project
        {
            assert_eq!(project, 1, "the fresh story's project");
            assert_eq!(playground.account, Some(1), "the scripted account precedes its project");
            return;
        }
        if !world.busy()
            && let Some(due) = world.next_due()
        {
            world.advance(due);
        }
    }
    panic!("StartPlayground did not make its project");
}
