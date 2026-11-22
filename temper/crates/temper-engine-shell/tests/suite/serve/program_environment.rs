use skein_shell::Origin;
use temper_engine_shell::{serve::program_environment, testing::WORKING};
use temper_shell::directories::Directories;

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
    // protocol/configuration/directories.md: each directory holds at most 4096 bytes.
    const PATH_BYTES: usize = 4096;
    // Three quoted apostrophe paths: 3 * (4 * 4096 + 2), 39 name bytes, three spaces, and temper.
    const WORD_BYTES: usize = 49206;
    let path = vec![b'\''; PATH_BYTES].into_boxed_slice();
    let dirs = Directories { config: path.clone(), data: path.clone(), cache: path, origins: [Origin::Flag; 3] };
    let words = program_environment(&dirs, &[], WORKING, b"temper");
    assert_eq!(words.len(), WORD_BYTES);
    let record = w::ProgramEnvironmentAnswer { environment: words };
    assert_eq!(
        wire::decode_program_environment_answer(&wire::program_environment_answer(&record), &w::CEILINGS),
        Some(record)
    );
}
