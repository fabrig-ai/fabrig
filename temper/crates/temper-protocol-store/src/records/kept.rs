//! Forge records and removal progress (temper/docs/design/draft/playground-reset.md, 3, 4 and 6.2).
use super::{FITS, d, w};

#[expect(clippy::manual_map, reason = "step translations use explicit matches without closures")]
pub(super) fn encode_forge(row: &d::ForgeRecord) -> w::ForgeRecord {
    let first = match &row.first {
        Some(first) => Some(encode_first(first)),
        None => None,
    };
    w::ForgeRecord::new(
        &w::CEILINGS,
        w::ForgeRecordParts {
            number: row.number,
            kind: match row.kind {
                d::Kind::GitHub => w::ForgeKind::Github,
                d::Kind::Forgejo => w::ForgeKind::Forgejo,
            },
            mode: match row.mode {
                d::Mode::Connected => w::ForgeMode::Connected,
                d::Mode::Managed => w::ForgeMode::Managed,
                d::Mode::Owned => w::ForgeMode::Owned,
                d::Mode::Fake => w::ForgeMode::Fake,
            },
            removing: row.removing,
            definition: row.definition.clone(),
            first,
        },
    )
    .expect(FITS)
}

fn encode_first(row: &d::FirstIntent) -> w::FirstIntent {
    let project = match &row.project {
        d::SetUpProject::Existing(project) => w::FirstProject::Existing(
            w::ExistingProject::new(&w::CEILINGS, w::ExistingProjectParts { project: *project }).expect(FITS),
        ),
        d::SetUpProject::New(name) => w::FirstProject::New(
            w::NewProject::new(&w::CEILINGS, w::NewProjectParts { name: name.clone() }).expect(FITS),
        ),
    };
    w::FirstIntent::new(
        &w::CEILINGS,
        w::FirstIntentParts {
            playground: row.playground,
            org: row.org.clone(),
            repository: row.repository.clone(),
            project,
            label: row.label.clone(),
        },
    )
    .expect(FITS)
}

#[expect(clippy::manual_map, reason = "step translations use explicit matches without closures")]
pub(super) fn decode_forge(row: w::ForgeRecord) -> d::ForgeRecord {
    let w::ForgeRecordParts { number, kind, mode, removing, definition, first } = row.into_parts();
    d::ForgeRecord {
        number,
        kind: match kind {
            w::ForgeKind::Github => d::Kind::GitHub,
            w::ForgeKind::Forgejo => d::Kind::Forgejo,
        },
        mode: match mode {
            w::ForgeMode::Connected => d::Mode::Connected,
            w::ForgeMode::Managed => d::Mode::Managed,
            w::ForgeMode::Owned => d::Mode::Owned,
            w::ForgeMode::Fake => d::Mode::Fake,
        },
        removing,
        definition,
        first: match first {
            Some(first) => Some(decode_first(first)),
            None => None,
        },
    }
}

fn decode_first(row: w::FirstIntent) -> d::FirstIntent {
    let w::FirstIntentParts { playground, org, repository, project, label } = row.into_parts();
    let project = match project {
        w::FirstProject::Existing(row) => d::SetUpProject::Existing(row.into_parts().project),
        w::FirstProject::New(row) => d::SetUpProject::New(row.into_parts().name),
    };
    d::FirstIntent { playground, org, repository, project, label }
}

pub(super) fn encode_playground(row: &d::Playground) -> w::Playground {
    let state = match row.state {
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
    w::Playground::new(
        &w::CEILINGS,
        w::PlaygroundParts { state, forge: row.forge, project: row.project, account: row.account },
    )
    .expect(FITS)
}

pub(super) fn decode_playground(row: w::Playground) -> d::Playground {
    let w::PlaygroundParts { state, forge, project, account } = row.into_parts();
    let state = match state {
        w::PlaygroundState::Running => d::PlaygroundState::Running,
        w::PlaygroundState::Resetting(row) => d::PlaygroundState::Resetting(match row.into_parts().step {
            w::ResetStep::Project => d::ResetStep::Project,
            w::ResetStep::Forge => d::ResetStep::Forge,
            w::ResetStep::Account => d::ResetStep::Account,
        }),
    };
    d::Playground { state, forge, project, account }
}

pub(super) fn encode_act(row: &d::KeptAct) -> w::KeptAct {
    let act = match row.act {
        d::KeptActKind::Cancel { project, task } => {
            w::KeptActKind::Cancel(w::KeptCancel::new(&w::CEILINGS, w::KeptCancelParts { project, task }).expect(FITS))
        }
        d::KeptActKind::RemoveGroup => w::KeptActKind::RemoveGroup,
        d::KeptActKind::EraseSecrets => w::KeptActKind::EraseSecrets,
    };
    w::KeptAct::new(&w::CEILINGS, w::KeptActParts { key: row.key, forge: row.forge, act }).expect(FITS)
}

pub(super) fn decode_act(row: w::KeptAct) -> d::KeptAct {
    let w::KeptActParts { key, forge, act } = row.into_parts();
    let act = match act {
        w::KeptActKind::Cancel(row) => {
            let w::KeptCancelParts { project, task } = row.into_parts();
            d::KeptActKind::Cancel { project, task }
        }
        w::KeptActKind::RemoveGroup => d::KeptActKind::RemoveGroup,
        w::KeptActKind::EraseSecrets => d::KeptActKind::EraseSecrets,
    };
    d::KeptAct { key, forge, act }
}
