//! Landed draft family 41 (temper/docs/design/protocol/store.md, 2).
use super::{FITS, Undecodable, names, w};
use skein_lib::Wall;
use temper_engine_domain_forge as d;
pub(super) fn encode(row: &d::LandedDraft) -> w::LandedDraft {
    w::LandedDraft::new(
        &w::CEILINGS,
        w::LandedDraftParts {
            repository: names::encode_repository(row.repository),
            path: row.path.clone(),
            project: row.project,
            branch: row.branch.clone(),
            commit: row.commit,
            title: row.title.clone(),
            writer: row.writer,
            at: row.at.as_nanos(),
            chat: row.chat,
            goal: row.goal,
            ready: row.ready,
        },
    )
    .expect(FITS)
}
pub(super) fn decode(row: w::LandedDraft) -> Result<d::LandedDraft, Undecodable> {
    let w::LandedDraftParts { repository, path, project, branch, commit, title, writer, at, chat, goal, ready } =
        row.into_parts();
    if !d::draft_path(&path) {
        return Err(Undecodable);
    }
    Ok(d::LandedDraft {
        repository: names::decode_repository(repository),
        path,
        project,
        branch,
        commit,
        title,
        writer,
        at: Wall::from_nanos(at),
        chat,
        goal,
        ready,
    })
}
