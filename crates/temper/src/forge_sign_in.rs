//! Host the sign-in file effects; all printable bytes belong to forge-setup.
use jig_protocol_config as config;
use std::{
    fs,
    io::Write,
    os::unix::fs::{DirBuilderExt, OpenOptionsExt},
    path::{Path, PathBuf},
};
use temper_forge_setup::{check::Checked, place::Place, setup::Mode, sign_in};
use temper_forge_setup_shell::process as host;
use temper_service::startup::Named;

pub(crate) fn run(
    directories: &super::Directories,
    bytes: &[u8],
    named: &Named,
    checked: &Checked,
    number: u16,
) -> Result<u8, String> {
    let row = named
        .sign_in
        .as_ref()
        .and_then(|sign_in| sign_in.forges.iter().find(|row| row.forge == number))
        .ok_or("it is not named by application.sign_in.forges")?;
    let document = config::document(
        bytes,
        &config::DocumentLimits { bytes: super::CONFIG_BYTES, tokens: super::CONFIG_BYTES, depth: 32 },
    )
    .expect("checked document");
    let engine = temper_service::startup::engine(&document, super::member(&document, 0, b"engine"), &directories.jig())
        .expect("checked engine");
    let provider = engine.clients.provider.as_ref().ok_or("engine.clients.provider is not configured")?;
    let client = provider.clients.iter().find(|client| client.id == row.client).expect("checked sign-in client");
    let entry = checked.setup.entry(number);
    let owned = match entry.map(|entry| &entry.mode) {
        Some(Mode::Owned(owned)) => Some(owned),
        Some(Mode::Managed { .. }) | None => None,
        Some(Mode::Fake(_)) => return Err("it is fake, not a Forgejo sign-in host".into()),
    };
    let application = super::member(&document, 0, b"application");
    let clients = super::member(&document, application, b"clients");
    let public = config::read::Object::new(&document, clients).expect("checked clients").member(&document, b"public");
    let origin = match public {
        Some(at) if document.token(at).expect("public value").kind != skein_json::document::Kind::Null => {
            config::read::text(&document, at, 4096).expect("checked public origin").to_vec()
        }
        Some(_) | None => format!("http://{}", super::address(&named.clients)?).into_bytes(),
    };
    let mut discovery = origin;
    discovery.extend_from_slice(b"/.well-known/openid-configuration");
    let place = entry.map(|entry| Place { directory: entry.directory.clone(), user: Box::new([]) });
    let instructions = sign_in::instructions(
        owned.map_or(b"forgejo".as_slice(), |owned| owned.forgejo.as_ref()),
        owned.and(place.as_ref()),
        &client.id,
        &client.secret,
        &discovery,
    );
    make_missing(super::path(&provider.key), true)?;
    make_missing(super::path(&client.secret), false)?;
    std::io::stdout().write_all(&instructions).map_err(|why| format!("print instructions: {why}"))?;
    Ok(0)
}

/// Never read existing credentials. New files are complete and private before rename.
fn make_missing(path: &Path, key: bool) -> Result<(), String> {
    match fs::symlink_metadata(path) {
        Ok(_) => return Ok(()),
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => {}
        Err(why) => return Err(format!("{}: {why}", path.display())),
    }
    let parent = path.parent().ok_or("credential path has no parent")?;
    fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(parent)
        .map_err(|why| format!("{}: {why}", parent.display()))?;
    let mut temporary: PathBuf = path.into();
    let seed = skein_shell::crypto_seed().map_err(|errno| format!("system randomness: errno {errno}"))?;
    let random: &[u8; 32] = seed.expose().try_into().expect("system seed has 32 bytes");
    let suffix = sign_in::secret(random);
    let mut name = path.as_os_str().to_os_string();
    name.push(".");
    name.push(std::str::from_utf8(suffix.expose()).expect("base64url suffix"));
    name.push(".partial");
    temporary.set_file_name(Path::new(&name).file_name().expect("partial file name"));
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&temporary)
        .map_err(|why| format!("{}: {why}", temporary.display()))?;
    let result = if key {
        host::sign_in_key(file)
    } else {
        skein_shell::crypto_seed().map_err(|errno| format!("system randomness: errno {errno}")).and_then(|seed| {
            let random: &[u8; 32] = seed.expose().try_into().expect("system seed has 32 bytes");
            file.write_all(sign_in::secret(random).expose()).map_err(|why| format!("{}: {why}", temporary.display()))
        })
    };
    let result = result.and_then(|()| fs::rename(&temporary, path).map_err(|why| format!("{}: {why}", path.display())));
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

#[cfg(test)]
mod tests;
