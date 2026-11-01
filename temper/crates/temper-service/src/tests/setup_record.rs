//! The machine's owned record and deployment offer (temper/docs/plan/playground-reset-r4/notes/contracts.md, 5 and 6).
#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "ordinary tests read and edit document fixtures"
)]
use super::fixtures;
use crate::{family, startup::owned_record};
use jig_protocol_config as config;
use skein_lib::List;
use temper_client_protocol_wire::{self as wire, v1 as w};
use temper_engine_domain_forge as forge;
use temper_forge_setup::setup::{self as s, Template};

fn entry() -> s::Entry {
    let document = include_bytes!("../../../temper-init/template/temper.json");
    let setup = fixtures::value(document, b"application.setup[1]");
    let setup = core::str::from_utf8(&setup)
        .unwrap()
        .replace("$owned", "127.0.0.1:8368")
        .replace("$login", "person")
        .replace("$email", "person@localhost");
    let setup = format!("[{setup}]");
    let document =
        config::document(setup.as_bytes(), &config::DocumentLimits { bytes: 16384, tokens: 4096, depth: 32 }).unwrap();
    let mut problems = temper_forge_setup::problem::Problems::empty();
    let setup = s::Setup::read(
        &document,
        0,
        &fixtures::engine().limits.setup.plan(),
        &config::Directories::given(b"/config", b"/data", b"/cache").unwrap(),
        &mut problems,
    )
    .unwrap();
    assert!(problems.is_empty());
    setup.entries[0].clone()
}
fn template() -> Template {
    let entry = entry();
    Template { directory: entry.directory, bot: entry.bot, orgs: entry.orgs, mode: entry.mode }
}
fn service(template: Option<Template>, seeded: bool, under_box: bool) -> crate::Service {
    let mut settings = fixtures::engine();
    fixtures::fill_files(&mut settings);
    settings.forges = Box::new([]);
    settings.identities = Box::new([]);
    settings.setup = Box::new([]);
    settings.seed = if seeded {
        let mut record = owned_record::record(&template.clone().unwrap()).unwrap();
        record.number = 3;
        record.identity.number = 3;
        Box::new([record])
    } else {
        Box::new([])
    };
    settings.owned = template;
    let io = settings.jig.limits.io;
    crate::Service::prepared(
        settings,
        &fixtures::seeds(),
        None,
        skein_forge_connection::Settings {
            forges: List::with_capacity(0),
            webhooks: None,
            word: b"temper".as_slice().into(),
            user_agent: b"temper".as_slice().into(),
        },
        None,
        io,
        if under_box { Some(128) } else { None },
        fixtures::shell(),
    )
    .unwrap()
}
fn offer(service: &crate::Service) -> w::Offer {
    let forges = service.engine.as_ref().unwrap().domain.forges();
    let bytes = family::forges(forges, &w::CEILINGS).unwrap();
    wire::decode_forges(&bytes, &w::CEILINGS).unwrap().owned().clone()
}
#[test]
fn the_template_record_reads_as_forge_three_did() {
    let template = template();
    let record = owned_record::record(&template).unwrap();
    assert!(record.fits());
    assert_eq!(record.number, 0);
    assert_eq!(record.identity.number, 0);
    assert_eq!(record.api, forge::record::Address::Socket(([127, 0, 0, 1], 8368).into()));
    assert_eq!(record.transport, forge::record::Transport::Plaintext);
    assert_eq!(record.git.as_ref(), b"http://127.0.0.1:8368/");
    assert_eq!(record.identity.login.as_ref(), b"temper");
    assert_eq!(record.identity.name.as_ref(), b"Temper");
    assert_eq!(record.identity.email.as_ref(), b"temper@localhost");
    assert!(record.files.is_none() && record.web.is_none() && !record.playground);
    let mut numbered = record;
    numbered.number = 3;
    assert_eq!(crate::forge::table::setup_entry(&numbered), Some(entry()));
}
#[test]
fn forges_say_set_up_is_offered_with_a_template_and_no_owned_row() {
    for (template, under_box, expected) in [
        (Some(template()), true, w::Offer::Offered),
        (Some(template()), false, w::Offer::NotUnderTheBox),
        (None, true, w::Offer::NotOffered),
        (None, false, w::Offer::NotOffered),
    ] {
        let service = service(template, false, under_box);
        assert!(service.engine.as_ref().unwrap().domain.forges().rows.is_empty());
        assert_eq!(offer(&service), expected);
    }
}
#[test]
fn forges_say_not_offered_once_an_owned_row_is_listed() {
    let service = service(Some(template()), true, true);
    assert_eq!(service.engine.as_ref().unwrap().domain.forges().rows.len(), 1);
    assert_eq!(offer(&service), w::Offer::NotOffered);
}
#[test]
fn an_owned_record_past_the_store_bound_is_refused_at_its_member() {
    let mut template = template();
    template.directory = skein_lib::bytes::zeroed(forge::record::PATH_BYTES + 1);
    let problems = owned_record::record(&template).unwrap_err();
    assert_eq!(problems.problems.get(0).unwrap().path, config::Path::root().member(b"application").member(b"owned"));
    assert_eq!(problems.problems.get(0).unwrap().what, config::What::Bound);
}

#[test]
fn an_owned_record_at_the_store_bound_is_accepted() {
    let mut template = template();
    template.directory = skein_lib::bytes::zeroed(forge::record::PATH_BYTES);
    assert_eq!(owned_record::record(&template).unwrap().setup.unwrap().directory.len(), forge::record::PATH_BYTES);
}
