//! Only current init output, with an explicit bundle, is importable.
use super::{Directories, public};
use crate::directories::jig_directories;
use std::path::PathBuf;
use temper_config_shell::{FILE, File, Shape, render};

pub(super) struct Source {
    pub marker: Box<[u8]>,
    pub file: Option<File>,
    pub bundle: Option<PathBuf>,
    pub bundle_limits: temper_service::limits::Clients,
}
impl Source {
    pub(super) fn read(dirs: &Directories, _environment: &[crate::EnvironmentEntry]) -> Result<Self, String> {
        let config = crate::path(&dirs.config);
        Self::read_with(
            dirs,
            || match std::fs::read_dir(config) {
                Ok(mut entries) => {
                    entries.next().transpose().map(|entry| entry.is_none()).map_err(|why| why.to_string())
                }
                Err(why) if why.kind() == std::io::ErrorKind::NotFound => Ok(true),
                Err(why) => Err(format!("source configuration {}: {why}", config.display())),
            },
            || temper_config_shell::read(dirs, Shape::Standalone).map_err(|why| why.to_string()),
            || public(&config.join(FILE)),
        )
    }

    pub(super) fn read_with(
        dirs: &Directories,
        empty: impl FnOnce() -> Result<bool, String>,
        configuration: impl FnOnce() -> Result<temper_config_shell::Read, String>,
        public_document: impl FnOnce() -> Result<Box<[u8]>, String>,
    ) -> Result<Self, String> {
        let empty = empty()?;
        let mut marker =
            format!("temper fake-backends v1\nconfig={:?}\ndata={:?}\n", dirs.config, dirs.data).into_bytes();
        if empty {
            marker.extend_from_slice(b"empty\n");
            return Ok(Self {
                marker: marker.into(),
                file: None,
                bundle: None,
                bundle_limits: temper_service::limits::stock().clients,
            });
        }
        let read = configuration()?;
        // Exact source bytes identify reuse; only the table's reader interprets them.
        let source = public_document()?;
        let file = read.file;
        let listen = file.listen.ok_or_else(|| format!("{FILE}: listen: expected init's listener"))?;
        let webhook = file.webhooks.unwrap_or_else(|| {
            ([127, 0, 0, 1], listen.port().checked_add(1).expect("the reader checked the default port")).into()
        });
        let port = |address: std::net::SocketAddr, key: &str| {
            if address.ip() != std::net::Ipv4Addr::LOCALHOST {
                return Err(format!("{FILE}: {key}: expected init's IPv4 loopback socket"));
            }
            std::num::NonZeroU16::new(address.port())
                .ok_or_else(|| format!("{FILE}: {key}: expected init's nonzero port"))
        };
        let mut expected = crate::init::configuration(port(listen, "listen")?, port(webhook, "webhooks")?);
        expected.internal.bundle.clone_from(&file.internal.bundle);
        if file != expected {
            let rendered = render(&file, Shape::Standalone);
            let text = String::from_utf8_lossy(&rendered);
            let keys = text
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty() && !line.starts_with('#'))
                .map(|line| line.split_once('=').map_or(line, |(key, _)| key.trim()))
                .collect::<Vec<_>>()
                .join(", ");
            return Err(format!(
                "{FILE}: only init's listen, webhooks and [internal] bundle are supported; source keys: {keys}"
            ));
        }
        let checked = crate::setup::check(&read.document, &jig_directories(dirs))?;
        marker.extend_from_slice(&skein_lib::sha256::digest(&source));
        Ok(Self {
            marker: marker.into(),
            bundle: file.internal.bundle.clone(),
            file: Some(file),
            bundle_limits: checked.named.bundle_limits,
        })
    }
    /// The file has no data paths: defaults resolve in the effective directories.
    pub(super) fn rebase(&self, _effective: &Directories) -> Option<Box<[u8]>> {
        self.file.as_ref().map(|file| render(file, Shape::Standalone))
    }
}
