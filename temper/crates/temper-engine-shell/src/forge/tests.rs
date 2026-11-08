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
