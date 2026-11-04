use super::*;

#[test]
fn trace_admission_resolves_existing_files_absent_files_and_aliases_before_opening() {
    let scratch = smith_real_world::Scratch::new();
    let chats = scratch.path().join("chats");
    let chat = chats.join("old");
    std::fs::create_dir_all(&chat).unwrap();
    let state = chat.join("state");
    std::fs::write(&state, b"older bytes").unwrap();
    let outside = scratch.path().join("outside");
    trace_outside(&outside, &chats).unwrap_or_else(|why| panic!("{}", why.line()));
    assert!(!outside.exists(), "admission does not open an ordinary outside destination");
    let alias = scratch.path().join("alias");
    std::os::unix::fs::symlink(&chat, &alias).unwrap();
    let link = scratch.path().join("linked-file");
    std::os::unix::fs::symlink(&state, &link).unwrap();
    for path in [&state, &chat.join("trace"), &alias.join("state"), &alias.join("trace"), &link] {
        let Err(refused) = trace_outside(path, &chats) else {
            panic!("trace inside chats must refuse");
        };
        assert_eq!(refused.code(), 2);
        assert!(refused.line().contains("inside this workspace's chats tree"));
        assert_eq!(std::fs::read(&state).unwrap(), b"older bytes");
        assert!(!chat.join("trace").exists());
    }
}

#[test]
fn configured_local_and_generated_child_traces_are_admitted_before_any_open() {
    let scratch = smith_real_world::Scratch::new();
    let workspace = scratch.path().canonicalize().unwrap();
    let paths = settings::Paths { state_home: Some(scratch.path().join("state").into()), ..settings::Paths::default() };
    let tree = chats_tree(&workspace, &paths).unwrap_or_else(|why| panic!("{}", why.line()));
    let chat = tree.join("old");
    std::fs::create_dir_all(&chat).unwrap();
    std::fs::write(chat.join("state"), b"old state").unwrap();
    let alias = scratch.path().join("alias");
    std::os::unix::fs::symlink(&chat, &alias).unwrap();
    let certificate = crate::fixture::trust().display().to_string();
    for (path, allowed) in
        [(scratch.path().join("outside"), true), (chat.join("state"), false), (alias.join("trace"), false)]
    {
        let source = settings::Source {
            layer: settings::Layer::User,
            path: "trace-admission.toml".into(),
            text: format!(
                "model='openai/gpt-6-luna'\ntrace.path='{}'\n[accounts.openai]\nprovider='openai'\nsource='env'\nvariable='SMITH_FIXTURE_TOKEN'\n[endpoints.openai]\naddress='127.0.0.1:443'\ntrust.system=false\ntrust.der=['{certificate}']\n",
                path.display()
            ),
        };
        let mut settings = crate::fixture::ready(settings::from_sources(
            &crate::command::Shared::default(),
            workspace.clone(),
            &[source],
        ));
        settings.paths = paths.clone();
        let (resolved, document) = crate::fixture::ready(settings::resolve::resolve_startup(&settings));
        let document: serde_json::Value = serde_json::from_slice(&document).unwrap();
        assert_eq!(document["trace"]["path"].as_str(), path.to_str());
        assert_eq!(trace_admission(&resolved, &workspace).is_ok(), allowed);
        assert_eq!(std::fs::read(chat.join("state")).unwrap(), b"old state");
        assert!(!chat.join("trace").exists());
        assert!(!scratch.path().join("outside").exists());
    }
}
