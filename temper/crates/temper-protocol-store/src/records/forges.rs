//! Forge records, playground rows and removal acts (temper/docs/design/draft/playground-reset.md, 3, 4, 6).
use super::{FITS, w};
use temper_engine_domain_forge as d;

#[expect(clippy::manual_map, reason = "explicit option matches follow programming-model.md, 10")]
pub(super) fn encode(value: &d::ForgeRecord) -> w::Forge {
    let d::ForgeRecord {
        number,
        kind,
        mode,
        api,
        git,
        git_program,
        web,
        read_only,
        bot,
        setup,
        first,
        playground,
        state,
    } = value;
    let d::Api { host, port, transport } = api;
    let transport = match transport {
        d::ApiTransport::Plaintext => w::ApiTransport::Plaintext,
        d::ApiTransport::Tls { server_name, trust } => w::ApiTransport::Tls(
            w::ApiTls::new(&w::CEILINGS, w::ApiTlsParts { server_name: server_name.clone(), trust: trust.clone() })
                .expect(FITS),
        ),
    };
    let d::Bot { login, name, email } = bot;
    let first = match first {
        Some(first) => Some(encode_first(first)),
        None => None,
    };
    w::Forge::new(
        &w::CEILINGS,
        w::ForgeParts {
            number: *number,
            kind: encode_kind(*kind),
            mode: encode_mode(*mode),
            api: w::ForgeApi::new(&w::CEILINGS, w::ForgeApiParts { host: host.clone(), port: *port, transport })
                .expect(FITS),
            git: git.clone(),
            git_program: git_program.clone(),
            web: web.clone(),
            read_only: *read_only,
            bot: w::ForgeBot::new(
                &w::CEILINGS,
                w::ForgeBotParts { login: login.clone(), name: name.clone(), email: email.clone() },
            )
            .expect(FITS),
            setup: setup.clone(),
            first,
            playground: *playground,
            state: encode_state(*state),
        },
    )
    .expect(FITS)
}
#[expect(clippy::manual_map, reason = "explicit option matches follow programming-model.md, 10")]
pub(super) fn decode(value: w::Forge) -> d::ForgeRecord {
    let w::ForgeParts {
        number,
        kind,
        mode,
        api,
        git,
        git_program,
        web,
        read_only,
        bot,
        setup,
        first,
        playground,
        state,
    } = value.into_parts();
    let w::ForgeApiParts { host, port, transport } = api.into_parts();
    let transport = match transport {
        w::ApiTransport::Plaintext => d::ApiTransport::Plaintext,
        w::ApiTransport::Tls(value) => {
            let w::ApiTlsParts { server_name, trust } = value.into_parts();
            d::ApiTransport::Tls { server_name, trust }
        }
    };
    let w::ForgeBotParts { login, name, email } = bot.into_parts();
    let first = match first {
        Some(first) => Some(decode_first(first)),
        None => None,
    };
    d::ForgeRecord {
        number,
        kind: decode_kind(kind),
        mode: decode_mode(mode),
        api: d::Api { host, port, transport },
        git,
        git_program,
        web,
        read_only,
        bot: d::Bot { login, name, email },
        setup,
        first,
        playground,
        state: decode_state(state),
    }
}
fn encode_first(value: &d::First) -> w::ForgeFirst {
    let d::First { playground, org, repository, project, state, label } = value;
    let project = match project {
        d::SetUpProject::Existing(project) => w::FirstProject::Existing(
            w::FirstExisting::new(&w::CEILINGS, w::FirstExistingParts { project: *project }).expect(FITS),
        ),
        d::SetUpProject::New(name) => {
            w::FirstProject::New(w::FirstNew::new(&w::CEILINGS, w::FirstNewParts { name: name.clone() }).expect(FITS))
        }
    };
    let state = match state {
        d::FirstState::Making => w::FirstState::Making,
        d::FirstState::Made { id } => {
            w::FirstState::Made(w::FirstMadeId::new(&w::CEILINGS, w::FirstMadeIdParts { id: *id }).expect(FITS))
        }
        d::FirstState::Failed { step } => w::FirstState::Failed(
            w::FirstFailedStep::new(&w::CEILINGS, w::FirstFailedStepParts { step: encode_create(*step) }).expect(FITS),
        ),
    };
    w::ForgeFirst::new(
        &w::CEILINGS,
        w::ForgeFirstParts {
            playground: *playground,
            org: org.clone(),
            repository: repository.clone(),
            project,
            state,
            label: label.clone(),
        },
    )
    .expect(FITS)
}
fn decode_first(value: w::ForgeFirst) -> d::First {
    let w::ForgeFirstParts { playground, org, repository, project, state, label } = value.into_parts();
    let project = match project {
        w::FirstProject::Existing(value) => d::SetUpProject::Existing(value.project()),
        w::FirstProject::New(value) => d::SetUpProject::New(value.into_parts().name),
    };
    let state = match state {
        w::FirstState::Making => d::FirstState::Making,
        w::FirstState::Made(value) => d::FirstState::Made { id: value.id() },
        w::FirstState::Failed(value) => d::FirstState::Failed { step: decode_create(value.into_parts().step) },
    };
    d::First { playground, org, repository, project, state, label }
}
pub(super) fn encode_playground(value: d::Playground) -> w::Playground {
    let d::Playground { state, forge, project, account } = value;
    let state = match state {
        d::PlaygroundState::Running => w::PlaygroundState::Running,
        d::PlaygroundState::Resetting(step) => w::PlaygroundState::Resetting(
            w::PlaygroundResetting::new(&w::CEILINGS, w::PlaygroundResettingParts { step: encode_reset(step) })
                .expect(FITS),
        ),
    };
    w::Playground::new(&w::CEILINGS, w::PlaygroundParts { state, forge, project, account }).expect(FITS)
}
pub(super) fn decode_playground(value: w::Playground) -> d::Playground {
    let w::PlaygroundParts { state, forge, project, account } = value.into_parts();
    let state = match state {
        w::PlaygroundState::Running => d::PlaygroundState::Running,
        w::PlaygroundState::Resetting(value) => d::PlaygroundState::Resetting(decode_reset(value.into_parts().step)),
    };
    d::Playground { state, forge, project, account }
}
pub(super) fn encode_kept(value: d::KeptAct) -> w::KeptAct {
    let d::KeptAct { key, forge, act } = value;
    let act = match act {
        d::KeptActKind::Cancel { project, task } => {
            w::KeptActKind::Cancel(w::KeptCancel::new(&w::CEILINGS, w::KeptCancelParts { project, task }).expect(FITS))
        }
        d::KeptActKind::RemoveGroup => w::KeptActKind::RemoveGroup,
        d::KeptActKind::EraseSecrets => w::KeptActKind::EraseSecrets,
    };
    w::KeptAct::new(&w::CEILINGS, w::KeptActParts { key, forge, act }).expect(FITS)
}
pub(super) fn decode_kept(value: w::KeptAct) -> d::KeptAct {
    let w::KeptActParts { key, forge, act } = value.into_parts();
    let act = match act {
        w::KeptActKind::Cancel(value) => {
            let w::KeptCancelParts { project, task } = value.into_parts();
            d::KeptActKind::Cancel { project, task }
        }
        w::KeptActKind::RemoveGroup => d::KeptActKind::RemoveGroup,
        w::KeptActKind::EraseSecrets => d::KeptActKind::EraseSecrets,
    };
    d::KeptAct { key, forge, act }
}
fn encode_kind(value: d::Kind) -> w::ForgeKind {
    match value {
        d::Kind::Forgejo => w::ForgeKind::Forgejo,
        d::Kind::GitHub => w::ForgeKind::Github,
    }
}
fn decode_kind(value: w::ForgeKind) -> d::Kind {
    match value {
        w::ForgeKind::Forgejo => d::Kind::Forgejo,
        w::ForgeKind::Github => d::Kind::GitHub,
    }
}
fn encode_mode(value: d::Mode) -> w::ForgeMode {
    match value {
        d::Mode::Connected => w::ForgeMode::Connected,
        d::Mode::Managed => w::ForgeMode::Managed,
        d::Mode::Owned => w::ForgeMode::Owned,
        d::Mode::Fake => w::ForgeMode::Fake,
    }
}
fn decode_mode(value: w::ForgeMode) -> d::Mode {
    match value {
        w::ForgeMode::Connected => d::Mode::Connected,
        w::ForgeMode::Managed => d::Mode::Managed,
        w::ForgeMode::Owned => d::Mode::Owned,
        w::ForgeMode::Fake => d::Mode::Fake,
    }
}
fn encode_state(value: d::ForgeState) -> w::ForgeState {
    match value {
        d::ForgeState::Kept => w::ForgeState::Kept,
        d::ForgeState::Removing => w::ForgeState::Removing,
    }
}
fn decode_state(value: w::ForgeState) -> d::ForgeState {
    match value {
        w::ForgeState::Kept => d::ForgeState::Kept,
        w::ForgeState::Removing => d::ForgeState::Removing,
    }
}
fn encode_create(value: d::CreateStep) -> w::FirstCreateStep {
    match value {
        d::CreateStep::Admin => w::FirstCreateStep::Admin,
        d::CreateStep::Org => w::FirstCreateStep::Org,
        d::CreateStep::Repository => w::FirstCreateStep::Repository,
        d::CreateStep::File => w::FirstCreateStep::File,
    }
}
fn decode_create(value: w::FirstCreateStep) -> d::CreateStep {
    match value {
        w::FirstCreateStep::Admin => d::CreateStep::Admin,
        w::FirstCreateStep::Org => d::CreateStep::Org,
        w::FirstCreateStep::Repository => d::CreateStep::Repository,
        w::FirstCreateStep::File => d::CreateStep::File,
    }
}
fn encode_reset(value: d::ResetStep) -> w::ResetStep {
    match value {
        d::ResetStep::Project => w::ResetStep::Project,
        d::ResetStep::Forge => w::ResetStep::Forge,
        d::ResetStep::Account => w::ResetStep::Account,
    }
}
fn decode_reset(value: w::ResetStep) -> d::ResetStep {
    match value {
        w::ResetStep::Project => d::ResetStep::Project,
        w::ResetStep::Forge => d::ResetStep::Forge,
        w::ResetStep::Account => d::ResetStep::Account,
    }
}
