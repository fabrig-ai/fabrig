//! Host the sign-in file effects; all printable bytes belong to forge-setup.
//! Source serving follows temper/docs/design/protocol/openid-provider.md, 9.
use temper_shell::directories::EnvironmentEntry;
mod http;
use jig_protocol_config as config;
use std::{
    fs,
    io::Write,
    os::unix::fs::{DirBuilderExt, OpenOptionsExt},
    path::{Path, PathBuf},
};
use temper_forge_setup::sign_in;
use temper_forge_setup_shell::process as host;
use temper_engine_service::startup::Named;
use temper_service_shell::jig_directories;

/// A dogfood hack's switch (temper/docs/design/draft/dogfood-hacks.md, 2.8).
pub(crate) fn first_sign_in(environment: &[EnvironmentEntry]) -> sign_in::FirstSignIn {
    if NO_REGISTRATION.value(environment).is_some() {
        sign_in::FirstSignIn::LinksOnly
    } else {
        sign_in::FirstSignIn::MakesAccount
    }
}

pub(crate) fn run(
    directories: &super::Directories,
    bytes: &[u8],
    named: &Named,
    number: u16,
    environment: &[EnvironmentEntry],
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
    let engine = temper_engine_service::startup::engine(
        &document,
        super::member(&document, 0, b"engine"),
        &jig_directories(directories),
    )
    .expect("checked engine");
    let provider = engine.clients.provider.as_ref().ok_or("engine.clients.provider is not configured")?;
    let client = provider.clients.iter().find(|client| client.id == row.client).expect("checked sign-in client");
    let secret = client.secret.as_ref().ok_or("its client has no secret; a box adds its own source")?;
    let discovery = discovery(bytes, named)?;
    let instructions =
        sign_in::instructions(b"forgejo", None, &client.id, secret, &discovery, first_sign_in(environment));
    make_missing(super::path(&provider.key), true)?;
    make_missing(super::path(secret), false)?;
    std::io::stdout().write_all(&instructions).map_err(|why| format!("print instructions: {why}"))?;
    Ok(0)
}

/// Never read existing credentials. New files are complete and private before rename.
pub fn make_missing(path: &Path, key: bool) -> Result<(), String> {
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

/// Add the public source after Forgejo answers, under the setup's one deadline.
pub(crate) fn source(
    bytes: &[u8],
    named: &Named,
    owned: &temper_forge_setup::setup::Owned,
    place: &temper_forge_setup::place::Place,
    deadline: skein_lib::Duration,
) -> Result<u8, String> {
    use temper_forge_setup::{prepare::Exit, spec};
    use temper_forge_setup_shell::command;
    let client = named
        .machine
        .owned
        .as_ref()
        .and_then(|owned| owned.sign_in.as_deref())
        .ok_or("machine.owned.sign_in is not configured")?;
    let discovery = discovery(bytes, named)?;
    let clock = skein_shell::Clock::new();
    let until = clock.now().now.saturating_add(deadline);
    let limits = jig_protocol_config::profile(config::Profile::Small).io;
    let answered = http::get(&discovery, until).is_ok_and(|answer| (200..300).contains(&answer.status));
    if !answered {
        return Err("temper's discovery did not answer".into());
    }
    // An unreadable list still attempts add: its refusal explains the failure.
    let present = command::run_until(&limits, &spec::source_list(owned, place), until)
        .is_ok_and(|answer| answer.exit == Exit::Code(0) && sign_in::source_present(answer.stdout.expose()));
    if present {
        let url = format!("http://{}/user/oauth2/temper", owned.listen);
        let issuer = discovery.strip_suffix(b"/.well-known/openid-configuration").expect("discovery path");
        let mut authorize = issuer.to_vec();
        authorize.extend_from_slice(b"/oauth/authorize");
        let served = http::get(url.as_bytes(), until).is_ok_and(|answer| {
            (300..400).contains(&answer.status)
                && answer.header(b"location").is_some_and(|location| {
                    location == authorize
                        || location
                            .strip_prefix(authorize.as_slice())
                            .is_some_and(|tail| tail.starts_with(b"?") || tail.starts_with(b"#"))
                })
        });
        if !served {
            return Err("Forgejo does not serve the source".into());
        }
    } else {
        let answer = command::run_until(&limits, &spec::source_add(owned, place, client, &discovery), until)
            .map_err(super::forge::command_error)?;
        if answer.exit != Exit::Code(0) {
            let diagnostic =
                if answer.stderr.expose().is_empty() { answer.stdout.expose() } else { answer.stderr.expose() };
            return Err(format!(
                "source registration ended: {:?}: {}",
                answer.exit,
                String::from_utf8_lossy(diagnostic.get(..diagnostic.len().min(512)).expect("bounded diagnostic"))
            ));
        }
    }
    std::io::stdout().write_all(&sign_in::Source::Added.line()).map_err(|why| format!("source outcome: {why}"))?;
    Ok(0)
}

/// The current issuer's origin and discovery path (protocol/openid-provider.md, 2).
pub(crate) fn discovery(bytes: &[u8], named: &Named) -> Result<Vec<u8>, String> {
    let document = config::document(
        bytes,
        &config::DocumentLimits { bytes: super::CONFIG_BYTES, tokens: super::CONFIG_BYTES, depth: 32 },
    )
    .expect("checked document");
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
    Ok(discovery)
}

/// First sign-in policy for the forge (cli/README.md, 8).
pub const NO_REGISTRATION: skein_shell::Variable = skein_shell::Variable {
    name: "TEMPER_INTERNAL_FORGE_NO_REGISTRATION",
    set: skein_shell::Set::Internal,
    form: "SWITCH",
    line: "Use links only instead of registering a first account.",
    absent: "Register the first account.",
    notes: "",
    way: skein_shell::Way::Reads,
};
