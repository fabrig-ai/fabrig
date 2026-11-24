//! The pre-store copy of the current providers (protocol/configuration/providers.md, 3).
use jig_protocol_config::{self as config, ProviderSetting};
use std::{
    io::{Read, Write},
    os::unix::fs::OpenOptionsExt,
    path::Path,
};

// Six JSON bytes per escaped byte: the bounded registrations and endpoints fit below 128 MiB.
const BYTES: u32 = 128 * 1024 * 1024;

/// A missing or unreadable copy leaves the application's defaults running.
pub fn read(path: &Path, defaults: Box<[ProviderSetting]>, log: &mut impl Write) -> Box<[ProviderSetting]> {
    read_using(path, defaults, log, load)
}

/// Decode the startup copy supplied by a reader, with the same bound and fallback as the file reader.
pub fn read_using(
    path: &Path,
    defaults: Box<[ProviderSetting]>,
    log: &mut impl Write,
    load: impl FnOnce(&Path) -> std::io::Result<Option<Vec<u8>>>,
) -> Box<[ProviderSetting]> {
    match load(path).map_err(|why| why.to_string()).and_then(|bytes| bytes.as_deref().map(decode).transpose()) {
        Ok(Some(values)) => values,
        Ok(None) => defaults,
        Err(why) => {
            let _ =
                writeln!(log, "providers.setting: {why}; the stock's providers run until the store's are kept again");
            defaults
        }
    }
}
fn load(path: &Path) -> std::io::Result<Option<Vec<u8>>> {
    let file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(why) => return Err(why),
    };
    let mut bytes = Vec::new();
    file.take(u64::from(BYTES) + 1).read_to_end(&mut bytes)?;
    Ok(Some(bytes))
}
fn decode(bytes: &[u8]) -> Result<Box<[ProviderSetting]>, String> {
    let document = config::document(bytes, &config::DocumentLimits { bytes: BYTES, tokens: 65_536, depth: 32 })
        .map_err(|_| "cannot decode the provider list")?;
    let mut problems = config::Problems::empty();
    let values = config::llm::providers_setting(&document, 0, &config::Path::root(), &mut problems)
        .ok_or("cannot decode the provider list")?;
    if !problems.is_empty() {
        return Err("cannot decode the provider list".into());
    }
    Ok(values)
}
#[must_use]
pub fn render(values: &[ProviderSetting]) -> Box<[u8]> {
    let limits = skein_json::writer::Limits { depth: 32, length: BYTES };
    let mut measure = skein_json::writer::Encoder::measure(&limits);
    config::llm::render_providers(values, &mut measure);
    let mut writer = skein_json::writer::Encoder::write(measure.measured().expect("bounded providers JSON"), &limits);
    config::llm::render_providers(values, &mut writer);
    writer.finish()
}
/// Returns whether different bytes were durably replaced. Never changes an equal file.
pub fn write(path: &Path, values: &[ProviderSetting]) -> std::io::Result<bool> {
    let bytes = render(values);
    if std::fs::read(path).is_ok_and(|previous| previous.as_slice() == bytes.as_ref()) {
        return Ok(false);
    }
    let temporary = path.with_extension("setting.new");
    match std::fs::remove_file(&temporary) {
        Ok(()) => {}
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => {}
        Err(why) => return Err(why),
    }
    let result = (|| {
        let mut file = std::fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(&temporary)?;
        file.write_all(&bytes)?;
        file.sync_all()?;
        std::fs::rename(&temporary, path)?;
        std::fs::File::open(path.parent().expect("data directory"))?.sync_all()?;
        Ok(true)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(temporary);
    }
    result
}

#[cfg(test)]
mod tests;
