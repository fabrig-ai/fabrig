//! Story A through the web host, with its kept log replayed (temper/docs/design/playground.md, 9).

use skein_fake_forge_domain::api;
use skein_fake_person::Person;
use skein_lib::{Duration, Wall};
use skein_ui_web::log;
use temper_playground::{documents, programs, stories::A, story};
use temper_playground_world::{
    page::{Page, Walked},
    stories,
};

const SEED: u64 = 7;
const BOUND: Duration = Duration::from_secs(120);
const REPOSITORY: &[u8] = b"playground/widget";

fn kept_log() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/story-a.log")
}

#[test]
fn story_a_walks_from_the_welcome_to_the_csv_change_landed() {
    let mut page = Page::start(&A, SEED, Wall::EPOCH);
    let script = stories::a();
    let steps = u32::try_from(script.len()).expect("a finite story");
    let mut person = Person::new(script.into_boxed_slice());
    assert_eq!(page.walk(&mut person, BOUND), Walked::Done);
    assert_eq!(person.at(), steps, "every reply line and the landed status were seen");
    assert_main(page.world());

    let bytes = page.log();
    let read = log::read(&bytes).expect("the page's session log");
    assert_eq!(read.seed, SEED);
    assert_eq!(read.start, Wall::EPOCH);
    assert_eq!(read.build, None, "the native build is -");
    assert!(matches!(read.end, log::End::At { .. }), "the whole walk fits the log bound");
    eprintln!(
        "story A: world {} ns, {} world steps, {} person steps, log {} bytes, {} inputs, pace 1x",
        page.world().now().as_nanos(),
        page.world().steps(),
        person.at(),
        bytes.len(),
        read.entries.len()
    );
    if std::env::var_os("SKEIN_CODEGEN_WRITE").is_some() {
        std::fs::create_dir_all(kept_log().parent().expect("fixtures directory")).expect("create fixtures directory");
        std::fs::write(kept_log(), &bytes).expect("write story A's kept log");
    }
    let kept = std::fs::read(kept_log()).expect("story A's kept log");
    assert_eq!(bytes.as_ref(), kept, "the native walk keeps the same session log byte for byte");
}

#[test]
fn story_a_kept_log_replays_natively() {
    let bytes = std::fs::read(kept_log()).expect("story A's kept log");
    let read = log::read(&bytes).expect("the kept session log");
    let world = temper_playground::world(A.start(read.seed, read.start));
    let parameters = story::parameters(&world);
    let replayed =
        programs::with(parameters, || skein_playground::replay::replay(world, story::reach(), &read, &mut ()))
            .unwrap_or_else(|differs| panic!("{differs:#?}"));
    assert_main(&replayed);
}

#[test]
fn the_visitors_exchanges_are_the_pages_fetch_bound() {
    assert_eq!(documents::VISITOR_EXCHANGES, temper_client_shell_web::limits::EXCHANGES + 1);
}

fn assert_main(world: &temper_playground_world::Playground) {
    let transport = story::parameters(world).forge;
    let forge = transport.forge.borrow();
    let api::Answer::Commit(main) = forge
        .domain
        .inspect(&forge.config, REPOSITORY, &api::Read::Branch { branch: b"main".as_slice().into() })
        .expect("widget's main")
    else {
        panic!("branch commit")
    };
    let object = forge.domain.object(main).expect("main's object");
    let scripts: serde_json::Value = serde_json::from_slice(A.llm).expect("story A's scripts");
    let edited = scripts["cued"][0]["turns"][1]["lines"][0]["arguments"]["edits"][0]["new"].as_str().expect("CSV edit");
    assert!(
        object.tree.get(b"src/main.rs".as_slice()).is_some_and(|content| skein_lib::bytes::find(
            content,
            edited.as_bytes()
        )
        .is_some()),
        "main holds the scripted CSV edit"
    );
    let api::Answer::Checks(checks) =
        forge.domain.inspect(&forge.config, REPOSITORY, &api::Read::Checks { commit: main }).expect("main's CI")
    else {
        panic!("checks")
    };
    assert!(
        !checks.is_empty() && checks.iter().all(|check| check.status.state == api::Check::Passed),
        "every check passed at main's head"
    );
}
