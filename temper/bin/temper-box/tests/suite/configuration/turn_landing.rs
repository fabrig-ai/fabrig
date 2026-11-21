//! Turn-landing startup admission in real directories (testing/qa/setups.md, 5–6).
use temper_box_kit::turn_landing_fixture::{figures, rendered};
use temper_service::startup::{self, Shape};

#[test]
fn the_turn_landing_engine_passes_every_startup_check() {
    let bytes = rendered("engine.toml");
    let directories = jig_protocol_config::Directories::working();
    let named = startup::named(&bytes, &directories).expect("named setup files");
    admission("turn-landing-engine-admission", &bytes, named.files);
}

#[test]
fn the_turn_landing_worker_passes_every_startup_check() {
    let bytes = rendered("worker.toml");
    let directories = jig_protocol_config::Directories::working();
    let part = startup::worker_part(&bytes, &directories, Some(b"/turn/data/bin/git"), &[]).expect("setup worker part");
    admission("turn-landing-worker-admission", &bytes, part.files);
}

fn admission(case: &str, bytes: &[u8], files: Box<[jig_protocol_config::File]>) {
    use skein_lib::{List, Secret};
    use temper_service::settings::{Files, Seeds};

    let directories = jig_protocol_config::Directories::working();
    let mut entries = List::with_capacity(u32::try_from(files.len()).expect("file count"));
    for file in files {
        entries.push((file.path, Secret::new(Box::new([b'7'; 64])))).expect("named file count");
    }
    let scratch = crate::support::Scratch::startup(case);
    let configuration = temper_worker_shell::agent::figures::configuration(&figures(), None, scratch.path(), &[])
        .expect("agent document from the engine's figures");
    let agent = temper_worker_shell::agent::prepare_bytes(configuration).expect("setup agent").agent;
    let seeds = Seeds {
        link_secret: None,
        local_secret: None,
        deployment: [7; 16],
        installation: [8; 16],
        seed: 7,
        crypto: Secret::new(Box::new([9; 32])),
    };
    let files = Files { entries };
    if startup::shape(bytes, &directories).expect("setup shape") == Shape::WorkerAlone {
        let figures = figures();
        let configuration = temper_worker_shell::agent::figures::configuration(&figures, None, scratch.path(), &[])
            .expect("agent document from the engine's figures");
        let agent = temper_worker_shell::agent::prepare_bytes(configuration).expect("worker's agent").agent;
        let mut part =
            startup::worker_part(bytes, &directories, Some(b"/turn/data/bin/git"), &[]).expect("worker part");
        temper_worker_service::figures::apply(&mut part, figures).expect("engine's figures applied");
        if let Err(problems) = temper_worker_service::startup::worker::check_worker_part(&mut part, &files, &agent) {
            panic!("{}", String::from_utf8_lossy(&temper_worker_service::startup::lines(&problems)));
        }
    } else if let Err(problems) =
        startup::check(bytes, &directories, &files, &agent, &seeds, None, List::with_capacity(0), None)
    {
        panic!("{}", String::from_utf8_lossy(&startup::lines(&problems)));
    }
    scratch.remove();
}
