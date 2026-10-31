//! Durable forge and playground rows (temper/docs/design/draft/playground-reset.md, 3, 4 and 6.1).
use super::{FITS, w};
use temper_engine_domain_forge as d;

#[expect(clippy::manual_map, reason = "step translations use explicit matches without closures")]
pub(super) fn encode(row: &d::ForgeRecord) -> w::Forge {
    let d::ForgeRecord { number, kind, mode, state, place, read_only, bot, asked, first } = row;
    w::Forge::new(
        &w::CEILINGS,
        w::ForgeParts {
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
                d::ForgeState::Ended { secrets_erased, box_removed } => w::ForgeState::Ended(
                    w::EndedForge::new(
                        &w::CEILINGS,
                        w::EndedForgeParts { secrets_erased: *secrets_erased, box_removed: *box_removed },
                    )
                    .expect(FITS),
                ),
            },
            place: encode_place(place),
            read_only: *read_only,
            bot: encode_bot(bot),
            asked: *asked,
            first: match first {
                None => None,
                Some(first) => Some(encode_first(first)),
            },
        },
    )
    .expect(FITS)
}

fn encode_place(row: &d::Place) -> w::Place {
    let d::Place { api, git, web } = row;
    let d::Address { host, port, tls } = api;
    let api = w::Address::new(&w::CEILINGS, w::AddressParts { host: host.clone(), port: *port, tls: tls.clone() })
        .expect(FITS);
    w::Place::new(&w::CEILINGS, w::PlaceParts { api, git: git.clone(), web: web.clone() }).expect(FITS)
}

fn encode_bot(row: &d::Bot) -> w::Bot {
    let d::Bot { login, name, email } = row;
    w::Bot::new(&w::CEILINGS, w::BotParts { login: login.clone(), name: name.clone(), email: email.clone() })
        .expect(FITS)
}

fn encode_first(row: &d::FirstIntent) -> w::FirstIntent {
    let d::FirstIntent { playground, org, repository, project, label } = row;
    let project = match project {
        d::SetUpProject::Existing(project) => w::FirstProject::Existing(
            w::ExistingFirst::new(&w::CEILINGS, w::ExistingFirstParts { project: *project }).expect(FITS),
        ),
        d::SetUpProject::New(name) => {
            w::FirstProject::New(w::NewFirst::new(&w::CEILINGS, w::NewFirstParts { name: name.clone() }).expect(FITS))
        }
    };
    w::FirstIntent::new(
        &w::CEILINGS,
        w::FirstIntentParts {
            playground: *playground,
            org: org.clone(),
            repository: repository.clone(),
            project,
            label: label.clone(),
        },
    )
    .expect(FITS)
}

#[expect(clippy::manual_map, reason = "step translations use explicit matches without closures")]
pub(super) fn decode(row: w::Forge) -> d::ForgeRecord {
    let w::ForgeParts { number, kind, mode, state, place, read_only, bot, asked, first } = row.into_parts();
    d::ForgeRecord {
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
                let w::EndedForgeParts { secrets_erased, box_removed } = row.into_parts();
                d::ForgeState::Ended { secrets_erased, box_removed }
            }
        },
        place: decode_place(place),
        read_only,
        bot: decode_bot(bot),
        asked,
        first: match first {
            None => None,
            Some(first) => Some(decode_first(first)),
        },
    }
}

fn decode_place(row: w::Place) -> d::Place {
    let w::PlaceParts { api, git, web } = row.into_parts();
    let w::AddressParts { host, port, tls } = api.into_parts();
    d::Place { api: d::Address { host, port, tls }, git, web }
}

fn decode_bot(row: w::Bot) -> d::Bot {
    let w::BotParts { login, name, email } = row.into_parts();
    d::Bot { login, name, email }
}

fn decode_first(row: w::FirstIntent) -> d::FirstIntent {
    let w::FirstIntentParts { playground, org, repository, project, label } = row.into_parts();
    let project = match project {
        w::FirstProject::Existing(row) => {
            let w::ExistingFirstParts { project } = row.into_parts();
            d::SetUpProject::Existing(project)
        }
        w::FirstProject::New(row) => {
            let w::NewFirstParts { name } = row.into_parts();
            d::SetUpProject::New(name)
        }
    };
    d::FirstIntent { playground, org, repository, project, label }
}

pub(super) fn encode_playground(row: &d::Playground) -> w::Playground {
    let d::Playground { state, forge, project, account } = row;
    let state = match state {
        d::PlaygroundState::Starting => w::PlaygroundState::Starting,
        d::PlaygroundState::Running => w::PlaygroundState::Running,
        d::PlaygroundState::Resetting(step) => w::PlaygroundState::Resetting(
            w::Resetting::new(
                &w::CEILINGS,
                w::ResettingParts {
                    step: match step {
                        d::ResetStep::Project => w::ResetStep::Project,
                        d::ResetStep::Forge => w::ResetStep::Forge,
                        d::ResetStep::Account => w::ResetStep::Account,
                    },
                },
            )
            .expect(FITS),
        ),
    };
    w::Playground::new(&w::CEILINGS, w::PlaygroundParts { state, forge: *forge, project: *project, account: *account })
        .expect(FITS)
}

pub(super) fn decode_playground(row: w::Playground) -> d::Playground {
    let w::PlaygroundParts { state, forge, project, account } = row.into_parts();
    let state = match state {
        w::PlaygroundState::Starting => d::PlaygroundState::Starting,
        w::PlaygroundState::Running => d::PlaygroundState::Running,
        w::PlaygroundState::Resetting(row) => {
            let w::ResettingParts { step } = row.into_parts();
            d::PlaygroundState::Resetting(match step {
                w::ResetStep::Project => d::ResetStep::Project,
                w::ResetStep::Forge => d::ResetStep::Forge,
                w::ResetStep::Account => d::ResetStep::Account,
            })
        }
    };
    d::Playground { state, forge, project, account }
}
