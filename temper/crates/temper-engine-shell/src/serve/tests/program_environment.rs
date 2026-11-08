use super::{Directories, EnvironmentEntry, program_environment};
use skein_shell::Origin;
use temper_shell::directories::resolve;
const WORKING: &[u8] = b"/work";
fn home() -> Vec<EnvironmentEntry> {
    vec![(Box::from(b"HOME".as_slice()), Box::from(b"/home/person".as_slice()))]
}
fn defaults() -> Directories {
    resolve(&[], &home(), WORKING).expect("defaults").0
}
#[test]
fn the_default_directories_answer_nothing() {
    assert_eq!(program_environment(&defaults(), &home(), WORKING, b"temper").as_ref(), b"");
}
#[test]
fn flags_naming_the_defaults_answer_nothing() {
    let dirs = resolve(&defaults().flags(), &home(), WORKING).expect("child flags").0;
    assert_eq!(program_environment(&dirs, &home(), WORKING, b"temper").as_ref(), b"");
}
#[test]
fn each_directory_is_named_only_off_its_own_default() {
    for (variables, expected) in [
        (vec![], ""),
        (vec!["TEMPER_CONFIG"], "TEMPER_CONFIG=/moved temper"),
        (vec!["TEMPER_DATA"], "TEMPER_DATA=/moved temper"),
        (vec!["TEMPER_CACHE"], "TEMPER_CACHE=/moved temper"),
        (vec!["TEMPER_CONFIG", "TEMPER_DATA"], "TEMPER_CONFIG=/moved TEMPER_DATA=/moved temper"),
        (vec!["TEMPER_CONFIG", "TEMPER_CACHE"], "TEMPER_CONFIG=/moved TEMPER_CACHE=/moved temper"),
        (vec!["TEMPER_DATA", "TEMPER_CACHE"], "TEMPER_DATA=/moved TEMPER_CACHE=/moved temper"),
        (
            vec!["TEMPER_CONFIG", "TEMPER_DATA", "TEMPER_CACHE"],
            "TEMPER_CONFIG=/moved TEMPER_DATA=/moved TEMPER_CACHE=/moved temper",
        ),
    ] {
        let mut environment = home();
        environment.extend(variables.iter().map(|name| (name.as_bytes().into(), b"/moved".as_slice().into())));
        let dirs = resolve(&[], &environment, WORKING).expect("variables").0;
        assert_eq!(program_environment(&dirs, &environment, WORKING, b"temper").as_ref(), expected.as_bytes());
    }
}
#[test]
fn a_directory_off_its_default_alone_is_named() {
    for (variables, flags, word, expected) in [
        (vec![("TEMPER_DATA", "/home/person/x")], vec![], "temper", "TEMPER_DATA=~/x temper"),
        (vec![("TEMPER_DATA", "/home/person/x")], vec![], "~/bin/temper", "TEMPER_DATA=~/x ~/bin/temper"),
        (vec![], vec!["--data", "/moved"], "temper", "TEMPER_DATA=/moved temper"),
        (
            vec![("TEMPER_DATA", "/home/person/.local/share/temper"), ("TEMPER_CACHE", "/moved")],
            vec![],
            "temper",
            "TEMPER_CACHE=/moved temper",
        ),
        (vec![("XDG_DATA_HOME", "/xdg")], vec![], "temper", ""),
    ] {
        let mut environment = home();
        environment.extend(variables.iter().map(|(name, value)| (name.as_bytes().into(), value.as_bytes().into())));
        let flags: Vec<Box<[u8]>> = flags.iter().map(|flag| flag.as_bytes().into()).collect();
        let dirs = resolve(&flags, &environment, WORKING).expect("directory").0;
        assert_eq!(program_environment(&dirs, &environment, WORKING, word.as_bytes()).as_ref(), expected.as_bytes());
    }
}
#[test]
fn no_home_names_all_three() {
    assert_eq!(program_environment(&defaults(), &[], WORKING, b"temper").as_ref(), b"TEMPER_CONFIG=/home/person/.config/temper TEMPER_DATA=/home/person/.local/share/temper TEMPER_CACHE=/home/person/.cache/temper temper");
}
#[test]
fn the_default_directories_only_need_a_different_binary_word() {
    assert_eq!(program_environment(&defaults(), &home(), WORKING, b"~/bin/temper").as_ref(), b"~/bin/temper");
}
#[test]
fn moved_directories_and_the_binary_share_short_words() {
    let dirs = Directories {
        config: b"/home/person/a".as_slice().into(),
        data: b"/home/person/b".as_slice().into(),
        cache: b"/moved".as_slice().into(),
        origins: [Origin::Flag; 3],
    };
    assert_eq!(
        program_environment(&dirs, &home(), WORKING, b"~/bin/temper").as_ref(),
        b"TEMPER_CONFIG=~/a TEMPER_DATA=~/b TEMPER_CACHE=/moved ~/bin/temper"
    );
}
#[test]
fn the_words_survive_a_shell() {
    use std::os::unix::{ffi::OsStrExt, fs::PermissionsExt};
    let scratch = skein_scratch::Scratch::new("printed-words");
    let home_path = scratch.path().as_os_str().as_bytes();
    let under_home = |tail: &[u8]| [home_path, b"/", tail].concat().into_boxed_slice();
    let dirs = Directories {
        config: under_home(b"space ' $`\nconfig"),
        data: under_home(b"data ' $`\n\xff"),
        cache: under_home(b"cache ' $`\n"),
        origins: [Origin::Flag; 3],
    };
    let binary = under_home(b"bin owner's/temper");
    let binary_path = std::path::Path::new(std::ffi::OsStr::from_bytes(&binary));
    std::fs::create_dir_all(binary_path.parent().expect("binary parent")).expect("binary directory");
    std::fs::write(
        binary_path,
        b"#!/bin/sh\nprintf '%s\\0%s\\0%s\\0%s' \"$TEMPER_CONFIG\" \"$TEMPER_DATA\" \"$TEMPER_CACHE\" \"$0\"\n",
    )
    .expect("test binary");
    std::fs::set_permissions(binary_path, std::fs::Permissions::from_mode(0o700)).expect("executable");
    let environment = vec![(b"HOME".as_slice().into(), home_path.into())];
    let word = super::printed::path_word(&binary, Some(home_path));
    let script = program_environment(&dirs, &environment, WORKING, &word);
    let output = std::process::Command::new("sh")
        .args([std::ffi::OsStr::new("-c"), std::ffi::OsStr::from_bytes(&script)])
        .current_dir(scratch.path())
        .env_clear()
        .env("HOME", scratch.path())
        .output()
        .expect("shell");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let expected =
        [dirs.config.as_ref(), b"\0", dirs.data.as_ref(), b"\0", dirs.cache.as_ref(), b"\0", &binary].concat();
    assert_eq!(output.stdout, expected);
}
#[test]
fn the_widest_directories_fit_the_wire() {
    use temper_client_protocol_wire::{self as wire, v1 as w};
    // configuration.md, 12: each directory holds at most 4096 bytes.
    const PATH_BYTES: usize = 4096;
    // Three quoted apostrophe paths: 3 * (4 * 4096 + 2), 39 name bytes, three spaces, and temper.
    const WORD_BYTES: usize = 49206;
    let path = vec![b'\''; PATH_BYTES].into_boxed_slice();
    let dirs = Directories { config: path.clone(), data: path.clone(), cache: path, origins: [Origin::Flag; 3] };
    let words = program_environment(&dirs, &[], WORKING, b"temper");
    assert_eq!(words.len(), WORD_BYTES);
    let record =
        w::ProgramEnvironmentAnswer::new(&w::CEILINGS, w::ProgramEnvironmentAnswerParts { environment: words })
            .expect("widest words fit");
    assert_eq!(
        wire::decode_program_environment_answer(&wire::program_environment_answer(&record), &w::CEILINGS),
        Some(record)
    );
}

#[test]
fn a_binary_word_past_the_answer_bound_gives_way_to_temper() {
    // client/README.md, 4.6: 49216 answer bytes; widest directory prefix is 49200.
    const BOUND: usize = 49216;
    const WIDEST_PREFIX: usize = 49200;
    const PATH_BYTES: usize = 4096;
    let path = vec![b'\''; PATH_BYTES].into_boxed_slice();
    let widest = Directories { config: path.clone(), data: path.clone(), cache: path, origins: [Origin::Flag; 3] };
    for (dirs, environment, room) in [(defaults(), home(), BOUND), (widest, vec![], BOUND - WIDEST_PREFIX)] {
        let at_bound = vec![b'a'; room];
        let words = program_environment(&dirs, &environment, WORKING, &at_bound);
        assert_eq!(words.len(), BOUND);
        assert!(words.ends_with(&at_bound));
        let past_bound = vec![b'a'; room + 1];
        let words = program_environment(&dirs, &environment, WORKING, &past_bound);
        assert!(words.len() <= BOUND, "a binary past the bound must give way");
        assert_eq!(words, program_environment(&dirs, &environment, WORKING, b"temper"));
    }
}
