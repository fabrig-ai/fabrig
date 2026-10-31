//! Forge and playground records (temper/docs/design/draft/playground-reset.md, 3, 4 and 6.2).
use super::{FITS, d, w};

#[expect(clippy::manual_map, reason = "step translations use explicit matches without closures")]
pub(super) fn encode_forge(row: &d::ForgeRecord) -> w::ForgeRecord {
    let first = match &row.first {
        None => None,
        Some(first) => Some(
            w::FirstIntent::new(
                &w::CEILINGS,
                w::FirstIntentParts {
                    playground: first.playground,
                    org: first.org.clone(),
                    repository: first.repository.clone(),
                    label: first.label.clone(),
                    project: match &first.project {
                        d::SetUpProject::Existing(number) => w::SetUpProject::Existing(
                            w::ExistingProject::new(&w::CEILINGS, w::ExistingProjectParts { number: *number })
                                .expect(FITS),
                        ),
                        d::SetUpProject::New(name) => w::SetUpProject::New(
                            w::FirstProject::new(&w::CEILINGS, w::FirstProjectParts { name: name.clone() })
                                .expect(FITS),
                        ),
                    },
                },
            )
            .expect(FITS),
        ),
    };
    w::ForgeRecord::new(
        &w::CEILINGS,
        w::ForgeRecordParts {
            number: row.number,
            kind: match row.kind {
                d::Kind::Forgejo => w::ForgeKind::Forgejo,
                d::Kind::GitHub => w::ForgeKind::GitHub,
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
pub(super) fn decode_forge(row: w::ForgeRecord) -> d::ForgeRecord {
    let w::ForgeRecordParts { number, kind, mode, removing, definition, first } = row.into_parts();
    let first = match first {
        None => None,
        Some(first) => {
            let w::FirstIntentParts { playground, org, repository, project, label } = first.into_parts();
            Some(d::FirstIntent {
                playground,
                org,
                repository,
                label,
                project: match project {
                    w::SetUpProject::Existing(row) => d::SetUpProject::Existing(row.into_parts().number),
                    w::SetUpProject::New(row) => d::SetUpProject::New(row.into_parts().name),
                },
            })
        }
    };
    d::ForgeRecord {
        number,
        kind: match kind {
            w::ForgeKind::Forgejo => d::Kind::Forgejo,
            w::ForgeKind::GitHub => d::Kind::GitHub,
        },
        mode: match mode {
            w::ForgeMode::Connected => d::Mode::Connected,
            w::ForgeMode::Managed => d::Mode::Managed,
            w::ForgeMode::Owned => d::Mode::Owned,
            w::ForgeMode::Fake => d::Mode::Fake,
        },
        removing,
        definition,
        first,
    }
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
            w::KeptActKind::Cancel(w::CancelAct::new(&w::CEILINGS, w::CancelActParts { project, task }).expect(FITS))
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
            let w::CancelActParts { project, task } = row.into_parts();
            d::KeptActKind::Cancel { project, task }
        }
        w::KeptActKind::RemoveGroup => d::KeptActKind::RemoveGroup,
        w::KeptActKind::EraseSecrets => d::KeptActKind::EraseSecrets,
    };
    d::KeptAct { key, forge, act }
}
