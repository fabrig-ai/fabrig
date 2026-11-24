use super::*;
use temper_config_shell::{Facts, File, Shape};

pub(crate) fn fixture() -> (Directories, Read, service::settings::Seeds, service::settings::Files) {
    let file = File { workers: Some("127.0.0.1:7200".parse().unwrap()), ..File::default() };
    let config = temper_config_shell::render(&file, Shape::EngineAlone);
    let facts = Facts { login: "fixture".into(), hostname: "fixture".into(), provider_key: false };
    let flags = ["--config", "/config", "--data", "/data", "--cache", "/cache"].map(|word| word.as_bytes().into());
    let (directories, _) = temper_shell::directories::resolve(&flags, &[], b"/").unwrap();
    let seeds = service::settings::Seeds {
        deployment: [7; 16],
        installation: [8; 16],
        seed: 7,
        crypto: skein_lib::Secret::new(Box::new([9; 32])),
        link_secret: None,
        local_secret: None,
    };
    let files = service::settings::Files { entries: skein_lib::List::with_capacity(0) };
    let read = temper_config_shell::parse(&config, Shape::EngineAlone, &facts).expect("stopped configuration");
    (directories, read.into(), seeds, files)
}

#[test]
fn stopped_preparation_names_bad_held_lines_without_opening_or_changing_the_store() {
    for (body, prefix) in [
        ("not.a.limit 1\n", "limits-held line 1: not.a.limit is no limit of this release"),
        ("people.identity_bytes 4294967296\n", "limits-held line 1: people.identity_bytes (setting limits):"),
    ] {
        let (directories, read, seeds, files) = fixture();
        let mut store_effects = Vec::new();
        let why = prepare_with(
            &directories,
            Ok(read),
            |path| {
                assert_eq!(path, Path::new("/data/limits-held"));
                Ok(body.into())
            },
            |bytes| {
                Ok((
                    LimitInputs::new(
                        bytes,
                        &jig_directories(&directories),
                        &files,
                        None,
                        &seeds,
                        &service::stock::Given::Stock,
                        false,
                        None,
                    ),
                    (),
                ))
            },
            |_, _, _, _, ()| {
                store_effects.extend(["open", "adopt"]);
                Ok(())
            },
        )
        .expect_err("stopped preparation refuses the held figures");
        assert!(why.starts_with(prefix), "{why}");
        assert!(!why.contains("put limits-held"), "the held-file refusal names its own line: {why}");
        assert!(store_effects.is_empty(), "held-file refusal opens and adopts no store: {store_effects:?}");
    }
}
