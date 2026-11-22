//! Catch-up progress redelivers both documents through the real core.
use jig_domain_core::documents::{Watched, Window};
use jig_domain_people::ApplicationScope;
use skein_lib::Token;
use temper_engine_domain::{Event, PartyInput};
use temper_engine_domain_forge::{CatchingUp, Document, changes_subject};
use temper_engine_domain_world::forge_routes::{World, change_world};

const FORGES: Token = Token::new(810);
const CHANGES: Token = Token::new(811);

#[test]
fn marked_forge_progress_redelivers_both_watches_when_a_count_falls_and_when_it_ends() {
    let (mut world, _, _, _) = change_world(false, 1000, None);
    world.oracle_settle();
    world.held_reads = Some(Vec::new());
    world.hold_reads_at_any_step = true;
    world.restart_same();
    for _ in 0..200 {
        world.tick();
        if world.root.ready() && world.held_reads.as_ref().is_some_and(|reads| !reads.is_empty()) {
            break;
        }
    }
    world.sign_in();
    watch(&mut world, FORGES, ApplicationScope::Deployment, 0);
    watch(&mut world, CHANGES, ApplicationScope::Project(1), changes_subject(1));
    for _ in 0..200 {
        world.tick();
        if !world.forges_documents.is_empty() && !world.oracle_documents.is_empty() {
            break;
        }
    }
    let initial = states(&world);
    assert!(initial.0.is_some(), "forge is still catching up");
    assert_eq!(initial.0, initial.1, "both documents carry the same progress");
    for read in world.held_reads.take().expect("held fresh reads") {
        world.out.push(read);
    }
    let mut changed = (false, false);
    let mut ended = false;
    for _ in 0..200 {
        for watcher in [FORGES, CHANGES] {
            world.send(Event::Party(PartyInput::Delivered { watcher }));
        }
        world.tick();
        let state = states(&world);
        changed.0 |= state.0.is_some() && state.0 != initial.0;
        changed.1 |= state.1.is_some() && state.1 != initial.1;
        ended = state == (None, None);
        if ended {
            break;
        }
    }
    assert_eq!(changed, (true, true), "both watches deliver a falling count before the end");
    assert!(ended, "both watches remove the line once caught up");
    world.close();
}

fn watch(world: &mut World, watcher: Token, scope: ApplicationScope, subject: u64) {
    world.send(Event::Party(PartyInput::Watch {
        watcher,
        sign_in: world.signed_in.expect("signed in"),
        subject: Watched::Application { scope, connector: 0, subject: Token::new(subject) },
        window: Window { rows: 1, after: None },
    }));
}

fn states(world: &World) -> (Option<CatchingUp>, Option<CatchingUp>) {
    let forges = world.forges_documents.iter().rev().find_map(|(watcher, rows)| {
        (*watcher == FORGES).then(|| rows.rows.first().expect("configured forge").catching_up)
    });
    let changes = world.oracle_documents.iter().rev().find_map(|(watcher, document)| {
        if *watcher == CHANGES
            && let Document::Changes(panel) = document
        {
            Some(panel.catching_up)
        } else {
            None
        }
    });
    (forges.expect("forge watch delivered"), changes.expect("changes watch delivered"))
}
