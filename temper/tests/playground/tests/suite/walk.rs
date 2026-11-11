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

fn doubled_byte_bounds(deployment: &[u8]) -> Box<[u8]> {
    fn double(overrides: &mut serde_json::Value) {
        for (name, value) in overrides.as_object_mut().expect("limit overrides") {
            // Startup requires checkout.name_bytes to agree with the agent's
            // name_bytes and the longest remote derived from forge.name_bytes,
            // and hub.event_bytes to fit tool_payload minus 2.
            if name == "checkout.name_bytes" || name == "forge.name_bytes" || name == "hub.event_bytes" {
                continue;
            }
            // These count payload alone: documents/small.rs clips UTF-8;
            // tasks/batch.rs counts spec text and contract words. The stock
            // templates also expose result_bytes as their report contract.
            if name == "documents.text_bytes" || name == "tasks.spec_bytes" || name == "tasks.result_bytes" {
                continue;
            }
            if name.ends_with("_bytes") {
                let bytes = value.as_u64().expect("a byte bound");
                *value = bytes.checked_mul(2).expect("doubled byte bound fits").into();
            }
        }
    }

    let mut document: serde_json::Value = serde_json::from_slice(deployment).expect("deployment document");
    for path in ["/engine/limits/overrides", "/application/limits/overrides"] {
        double(document.pointer_mut(path).expect("deployment limit overrides"));
    }
    for worker in document["workers"].as_array_mut().expect("deployment workers") {
        let overrides = worker.pointer_mut("/limits/overrides").expect("worker limit overrides");
        let output = overrides["io.output"]
            .as_u64()
            .unwrap_or(u64::from(jig_protocol_config::profile(jig_protocol_config::Profile::Small).io.output));
        overrides["io.output"] = output.checked_mul(2).expect("doubled output fits").into();
        double(overrides);
    }
    document["engine"]["limits"]["overrides"]["io.output"] =
        document["workers"][0]["limits"]["overrides"]["io.output"].clone();
    serde_json::to_vec(&document).expect("doubled deployment document").into_boxed_slice()
}

#[test]
fn story_a_walks_from_the_welcome_to_the_csv_change_landed() {
    let mut page = Page::start(&A, SEED, Wall::EPOCH);
    let script = stories::a();
    let steps = u32::try_from(script.len()).expect("a finite story");
    let mut person = Person::new(script.into_boxed_slice());
    let walked = page.walk(&mut person, BOUND);
    let bytes = page.log();
    let read = log::read(&bytes).expect("the page's session log");
    eprintln!(
        "story A: world {} ns, {} world steps, {} person steps, log {} bytes, {} inputs, pace 1x",
        page.world().now().as_nanos(),
        page.world().steps(),
        person.at(),
        bytes.len(),
        read.entries.len()
    );
    assert_eq!(walked, Walked::Done);
    assert_eq!(person.at(), steps, "every reply line and the landed status were seen");
    assert_main(page.world());
    assert_eq!(read.seed, SEED);
    assert_eq!(read.start, Wall::EPOCH);
    assert_eq!(read.build, None, "the native build is -");
    assert!(matches!(read.end, log::End::At { .. }), "the whole walk fits the log bound");
    if temper_test_env::rewrite() {
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
    for (row, deployment) in [("kept", A.deployment.into()), ("doubled", doubled_byte_bounds(A.deployment))] {
        let mut start = A.start(read.seed, read.start);
        start.deployment = deployment;
        let world = temper_playground::world(start);
        let parameters = story::parameters(&world);
        let replayed = programs::with(parameters, || {
            skein_playground::replay::replay(world, story::reach(), &read, &mut ())
                .unwrap_or_else(|differs| panic!("{row}: {differs:#?}"))
        });
        assert_main(&replayed);
    }
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
