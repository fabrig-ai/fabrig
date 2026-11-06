use crate::directories::{Directories, EnvironmentEntry, resolve};
use skein_shell::Origin;
const WORKING: &[u8] = b"/work";
fn home() -> Vec<EnvironmentEntry> {
    vec![(Box::from(b"HOME".as_slice()), Box::from(b"/home/person".as_slice()))]
}
fn defaults() -> Directories {
    resolve(&[], &home(), WORKING).expect("defaults").0
}
#[test]
fn the_default_directories_answer_nothing() {
    assert_eq!(defaults().program_environment(&home(), WORKING).as_ref(), b"");
}
#[test]
fn flags_naming_the_defaults_answer_nothing() {
    let dirs = resolve(&defaults().flags(), &home(), WORKING).expect("child flags").0;
    assert_eq!(dirs.program_environment(&home(), WORKING).as_ref(), b"");
}
#[test]
fn one_moved_directory_names_all_three() {
    for (variable, expected) in [
        (
            b"TEMPER_CONFIG".as_slice(),
            "TEMPER_CONFIG='/moved' TEMPER_DATA='/home/person/.local/share/temper' TEMPER_CACHE='/home/person/.cache/temper'",
        ),
        (
            b"TEMPER_DATA".as_slice(),
            "TEMPER_CONFIG='/home/person/.config/temper' TEMPER_DATA='/moved' TEMPER_CACHE='/home/person/.cache/temper'",
        ),
        (
            b"TEMPER_CACHE".as_slice(),
            "TEMPER_CONFIG='/home/person/.config/temper' TEMPER_DATA='/home/person/.local/share/temper' TEMPER_CACHE='/moved'",
        ),
    ] {
        let mut environment = home();
        environment.push((variable.into(), b"/moved".as_slice().into()));
        let dirs = resolve(&[], &environment, WORKING).expect("variable").0;
        assert_eq!(dirs.program_environment(&environment, WORKING).as_ref(), expected.as_bytes());
    }
}
#[test]
fn no_home_names_all_three() {
    assert_eq!(defaults().program_environment(&[], WORKING).as_ref(), b"TEMPER_CONFIG='/home/person/.config/temper' TEMPER_DATA='/home/person/.local/share/temper' TEMPER_CACHE='/home/person/.cache/temper'");
}
#[test]
fn the_words_survive_a_shell() {
    use std::os::unix::ffi::OsStrExt;
    let dirs = Directories {
        config: b"/space ' $`\nconfig".as_slice().into(),
        data: b"/data ' $`\n".as_slice().into(),
        cache: b"/cache ' $`\n".as_slice().into(),
        origins: [Origin::Flag; 3],
    };
    let mut script = dirs.program_environment(&home(), WORKING).into_vec();
    script.extend_from_slice(b" sh -c 'printf \"%s\\0%s\\0%s\" \"$TEMPER_CONFIG\" \"$TEMPER_DATA\" \"$TEMPER_CACHE\"'");
    let output = std::process::Command::new("sh")
        .args([std::ffi::OsStr::new("-c"), std::ffi::OsStr::from_bytes(&script)])
        .env_clear()
        .output()
        .expect("shell");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let expected = [dirs.config.as_ref(), b"\0", dirs.data.as_ref(), b"\0", dirs.cache.as_ref()].concat();
    assert_eq!(output.stdout, expected);
}
#[test]
fn the_widest_directories_fit_the_wire() {
    use temper_client_protocol_wire::{self as wire, v1 as w};
    // configuration.md, 12: each directory holds at most 4096 bytes.
    const PATH_BYTES: usize = 4096;
    // Three quoted apostrophe paths: 3 * (4 * 4096 + 2), 39 name bytes, two spaces.
    const WORD_BYTES: usize = 49199;
    let path = vec![b'\''; PATH_BYTES].into_boxed_slice();
    let dirs = Directories { config: path.clone(), data: path.clone(), cache: path, origins: [Origin::Flag; 3] };
    let words = dirs.program_environment(&[], WORKING);
    assert_eq!(words.len(), WORD_BYTES);
    let record =
        w::ProgramEnvironmentAnswer::new(&w::CEILINGS, w::ProgramEnvironmentAnswerParts { environment: words })
            .expect("widest words fit");
    assert_eq!(
        wire::decode_program_environment_answer(&wire::program_environment_answer(&record), &w::CEILINGS),
        Some(record)
    );
}
