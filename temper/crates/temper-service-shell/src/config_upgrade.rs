//! The print-only configuration upgrade (temper/docs/design/protocol/configuration/file.md, 7).
use skein_shell::{Argument, Command, Reach, Set, Table};
use std::{os::unix::ffi::OsStrExt, path::Path};
use temper_config_shell::Shape;
use temper_shell::{
    directories::{self, Directories, EnvironmentEntry},
    table,
};

pub const COMMAND: Command = Command {
    reach: Reach::Words(&["config", "upgrade"]),
    set: Set::Public,
    forms: &["[DIRECTORIES]"],
    line: temper_config_shell::UPGRADE_LINE,
    arguments: &[Argument {
        shown: temper_shell::commands::DIRECTORIES.shown,
        line: temper_shell::commands::DIRECTORIES.line,
    }],
    settings: &[&directories::SETTINGS],
    switches: &[],
    variables: &[],
    notes: "Prints the configuration on stdout and any notices on stderr. Writes no files and starts nothing.",
};

pub fn configure(
    arguments: &[Box<[u8]>],
    environment: &[EnvironmentEntry],
    working: &[u8],
) -> Result<Directories, String> {
    let (directories, rest) = directories::resolve(arguments, environment, working).map_err(|why| why.to_string())?;
    if !rest.is_empty() {
        return Err("invalid arguments".into());
    }
    Ok(directories)
}

pub fn answer(binary: &str, directories: &Directories, shape: Shape) -> Result<String, String> {
    let read = super::configuration::read_only(directories, shape)
        .map_err(|why| super::prefixed(&format!("{binary} config upgrade"), &why))?;
    Ok(String::from_utf8(temper_config_shell::render(&read.file, shape).into_vec()).expect("configuration UTF-8"))
}

pub fn reply(
    table: &Table,
    arguments: &[Box<[u8]>],
    environment: &[EnvironmentEntry],
    working: Result<&Path, &str>,
    shape: Shape,
) -> Result<String, (u8, String)> {
    let working = working.map_err(|why| (1, format!("{} config upgrade: {why}\n", table.binary)))?;
    let directories = configure(arguments, environment, working.as_os_str().as_bytes())
        .map_err(|why| (2, table::refused(table, &COMMAND, &why)))?;
    answer(table.binary, &directories, shape).map_err(|why| (1, why))
}
