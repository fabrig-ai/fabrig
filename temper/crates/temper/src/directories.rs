//! The binary's three directories (protocol/configuration.md, 12).
pub use temper_shell::directories::*;

/// Words needed by program-login on this deployment (temper/docs/design/client/README.md, 4.6).
#[must_use]
pub fn program_environment(
    directories: &Directories,
    environment: &[EnvironmentEntry],
    working: &[u8],
    word: &[u8],
) -> Box<[u8]> {
    // temper/docs/design/client/README.md, 4.6: the answer's byte bound.
    const BOUND: usize = 49_216;
    let defaults: Vec<_> = environment
        .iter()
        .filter(|(name, _)| ![b"TEMPER_CONFIG".as_slice(), b"TEMPER_DATA", b"TEMPER_CACHE"].contains(&name.as_ref()))
        .cloned()
        .collect();
    let defaults = resolve(&[], &defaults, working).ok().map(|(defaults, _)| defaults);
    let home = environment.iter().find(|(name, _)| name.as_ref() == b"HOME").map(|(_, value)| value.as_ref());
    let mut words = Vec::new();
    for (name, path, default) in [
        (b"TEMPER_CONFIG=".as_slice(), &directories.config, defaults.as_ref().map(|dirs| &dirs.config)),
        (b"TEMPER_DATA=", &directories.data, defaults.as_ref().map(|dirs| &dirs.data)),
        (b"TEMPER_CACHE=", &directories.cache, defaults.as_ref().map(|dirs| &dirs.cache)),
    ] {
        if Some(path) != default {
            words.extend_from_slice(name);
            words.extend_from_slice(&crate::printed::path_word(path, home));
            words.push(b' ');
        }
    }
    let word = if word.len() <= BOUND.saturating_sub(words.len()) { word } else { b"temper" };
    if !words.is_empty() || word != b"temper" {
        words.extend_from_slice(word);
    }
    words.into_boxed_slice()
}

#[cfg(test)]
pub(crate) mod tests;
