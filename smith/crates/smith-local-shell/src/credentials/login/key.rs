//! Read a private key from piped input (shell/commands.md, 5, shell/credentials.md, 2).
//! Values enter the private store only; every diagnostic names the account.
use super::LoginOutcome;
use skein_io::file::Expect;
use std::{
    io::{IsTerminal, Read},
    path::Path,
};

pub(super) fn run(name: &str, file: &Path) -> Result<LoginOutcome, String> {
    keep(file, &mut std::io::stdin().lock(), std::io::stdin().is_terminal())?;
    eprintln!("account {name}: key kept");
    Ok(LoginOutcome::Usable)
}

fn keep(file: &Path, input: &mut dyn Read, terminal: bool) -> Result<(), String> {
    keep_with(file, input, terminal, |file, bytes, expected| {
        let entropy = crate::startup::oauth_entropy()?;
        super::super::private_file::store(file, bytes, expected, &entropy).map(|_| ())
    })
}

fn keep_with(
    file: &Path,
    input: &mut dyn Read,
    terminal: bool,
    publish_private: impl FnOnce(&Path, Box<[u8]>, Expect) -> Result<(), String>,
) -> Result<(), String> {
    if terminal {
        return Err("key input must be piped; a terminal echoes credentials".into());
    }
    let bound = crate::credentials::token_limits().token_bytes;
    let mut bytes = Vec::new();
    input
        .take(u64::from(bound).checked_add(2).expect("key input bound"))
        .read_to_end(&mut bytes)
        .map_err(|_| "cannot read key input")?;
    let bytes = super::super::key::value(&bytes).map_err(|_| "empty or invalid key input")?;
    publish_private(file, bytes, Expect::Any)
}

#[cfg(test)]
mod tests;
