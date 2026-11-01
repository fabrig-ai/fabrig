//! The document's first-start forge records (temper/docs/plan/playground-reset-r4/notes/contracts.md, D1).
use super::{Problems, What, application};
use crate::settings;
use alloc::boxed::Box;
use jig_protocol_config::{self as config, Path, Segment};
use skein_lib::List;
use temper_engine_domain_forge::{Forge, Kind, record as r};
use temper_forge_setup::setup as s;

/// Convert checked settings and the file form's names into first-start records.
/// Typed fixtures without file names pass an empty members slice.
pub fn seed(
    members: &[application::Forge],
    identities: &[application::Identity],
    settings: &settings::Engine,
) -> Result<Box<[Forge]>, Problems> {
    let mut rows = List::with_capacity(settings.limits.forge.forges);
    let mut problems = Problems::empty();
    for (index, forge) in settings.forges.iter().enumerate() {
        let path = Path::root()
            .member(b"application")
            .member(b"forges")
            .child(Segment::Index(u32::try_from(index).expect("bounded forges")));
        let mut identity = None;
        for value in &settings.identities {
            if value.number == forge.identity {
                identity = Some(value);
                break;
            }
        }
        let Some(identity) = identity else {
            problems.push(path.member(b"identity"), What::Jig(config::What::Form));
            continue;
        };
        let mut files = None;
        for member in members {
            if member.number == forge.number && member.kind == settings::Kind::Forgejo && !member.records_only {
                files = Some(r::Files {
                    api_token: member.api_token.clone(),
                    webhook_secret: member.webhook_secret.clone(),
                    user: match &member.user {
                        application::User::Id(id) => r::FileUser::Id(*id),
                        application::User::File(name) => r::FileUser::File(name.clone()),
                    },
                    git_token: None,
                });
                for named in identities {
                    if named.number == forge.identity && !named.token.is_empty() {
                        files.as_mut().expect("file form").git_token = Some(named.token.clone());
                    }
                }
            }
        }
        let mut setup = None;
        for value in &settings.setup {
            if value.entry.forge == forge.number {
                setup = Some(entry(&value.entry));
                break;
            }
        }
        let row = Forge {
            number: forge.number,
            kind: match forge.kind {
                settings::Kind::Forgejo => Kind::Forgejo,
                settings::Kind::GitHub => Kind::GitHub,
            },
            api: match &forge.api {
                config::Address::Addr(skein_io::Addr::Ip(address)) => r::Address::Socket(*address),
                config::Address::Host { name, port } => r::Address::Host { name: name.clone(), port: *port },
                config::Address::Addr(skein_io::Addr::InProcess(_)) => {
                    problems.push(path.member(b"api"), What::Jig(config::What::Form));
                    continue;
                }
            },
            transport: match &forge.transport {
                settings::Transport::Plaintext => r::Transport::Plaintext,
                settings::Transport::Tls { server_name, .. } => {
                    let mut trust = None;
                    for member in members {
                        if member.number == forge.number
                            && let application::Transport::Tls { trust: name, .. } = &member.transport
                        {
                            trust.clone_from(name);
                        }
                    }
                    r::Transport::Tls { server_name: server_name.clone(), trust }
                }
            },
            git: forge.git.clone(),
            git_program: forge.git_program.clone(),
            web: forge.web.clone(),
            read_only: forge.app_read_only,
            identity: r::Identity {
                number: identity.number,
                login: identity.login.clone(),
                name: identity.author.name.clone(),
                email: identity.author.email.clone(),
            },
            files,
            setup,
            first: None,
            playground: match &settings.demo {
                Some(demo) => demo.forge == forge.number,
                None => false,
            },
            removing: false,
        };
        if !row.fits() || row.number == u16::MAX || rows.push(row).is_err() {
            problems.push(path, What::Jig(config::What::Bound));
        }
    }
    if problems.is_empty() { Ok(rows.into_boxed()) } else { Err(problems) }
}
fn account(value: &s::Account) -> r::Account {
    r::Account { login: value.login.clone(), email: value.email.clone() }
}
fn origin(value: &s::Origin) -> r::Origin {
    r::Origin { host: value.host.clone(), port: value.port }
}
#[expect(clippy::manual_map, reason = "translations use explicit matches without closures")]
fn owned(value: &s::Owned) -> r::OwnedEntry {
    r::OwnedEntry {
        public: match &value.public {
            Some(value) => Some(origin(value)),
            None => None,
        },
        fetch: r::Fetch {
            forgejo: value.fetch.forgejo,
            runner: value.fetch.runner,
            mirror: match &value.fetch.mirror {
                Some(value) => Some(r::Mirror { forgejo: origin(&value.forgejo), runner: origin(&value.runner) }),
                None => None,
            },
        },
        forgejo: value.forgejo.clone(),
        runner: match &value.runner {
            Some(value) => Some(r::Runner {
                binary: value.binary.clone(),
                labels: value.labels.clone(),
                capacity: value.capacity,
                path: value.path.clone(),
            }),
            None => None,
        },
        listen: value.listen,
        person: account(&value.person),
    }
}
fn entry(value: &s::Entry) -> r::Setup {
    r::Setup {
        directory: value.directory.clone(),
        bot: account(&value.bot),
        orgs: value.orgs.clone(),
        asked: value.asked,
        mode: match &value.mode {
            s::Mode::Managed { admin_token } => r::SetupMode::Managed { admin_token: admin_token.clone() },
            s::Mode::Owned(value) => r::SetupMode::Owned(owned(value)),
            s::Mode::Fake(value) => {
                let mut repositories =
                    List::with_capacity(u32::try_from(value.repositories.len()).expect("fake repositories"));
                for repository in &value.repositories {
                    repositories
                        .push(r::FakeRepository { name: repository.name.clone(), landing: repository.landing.clone() })
                        .expect("fake repositories");
                }
                r::SetupMode::Fake(r::FakeEntry {
                    listen: value.listen,
                    git: value.git.clone(),
                    owner: value.owner.clone(),
                    repositories: repositories.into_boxed(),
                })
            }
        },
    }
}
