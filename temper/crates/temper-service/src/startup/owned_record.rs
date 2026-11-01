//! The machine's owned forge record (temper/docs/plan/playground-reset-r4/notes/contracts.md, 6;
//! temper/docs/design/protocol/setup.md, 8.1).
use alloc::{boxed::Box, format};
use jig_protocol_config::{self as config, Path, Problems};
use temper_engine_domain_forge::{Forge, Kind, record as r};
use temper_forge_setup::setup as s;

#[expect(clippy::disallowed_macros, reason = "checked startup renders the socket address as an HTTP git base")]
pub(crate) fn record(template: &s::Template) -> Result<Forge, Problems> {
    let entry = template.entry(0);
    let s::Mode::Owned(value) = &entry.mode else {
        return Err(problem(config::What::Form));
    };
    let record = Forge {
        number: entry.forge,
        kind: Kind::Forgejo,
        api: r::Address::Socket(value.listen),
        transport: r::Transport::Plaintext,
        git: format!("http://{}/", value.listen).into_bytes().into_boxed_slice(),
        git_program: None,
        web: None,
        read_only: false,
        identity: r::Identity {
            number: 0,
            login: entry.bot.login.clone(),
            name: Box::from(b"Temper".as_slice()),
            email: entry.bot.email.clone(),
        },
        files: None,
        setup: Some(r::Setup {
            directory: entry.directory,
            bot: account(&entry.bot),
            orgs: entry.orgs,
            mode: r::SetupMode::Owned(owned(value)),
            asked: true,
        }),
        first: None,
        playground: false,
        removing: false,
    };
    if record.fits() { Ok(record) } else { Err(problem(config::What::Bound)) }
}
fn problem(what: config::What) -> Problems {
    let mut problems = Problems::empty();
    problems.push(Path::root().member(b"application").member(b"owned"), what);
    problems
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
