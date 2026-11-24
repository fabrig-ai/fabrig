//! Read a managed forge's landed change across a crash (temper/docs/design/testing/tiers.md, 1.5).
use super::{MERGE_PULL, PLAN_REQUEST, see};
use crate::{
    fixtures,
    referee::client::Landing,
    world::{OneProcess, Proc, Story},
};
use skein_fake_forge_domain::Observation;
use skein_fake_person::{Find, Step, Target};
use skein_ui::Role;
use skein_world::{Inherited, StartupRoot, World};
use std::cell::{Cell, RefCell};
use temper_client_domain::Address;
use temper_playground::{fakes, machine::Machine, programs};

// The adoption fixture's planning chat delegates one change.
const PLAN: u64 = 1;
const CHANGE: u64 = 2;

thread_local! {
    static MERGE_GRANTED: Cell<bool> = const { Cell::new(false) };
    static FORGE_IMAGE: RefCell<Option<Box<[u8]>>> = const { RefCell::new(None) };
}

pub(crate) fn with_merge<R>(run: impl FnOnce() -> R) -> R {
    struct Restore(bool);
    impl Drop for Restore {
        fn drop(&mut self) {
            MERGE_GRANTED.set(self.0);
        }
    }
    let _restore = Restore(MERGE_GRANTED.replace(true));
    run()
}

pub(crate) fn merge_granted() -> bool {
    MERGE_GRANTED.get()
}

pub(crate) fn person(after_crash: bool) -> Vec<Step<Address>> {
    let mut steps = if after_crash {
        vec![Step::Go { address: Address::Chat { task: PLAN } }, see(Find::text(PLAN_REQUEST))]
    } else {
        let mut steps = super::person();
        steps.push(see(Find::exactly_named(Role::Link, b"Task 2")));
        steps
    };
    steps.extend([
        Step::Go { address: Address::Task { task: CHANGE } },
        see(Find::exactly_named(Role::Status, b"Completed")
            .within(Box::new([Target::Role { role: Role::Region, name: b"Task details".as_slice().into() }]))),
        see(Find::text("Landed · merge commit ".as_bytes())),
    ]);
    steps
}

pub(crate) fn grant_merge(document: &mut temper_playground::documents::Configuration) {
    let grant = serde_json::json!({
        "connector": fixtures::FORGE_CONNECTOR, "kind": MERGE_PULL,
        "pattern": {"segments": ["forge", document.forge.to_string()], "last": "open", "value": "acme"}
    });
    let policy = &mut document["world_settings"]["policies"][0]["policy"];
    policy["ceiling"]["grants"].as_array_mut().expect("policy grants").push(grant.clone());
    policy["roles"][0]["authority"]["grants"].as_array_mut().expect("owner grants").push(grant);
}

pub(crate) fn spawn<R: skein_world::Referee<Proc>>(world: &mut World<Proc, R, Machine>) {
    FORGE_IMAGE.with_borrow_mut(|image| *image = None);
    world.spawn_restartable(
        vec![StartupRoot { name: b"private".as_slice().into(), path: b"client".as_slice().into() }],
        crate::world::landing_person,
    );
    world.spawn_restartable(vec![], provider);
    world.spawn_restartable(vec![], forge_peer);
}

fn provider(inherited: &Inherited) -> Proc {
    let peer = super::provider(Story::Adopt);
    programs::llm_peer::<OneProcess>(peer, inherited)
}

fn forge_peer(inherited: &Inherited) -> Proc {
    let mut peer = fakes::forge_peer(&programs::parameters().forge, fixtures::SEED);
    if let Some(image) = FORGE_IMAGE.with_borrow_mut(Option::take) {
        peer.restore(&image).expect("restore the retained forge");
    }
    programs::fake_forge::<OneProcess>(peer, inherited)
}

pub(crate) fn keep_forge(other: &[Proc]) -> Result<(), String> {
    let peer = other.iter().find_map(|proc| if let Proc::Forge(peer) = proc { Some(peer) } else { None });
    let image =
        peer.ok_or("the restartable forge peer")?.image().map_err(|why| format!("forge at the cut: {why:?}"))?;
    FORGE_IMAGE.with_borrow_mut(|slot| *slot = Some(image));
    Ok(())
}

pub(crate) fn check_landing(landing: Landing, ledger: &[Observation]) -> Result<(), String> {
    let mut merges = ledger.iter().filter(|row| matches!(row, Observation::Merged { .. }));
    if let (Some(Observation::Merged { number, commit, .. }), None) = (merges.next(), merges.next())
        && landing.task == CHANGE
        && landing.pull == *number
        && landing.merge == skein_fake_forge_forgejo::routes::object(*commit).bytes
    {
        Ok(())
    } else {
        Err(format!("the person must read the one merged pull and commit: {landing:?}"))
    }
}

pub(crate) fn no_writes(ledger: &[Observation]) -> Result<(), String> {
    if let Some(write) = ledger.iter().find(|row| matches!(row, Observation::Moved { .. } | Observation::Merged { .. }))
    {
        return Err(format!("a recovered landing must not push or merge again: {write:?}"));
    }
    Ok(())
}
