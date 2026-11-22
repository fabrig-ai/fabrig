use std::os::unix::{ffi::OsStrExt, fs::PermissionsExt};
use temper_box_shell::{
    printed,
    testing::{path, program_login::line},
};

#[test]
fn the_printed_line_preserves_shell_arguments_from_another_directory() {
    let directory = std::env::temp_dir().join(format!("temper-printed-login-{}", std::process::id()));
    std::fs::create_dir(&directory).expect("printed command scratch");
    let binary = directory.join("temper ' binary");
    std::fs::write(&binary, b"#!/bin/sh\nprintf '%s\\n' \"$@\"\n").expect("stub program");
    std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o700)).expect("executable stub");
    let mut directories = temper_box_kit::configuration::directories(&directory);
    directories.config = directory.join("config ' one").as_os_str().as_bytes().into();
    directories.data = directory.join("data ' two").as_os_str().as_bytes().into();
    directories.cache = directory.join("cache ' three").as_os_str().as_bytes().into();
    let home = directory.as_os_str().as_bytes();
    let word = printed::path_word(binary.as_os_str().as_bytes(), Some(home));
    let printed = line(&directories, &word, Some(home)).expect("portable command");
    assert!(printed.starts_with("~/'temper"));
    assert!(printed.contains(" --data ~/'data"));
    let output = std::process::Command::new("/bin/sh")
        .args(["-c", &format!("{printed} source")])
        .current_dir("/")
        .env_clear()
        .env("PATH", "/missing")
        .env("HOME", &directory)
        .output()
        .expect("paste command into shell");
    assert!(output.status.success(), "{:?}", output.stderr);
    let expected = format!(
        "program-login\n--config\n{}\n--data\n{}\n--cache\n{}\nsource\n",
        path(&directories.config).display(),
        path(&directories.data).display(),
        path(&directories.cache).display()
    );
    assert_eq!(output.stdout, expected.as_bytes());
    std::fs::remove_dir_all(directory).expect("remove printed command scratch");
}
