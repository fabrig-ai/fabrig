//! The binary's three directories (protocol/configuration.md, 12).
use skein_shell::Origin;
use std::path::PathBuf;

pub use temper_shell::directories::{Refusal, SETTINGS};
pub(crate) use temper_shell::directories::{make_private, private_root};
type Argument = Box<[u8]>;
pub(crate) type EnvironmentEntry = (Argument, Argument);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Directories {
    pub config: Box<[u8]>,
    pub data: Box<[u8]>,
    pub cache: Box<[u8]>,
    pub origins: [Origin; 3],
}
impl From<temper_shell::directories::Paths> for Directories {
    fn from(paths: temper_shell::directories::Paths) -> Self {
        Self { config: paths.config, data: paths.data, cache: paths.cache, origins: paths.origins }
    }
}
pub fn resolve(
    arguments: &[Argument],
    environment: &[EnvironmentEntry],
    working: &[u8],
) -> Result<(Directories, Box<[Argument]>), Refusal> {
    temper_shell::directories::resolve(arguments, environment, working).map(|(paths, rest)| (paths.into(), rest))
}
impl Directories {
    /// Words needed by program-login on this deployment (temper/docs/design/client/README.md, 4.6).
    #[must_use]
    pub fn program_environment(&self, environment: &[EnvironmentEntry], working: &[u8], word: &[u8]) -> Box<[u8]> {
        // temper/docs/design/client/README.md, 4.6: the answer's byte bound.
        const BOUND: usize = 49_216;
        let defaults: Vec<_> = environment
            .iter()
            .filter(|(name, _)| {
                ![b"TEMPER_CONFIG".as_slice(), b"TEMPER_DATA", b"TEMPER_CACHE"].contains(&name.as_ref())
            })
            .cloned()
            .collect();
        let defaults = resolve(&[], &defaults, working).ok().map(|(defaults, _)| defaults);
        let home = environment.iter().find(|(name, _)| name.as_ref() == b"HOME").map(|(_, value)| value.as_ref());
        let mut words = Vec::new();
        for (name, path, default) in [
            (b"TEMPER_CONFIG=".as_slice(), &self.config, defaults.as_ref().map(|dirs| &dirs.config)),
            (b"TEMPER_DATA=", &self.data, defaults.as_ref().map(|dirs| &dirs.data)),
            (b"TEMPER_CACHE=", &self.cache, defaults.as_ref().map(|dirs| &dirs.cache)),
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

    #[must_use]
    pub fn jig(&self) -> jig_protocol_config::Directories {
        jig_protocol_config::Directories::given(&self.config, &self.data, &self.cache)
            .expect("resolved absolute bounded directories")
    }
    #[must_use]
    pub fn document(&self) -> PathBuf {
        super::path(&self.config).join("temper.json")
    }
    #[must_use]
    pub fn flags(&self) -> Box<[Argument]> {
        vec![
            Box::from(b"--config".as_slice()),
            self.config.clone(),
            Box::from(b"--data".as_slice()),
            self.data.clone(),
            Box::from(b"--cache".as_slice()),
            self.cache.clone(),
        ]
        .into_boxed_slice()
    }
    pub fn open(&self, user: u32) -> Result<(), String> {
        for directory in [&self.data, &self.cache] {
            make_private(super::path(directory), user)?;
        }
        Ok(())
    }
    /// Missing documents are refused before data or cache is touched.
    pub(crate) fn read(&self) -> Result<Box<[u8]>, String> {
        let document = self.document();
        match std::fs::symlink_metadata(&document) {
            Err(why) if why.kind() == std::io::ErrorKind::NotFound => {
                return Err(format!("configuration {}: absent; temper init writes one", document.display()));
            }
            Err(why) => return Err(format!("configuration {}: {why}", document.display())),
            Ok(_) => {}
        }
        self.open(skein_shell::effective_user())?;
        skein_shell::read_file(&document, super::CONFIG_BYTES)
            .map_err(|why| format!("configuration {}: {}", document.display(), super::unread(why)))
    }
}
#[cfg(test)]
pub(crate) mod tests;
