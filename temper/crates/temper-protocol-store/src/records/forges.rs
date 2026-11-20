//! Durable forge and playground rows (temper/docs/design/protocol/store.md, 2).
use super::{FITS, Undecodable, w};
use temper_engine_domain_forge_vocabulary as d;

pub(super) fn encode(row: &d::ForgeRecord) -> w::Forge {
    let d::ForgeRecord { number, kind, mode, state, place, git_program, read_only, bot, asked, first, person } = row;
    w::Forge {
        number: *number,
        kind: match kind {
            d::Kind::Forgejo => w::ForgeKind::Forgejo,
            d::Kind::GitHub => w::ForgeKind::Github,
        },
        mode: match mode {
            d::Mode::Connected => w::ForgeMode::Connected,
            d::Mode::Managed => w::ForgeMode::Managed,
            d::Mode::Owned => w::ForgeMode::Owned,
            d::Mode::Fake => w::ForgeMode::Fake,
        },
        state: match state {
            d::ForgeState::Kept => w::ForgeState::Kept,
            d::ForgeState::Ended { secrets_erased, box_removed } => {
                w::ForgeState::Ended(w::EndedForge { secrets_erased: *secrets_erased, box_removed: *box_removed })
            }
        },
        place: encode_place(place),
        git_program: git_program.clone(),
        read_only: *read_only,
        bot: encode_bot(bot),
        asked: *asked,
        first: match first {
            None => None,
            Some(first) => Some(encode_first(first)),
        },
        person: match person {
            None => None,
            Some(link) => Some(w::ForgeLink { user: link.user, person: link.person }),
        },
    }
}

fn encode_place(row: &d::Place) -> w::Place {
    let d::Place { api, git, web } = row;
    let d::Address { host, port, tls } = api;
    let api = w::Address { host: host.clone(), port: *port, tls: tls.clone() };
    w::Place { api, git: git.clone(), web: web.clone() }
}

fn encode_bot(row: &d::Bot) -> w::Bot {
    let d::Bot { login, name, email } = row;
    w::Bot { login: login.clone(), name: name.clone(), email: email.clone() }
}

fn encode_first(row: &d::FirstIntent) -> w::FirstIntent {
    match row {
        d::FirstIntent::Empty(first) => w::FirstIntent::Empty(w::EmptyIntent {
            playground: first.playground,
            sample: first.sample,
            org: first.org.clone(),
            repository: first.repository.clone(),
            label: first.label.clone(),
        }),
        d::FirstIntent::Copies { addresses } => {
            let mut list = skein_lib::List::with_capacity(w::CEILINGS.copy_intent_addresses);
            for address in addresses {
                list.push(address.clone()).expect(FITS);
            }
            w::FirstIntent::Copies(w::CopyIntent { addresses: list })
        }
        d::FirstIntent::Nothing => w::FirstIntent::Nothing,
    }
}

pub(super) fn decode(row: w::Forge) -> Result<d::ForgeRecord, Undecodable> {
    let w::Forge { number, kind, mode, state, place, git_program, read_only, bot, asked, first, person } = row;
    let person = match person {
        None => None,
        Some(link) => {
            let w::ForgeLink { user, person } = link;
            if user == 0 || person == 0 {
                return Err(Undecodable::Malformed);
            }
            Some(d::Link { user, person })
        }
    };
    Ok(d::ForgeRecord {
        number,
        kind: match kind {
            w::ForgeKind::Forgejo => d::Kind::Forgejo,
            w::ForgeKind::Github => d::Kind::GitHub,
        },
        mode: match mode {
            w::ForgeMode::Connected => d::Mode::Connected,
            w::ForgeMode::Managed => d::Mode::Managed,
            w::ForgeMode::Owned => d::Mode::Owned,
            w::ForgeMode::Fake => d::Mode::Fake,
        },
        state: match state {
            w::ForgeState::Kept => d::ForgeState::Kept,
            w::ForgeState::Ended(row) => {
                let w::EndedForge { secrets_erased, box_removed } = row;
                d::ForgeState::Ended { secrets_erased, box_removed }
            }
        },
        place: decode_place(place),
        git_program,
        read_only,
        bot: decode_bot(bot),
        asked,
        first: match first {
            None => None,
            Some(first) => Some(decode_first(first)),
        },
        person,
    })
}

fn decode_place(row: w::Place) -> d::Place {
    let w::Place { api, git, web } = row;
    let w::Address { host, port, tls } = api;
    d::Place { api: d::Address { host, port, tls }, git, web }
}

fn decode_bot(row: w::Bot) -> d::Bot {
    let w::Bot { login, name, email } = row;
    d::Bot { login, name, email }
}

fn decode_first(row: w::FirstIntent) -> d::FirstIntent {
    match row {
        w::FirstIntent::Empty(row) => {
            let w::EmptyIntent { playground, sample, org, repository, label } = row;
            d::FirstIntent::Empty(d::EmptyIntent { playground, sample, org, repository, label })
        }
        w::FirstIntent::Copies(row) => d::FirstIntent::Copies { addresses: row.addresses.into_boxed() },
        w::FirstIntent::Nothing => d::FirstIntent::Nothing,
    }
}

pub(super) fn encode_playground(row: &d::Playground) -> w::Playground {
    let d::Playground { state, forge, project, account } = row;
    let state = match state {
        d::PlaygroundState::Starting => w::PlaygroundState::Starting,
        d::PlaygroundState::Running => w::PlaygroundState::Running,
        d::PlaygroundState::Resetting(step) => w::PlaygroundState::Resetting(w::Resetting {
            step: match step {
                d::ResetStep::Project => w::ResetStep::Project,
                d::ResetStep::Forge => w::ResetStep::Forge,
                d::ResetStep::Account => w::ResetStep::Account,
            },
        }),
    };
    w::Playground { state, forge: *forge, project: *project, account: *account }
}

pub(super) fn decode_playground(row: w::Playground) -> d::Playground {
    let w::Playground { state, forge, project, account } = row;
    let state = match state {
        w::PlaygroundState::Starting => d::PlaygroundState::Starting,
        w::PlaygroundState::Running => d::PlaygroundState::Running,
        w::PlaygroundState::Resetting(row) => {
            let w::Resetting { step } = row;
            d::PlaygroundState::Resetting(match step {
                w::ResetStep::Project => d::ResetStep::Project,
                w::ResetStep::Forge => d::ResetStep::Forge,
                w::ResetStep::Account => d::ResetStep::Account,
            })
        }
    };
    d::Playground { state, forge, project, account }
}
