//! The base box fixture over explicit startup facts (temper/docs/design/testing/box/contracts.md, 2.1).
use crate::configuration::{file, parse_with_key, turn_bounds, typed_documents};
use crate::{Forge, owned};
use serde_json::Value;
use skein_lib::{List, Secret};
use std::{fs, os::unix::ffi::OsStrExt, path::Path};
use temper_agent_fixtures::{Setting, providers};
use temper_config_shell::{Read, Shape};
use temper_playground::documents::Configuration;
use temper_service::settings::Files;

/// The base box's typed limits, retained start file, named files and prospective agent.
pub struct Recipe {
    pub configuration: Configuration,
    pub files: Files,
    pub landing_settings: Option<Vec<u8>>,
    pub agent: temper_worker_shell::agent::Prepared,
    pub given: temper_service::stock::Given,
    start: Read,
    setting: Setting,
}
impl Recipe {
    /// The same file and start bytes that the native fixture reader returns.
    #[must_use]
    pub fn read_start(&self) -> &Read {
        &self.start
    }

    #[must_use]
    pub fn start_document(&self) -> Value {
        serde_json::from_slice(&self.start.document).expect("start document")
    }

    /// The agent and provider startup copies that back the prospective agent.
    #[must_use]
    pub fn agent_setting(&self) -> &Setting {
        &self.setting
    }

    fn lay(&self, root: &Path) {
        for name in ["accounts", "secrets", "state"] {
            fs::create_dir_all(root.join(name)).expect("deployment directory");
        }
        for (name, bytes) in &self.files.entries {
            let path = Path::new(std::ffi::OsStr::from_bytes(name));
            let path = if path.starts_with(root) { path.into() } else { root.join(path) };
            crate::write_private(&path, bytes.expose());
        }
        if let Some(bytes) = &self.landing_settings {
            owned::write_landing_settings(root, bytes);
        }
        self.setting.lay(root);
        crate::write_file(&root.join(temper_config_shell::FILE), &self.start.file, Shape::Standalone);
        crate::limit_settings::remember(root, &self.configuration);
    }
}

/// The base recipe's typed configuration. `root` is a value, never opened.
#[must_use]
pub fn configuration(root: &Path, forge: &Forge<'_>) -> Configuration {
    recipe(root, forge).configuration
}

/// Compose the complete base recipe with unopened listeners and no provider key.
#[must_use]
pub fn recipe(root: &Path, forge: &Forge<'_>) -> Recipe {
    let owned = match forge {
        Forge::Unreached => None,
        Forge::Throwaway { forgejo, api, .. } => Some(owned::Owned {
            forgejo: *api,
            forgejo_binary: (*forgejo).into(),
            review: false,
            ..owned::Owned::example()
        }),
    };
    compose(root, owned.as_ref(), false, temper_service::stock::providers())
}

/// Collect the native fixture's facts, then lay the base recipe once for a subprocess story.
#[expect(clippy::must_use_candidate, reason = "process stories may need only the laid files")]
pub fn lay_configuration(root: &Path, forge: &Forge<'_>) -> Configuration {
    assert!(Path::new("/usr/bin/git").is_file(), "/usr/bin/git is required");
    let path = root.join(temper_config_shell::FILE);
    let owned = match forge {
        Forge::Unreached => None,
        Forge::Throwaway { forgejo, api, .. } => {
            let mut owned = owned::Owned::beside(*api);
            owned.forgejo_binary = (*forgejo).into();
            if path.exists() {
                let previous = crate::read_file(&path, Shape::Standalone);
                owned.clients = previous.listen.unwrap_or(owned.clients);
                owned.webhooks = previous.webhooks.unwrap_or(owned.webhooks);
            }
            Some(owned)
        }
    };
    let provider_key = temper_agent_fixtures::data(root).join("provider.key").is_file();
    let recipe = compose(root, owned.as_ref(), provider_key, providers::values(&providers::read(root)));
    recipe.lay(root);
    recipe.configuration
}

fn compose(
    root: &Path,
    owned: Option<&owned::Owned>,
    provider_key: bool,
    mut providers: Box<[jig_protocol_config::ProviderSetting]>,
) -> Recipe {
    let (agent, mut configuration) = typed_documents(root, "/fixture/temper");
    let agent: Value = serde_json::from_slice(&agent).expect("agent JSON");
    if let Some(owned) = owned {
        let for_forge = crate::forgejo::configuration(owned, &mut configuration, agent.clone());
        providers = providers::take(&mut configuration).unwrap_or(providers);
        providers = temper_agent_fixtures::compose_setting(&mut configuration, &for_forge, &providers).providers;
    }
    providers = providers::take(&mut configuration).unwrap_or(providers);
    let setting = temper_agent_fixtures::compose_setting(&mut configuration, &agent, &providers);
    let composed = temper_agent_fixtures::with_figures(&configuration, &setting.figures, root, &setting.providers);
    turn_bounds::apply(&mut configuration, turn_bounds::turn_body(&composed));

    let mut file = file();
    let address = |name: &str| {
        configuration["application"][name]["listener"]["socket"].as_str().map(|v| v.parse().expect("listener"))
    };
    file.listen = address("clients");
    file.webhooks = address("webhooks");
    let start = parse_with_key(&temper_config_shell::render(&file, Shape::Standalone), Shape::Standalone, provider_key);
    let document = serde_json::from_slice(&start.document).expect("start document");
    let document = temper_agent_fixtures::with_figures(&document, &setting.figures, root, &setting.providers);
    let agent = prepare_agent(serde_json::to_vec(&document).expect("agent JSON").into_boxed_slice());
    configuration.agent_bounds(&agent.agent);
    let given = providers::given_with(setting.providers.clone(), Some(([127, 0, 0, 1], 8370).into()));
    let landing_settings = owned.map(|_| owned::producer_landing_settings_bytes(2));
    Recipe { configuration, files: files(root), landing_settings, agent, given, start, setting }
}

fn files(root: &Path) -> Files {
    let mut entries = List::with_capacity(8);
    for (name, bytes) in jig_channel_kit::fixtures::files(false).entries.into_boxed() {
        if !name.starts_with(b"charters/") && name.as_ref() != b"agent.json" {
            entries.push((name, bytes)).expect("fixture file count");
        }
    }
    for (name, bytes) in [
        (b"worker.secret".as_slice(), b"worker-secret".as_slice()),
        (b"secrets/forge-1.user", b"42\n"),
        (b"secrets/forge-1.api-token", b"forge-1-api-token-value"),
        (b"secrets/forge-1.git-token", b"forge-1-git-token-value"),
    ] {
        entries.push((name.into(), Secret::new(bytes.into()))).expect("fixture file count");
    }
    entries
        .push((root.join("root.der").as_os_str().as_bytes().into(), Secret::new(skein_tls_kit::pki::ROOT.into())))
        .expect("fixture trust file");
    Files { entries }
}

/// Prepare fixture agent bytes with an unopened programs path, fixture addresses and fixture trust.
#[must_use]
pub fn prepare_agent(bytes: Box<[u8]>) -> temper_worker_shell::agent::Prepared {
    temper_worker_shell::agent::prepare_bytes_resolving(
        bytes,
        |_| Ok(()),
        |decoded| {
            decoded.resolve_with(
                |address| Ok(address.parse().unwrap_or_else(|_| ([127, 0, 0, 1], 9).into())),
                |_, _| {
                    let (trust, _) =
                        skein_tls::Config::from_der(&[skein_tls_kit::pki::ROOT.into()], &[]).expect("fixture trust");
                    Ok(trust)
                },
                |_| None,
            )
        },
    )
    .expect("prospective fixture agent")
}
