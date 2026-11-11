use super::prepare_hint;
use temper_shell::directories;

#[test]
fn prepare_hint_names_the_running_binary() {
    let arguments =
        ["--config", "/config", "--data", "/data", "--cache", "/cache"].map(|word| Box::from(word.as_bytes()));
    let (directories, _) = directories::resolve(&arguments, &[], b"/").expect("explicit directories");
    for (binary, expected) in [
        ("temper", "not prepared: run temper forge prepare --config /config --data /data --cache /cache 17"),
        (
            "temper-engine",
            "not prepared: run temper-engine forge prepare --config /config --data /data --cache /cache 17",
        ),
    ] {
        assert_eq!(prepare_hint(binary, "not prepared", 17, &directories), expected, "{binary}");
    }
}

#[test]
fn inherited_listener_uses_the_given_descriptor_count() {
    use temper_forge_setup::spec::Which;
    let environment = [(Box::from(b"LISTEN_FDS".as_slice()), Box::from(b"2".as_slice()))];
    assert_eq!(
        super::inherited_listener(Which::Forgejo, &environment),
        Err("LISTEN_FDS: forgejo requires exactly 1 listener at fd 3".into())
    );
    assert_eq!(
        super::inherited_listener(Which::Runner, &environment),
        Err("LISTEN_FDS: runner takes no listeners (fd 3)".into())
    );
    assert_eq!(super::inherited_listener(Which::Forgejo, &[]), Ok(None));
    let empty = [(Box::from(b"LISTEN_FDS".as_slice()), Box::from(b"".as_slice()))];
    assert_eq!(super::inherited_listener(Which::Forgejo, &empty), Ok(None));
}
