//! Remove runtime access and durably erase secrets (temper/docs/design/draft/playground-reset.md, 4).
use crate::engine::Engine;
use alloc::boxed::Box;
use jig_protocol_store::{SecretOwner, Secrets};
use skein_lib::Token;
use temper_protocol_records::{self as records, Item};

const PREFIX: u64 = crate::tokens::FORGE | (1_u64 << 59_u32);

pub(crate) fn owns(owner: Token) -> bool {
    owner.raw() & !0xffff_u64 == PREFIX
}

pub(crate) fn request(engine: &mut Engine, forge: u16) {
    match engine.forge.erasing.get(&forge) {
        Some(Some(_)) => return,
        Some(None) => {
            answered(engine, forge);
            return;
        }
        None => {}
    }
    engine.forge.down(skein_forge_connection::Request::Remove { forge });
    crate::setup::forget(engine, forge);
    if let Some(forge) = engine.workers.forges.remove(forge) {
        engine.values.identities.forget(forge.identity);
    }
    if let Some(number) = engine.values.tokens.forget(forge) {
        engine.values.identities.forget(number);
    }
    engine.forge.file_access.remove(&forge);
    engine.forge.startup_access.remove(&forge);
    engine.forge.endpoints.remove(&forge);
    engine.forge.held.remove(&forge);
    engine.forge.resolving.remove(&forge);
    engine.forge_records.rows.remove(forge);
    engine.forge.erasing.insert(forge, Some(Item::AdminToken)).expect("bounded removed forges");
    erase(engine, forge);
}

fn erase(engine: &mut Engine, forge: u16) {
    let owner = Token::new(PREFIX | u64::from(forge));
    let item = engine.forge.erasing.get(&forge).expect("active erase").expect("erase in flight");
    // Erase each item even when no durable row exists: setup may have a put in flight.
    let terminal = engine
        .store
        .secrets(owner, Secrets::Erase { owner: SecretOwner::Application, number: records::number(forge, item) });
    assert!(terminal.is_none(), "erase terminals are queued for the engine's store pass");
}

fn next(engine: &mut Engine, forge: u16) -> bool {
    let item = engine.forge.erasing.get_mut(&forge).expect("active erase");
    *item = Some(match item.expect("erase in flight") {
        Item::AdminToken => Item::ApiToken,
        Item::ApiToken => Item::GitToken,
        Item::GitToken => Item::User,
        Item::User => Item::WebhookSecret,
        Item::WebhookSecret => Item::AppId,
        Item::AppId => Item::AppSlug,
        Item::AppSlug => Item::AppKey,
        Item::AppKey => Item::Accepted,
        Item::Accepted => Item::Origin,
        Item::Origin => {
            *item = None;
            answered(engine, forge);
            return false;
        }
    });
    true
}

pub(crate) fn answer(engine: &mut Engine, owner: Token) {
    let forge = u16::try_from(owner.raw() & 0xffff).expect("forge number");
    if let Some(Some(_)) = engine.forge.erasing.get(&forge)
        && next(engine, forge)
    {
        erase(engine, forge);
    }
}

fn answered(engine: &mut Engine, forge: u16) {
    engine
        .events
        .push(temper_engine_domain::Event::Forge(Box::new(temper_engine_domain_forge::Event::SecretsErased { forge })));
}
