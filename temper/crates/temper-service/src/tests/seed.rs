//! First-start document records (contracts, D1).
#![expect(clippy::disallowed_methods, clippy::disallowed_macros, reason = "ordinary tests edit document fixtures")]
use crate::tests::fixtures;
use crate::{
    settings,
    startup::{self, Problems, application, seed},
};
use jig_protocol_config::{self as config, Path};
use skein_lib::List;
use temper_engine_domain_forge::{Configured, Mode, record as r};

fn parsed(input: &[u8], files: &settings::Files) -> Result<settings::Engine, Problems> {
    let d = config::document(input, &config::DocumentLimits { bytes: 1 << 20, tokens: 16_384, depth: 32 }).unwrap();
    let directories = config::Directories::working();
    let mut root = config::read::Object::new(&d, 0).unwrap();
    let jig = startup::engine(&d, root.member(&d, b"engine").unwrap(), &directories).unwrap();
    let at = root.member(&d, b"application").unwrap();
    let mut problems = Problems::empty();
    let members = application::parse(&d, at, &jig, &directories, &mut problems);
    if !problems.is_empty() {
        return Err(problems);
    }
    let members = members.unwrap();
    let resolved = application::resolve(&members, files, &mut problems).unwrap();
    assert!(problems.is_empty(), "{problems:?}");
    let mut application = config::read::Object::new(&d, at).unwrap();
    let mut setup = List::with_capacity(members.limits.forge.forges);
    if let Some(at) = application.member(&d, b"setup") {
        let mut found = temper_forge_setup::problem::Problems::empty();
        let entries =
            temper_forge_setup::setup::Setup::read(&d, at, &members.limits.setup.plan(), &directories, &mut found)
                .unwrap();
        assert!(found.is_empty());
        for entry in entries.entries {
            setup.push(settings::Setup { entry, handover: None }).unwrap();
        }
    }
    let mut settings = settings::Engine {
        jig,
        forges: resolved.forges,
        identities: resolved.identities,
        setup: setup.into_boxed(),
        demo: members.demo,
        limits: members.limits,
        ..fixtures::engine()
    };
    application::git_programs(
        &mut settings.forges,
        &temper_forge_setup::setup::Setup { entries: settings.setup.iter().map(|setup| setup.entry.clone()).collect() },
        &mut problems,
    );
    settings.seed = seed::seed(&members.forges, &members.identities, &settings)?;
    Ok(settings)
}
fn template() -> Box<[u8]> {
    let input = include_str!("../../../temper-init/template/temper.json");
    let input = input.replace("$forge_git", "http://127.0.0.1:8369").replace("$owned_git", "http://127.0.0.1:8368");
    let input = input
        .replace("$forge", "127.0.0.1:8369")
        .replace("$owned", "127.0.0.1:8368")
        .replace("$listener", "127.0.0.1:8367")
        .replace("$webhooks", "127.0.0.1:8371")
        .replace("$llm", "127.0.0.1:8370")
        .replace("$binary", "/bin/temper")
        .replace("$agent", "/config/agent.json")
        .replace("$git", "/bin/git")
        .replace("$login", "person")
        .replace("$email", "person@localhost");
    input.into_bytes().into_boxed_slice()
}
#[test]
fn the_template_seeds_its_three_forges() {
    let settings = parsed(&template(), &settings::Files { entries: List::with_capacity(0) }).unwrap();
    let [fake, github, owned] = settings.seed.as_ref() else { panic!("three template records") };
    assert_eq!((fake.number, github.number, owned.number), (1, 2, 3));
    assert!(fake.playground);
    assert_eq!(Configured::from_record(fake).mode, Mode::Fake);
    assert_eq!(github.kind, temper_engine_domain_forge::Kind::GitHub);
    assert!(github.files.is_none());
    assert!(github.identity.login.is_empty() && github.identity.name.is_empty() && github.identity.email.is_empty());
    assert_eq!(Configured::from_record(owned).mode, Mode::Owned);
    assert!(Configured::from_record(owned).asked);
    assert!(owned.setup.as_ref().unwrap().asked);
    for row in &settings.seed {
        assert!(row.first.is_none() && !row.removing && row.fits());
    }
}
#[test]
fn a_file_form_forge_keeps_its_file_names() {
    let settings = parsed(&fixtures::document(), &fixtures::files()).unwrap();
    let files = settings.seed[0].files.as_ref().unwrap();
    assert_eq!(files.api_token.as_ref(), b"secrets/forge-1.api-token");
    assert_eq!(files.git_token.as_deref(), Some(b"secrets/forge-1.git-token".as_slice()));
    assert_eq!(files.user, r::FileUser::File(b"secrets/forge-1.user".as_slice().into()));
    assert_eq!(settings.seed[0].identity.login.as_ref(), b"temper");
}
#[test]
fn a_document_naming_no_forge_starts_with_an_empty_seed() {
    let input = fixtures::with(&fixtures::document(), b"application.forges", b"[]");
    let input = fixtures::with(&input, b"application.identities", b"[]");
    for absent in [false, true] {
        let input = if absent {
            core::str::from_utf8(&input)
                .unwrap()
                .replace("\"forges\":[],", "")
                .replace("\"identities\":[],", "")
                .replace("\"setup\":[],", "")
                .into_bytes()
                .into_boxed_slice()
        } else {
            input.clone()
        };
        assert!(parsed(&input, &fixtures::files()).unwrap().seed.is_empty());
        let service = crate::startup(
            &input,
            &config::Directories::working(),
            &fixtures::files(),
            &fixtures::startup_agent(),
            &fixtures::seeds(),
            None,
            fixtures::worker_runtime(),
            List::with_capacity(0),
            crate::forge::table::Shell::refusing(),
        )
        .unwrap();
        assert!(service.engine.as_ref().unwrap().domain.forges().rows.is_empty());
    }
}
#[test]
fn a_seed_past_its_bounds_is_refused_at_its_path() {
    let long = "a".repeat(256);
    let input = fixtures::with(
        &fixtures::document(),
        b"application.identities[0].author.name",
        format!("\"{long}\"").as_bytes(),
    );
    let problems = parsed(&input, &fixtures::files()).unwrap_err();
    assert_eq!(problems.problems.len(), 1);
    assert_eq!(
        problems.problems.get(0).unwrap().path,
        Path::root().member(b"application").member(b"forges").child(config::Segment::Index(0))
    );
    assert_eq!(problems.problems.get(0).unwrap().what, startup::What::Jig(config::What::Bound));
}
