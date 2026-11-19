//! A world's file gives a start's document; limits and seeds stay typed (configuration/file.md, 3.2 and 4).
use std::{net::SocketAddr, path::Path};
use temper_config_shell::{File, Read, Shape};

pub(crate) fn read(directory: &Path, shape: Shape) -> Read {
    let dirs = temper_agent_fixtures::directories(directory);
    let facts = temper_config_shell::facts(&dirs).expect("fixture facts");
    temper_config_shell::parse(
        &std::fs::read(directory.join(temper_config_shell::FILE)).expect("configuration file"),
        shape,
        &facts,
    )
    .expect("fixture file")
}

pub(crate) fn document(directory: &Path, shape: Shape) -> serde_json::Value {
    serde_json::from_slice(&read(directory, shape).document).expect("file's start")
}

pub(crate) fn write(directory: &Path, file: &File, shape: Shape) -> std::io::Result<()> {
    super::private(&directory.join(temper_config_shell::FILE), &temper_config_shell::render(file, shape))
}

pub(crate) fn standalone(value: &serde_json::Value) -> File {
    let socket = |value: &serde_json::Value| -> Option<SocketAddr> {
        value.as_str().map(|text| text.parse().expect("fixture socket"))
    };
    let github = &value["application"]["test"]["github"];
    File {
        listen: socket(&value["application"]["clients"]["listener"]["socket"]),
        webhooks: Some(
            socket(&value["application"]["webhooks"]["listener"]["socket"]).unwrap_or(([127, 0, 0, 1], 0).into()),
        ),
        owner: Some("owner-a@localhost".into()),
        test: temper_config_shell::Test {
            github: github.as_object().map(|_| temper_config_shell::TestGitHub {
                api: github["api"]["socket"].as_str().expect("fake GitHub socket").into(),
                transport: temper_config_shell::Transport::Plaintext,
                remote: github["remote"].as_str().expect("fake remote").into(),
                web: github["web"].as_str().expect("fake web").into(),
                git: github["git"].as_str().map(Into::into),
            }),
            ..Default::default()
        },
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_rebound_world_keeps_its_file_and_typed_inputs_separate() {
        let scratch = skein_scratch::Scratch::new("real-loop-file");
        let initial = "127.0.0.1:42001".parse().expect("initial");
        let mut configured = crate::fixtures::write(scratch.path(), initial, initial).expect("world files");
        let limits = (configured.jig, configured.limits);
        let llm = "127.0.0.1:42002".parse().expect("LLM");
        let forge = "127.0.0.1:42003".parse().expect("forge");
        crate::fixtures::bind(scratch.path(), llm, forge, &mut configured).expect("rebound world");
        assert!(!scratch.path().join(temper_config_shell::FORMER).exists());
        let read = read(scratch.path(), Shape::Standalone);
        assert_eq!(read.file.listen, Some("127.0.0.1:0".parse().expect("listen")));
        let start: serde_json::Value = serde_json::from_slice(&read.document).expect("start");
        assert!(start["engine"].get("projects").is_none());
        assert!(start.get("world_settings").is_none());
        assert_eq!((configured.jig, configured.limits), limits);
        let flags: serde_json::Value =
            serde_json::from_slice(&std::fs::read(scratch.path().join("world-settings.json")).expect("world settings"))
                .expect("settings");
        assert_eq!(flags["forge"], forge.to_string());
        assert_eq!(flags["llm"]["endpoints"][0]["address"]["socket"], llm.to_string());
    }
}
