//! Locate's owned names and the forge's nonzero repository id (forge/README.md, 3.7).
use crate::api::Answer;
use skein_lib::len_u64;

#[must_use]
pub fn bytes(owner: &[u8], name: &[u8]) -> Option<u64> {
    len_u64(owner.len()).checked_add(len_u64(name.len()))
}

#[must_use]
pub fn accepts(answer: &Answer) -> bool {
    match answer {
        Answer::Located { repository } => *repository != 0,
        Answer::Runs { .. }
        | Answer::Run(_)
        | Answer::Tree { .. }
        | Answer::Commits { .. }
        | Answer::Items { .. }
        | Answer::Item { .. }
        | Answer::Pull(_)
        | Answer::Reviews { .. }
        | Answer::Statuses { .. }
        | Answer::Remarks { .. }
        | Answer::Commit(_)
        | Answer::Branches(_)
        | Answer::PullFiles { .. }
        | Answer::Compare { .. }
        | Answer::Checks(_)
        | Answer::Job { .. }
        | Answer::Directory { .. }
        | Answer::File { .. }
        | Answer::Protection(_)
        | Answer::Settings(_)
        | Answer::Collaborators { .. }
        | Answer::Permission(_)
        | Answer::Created(_)
        | Answer::Commented(_)
        | Answer::Reviewed(_)
        | Answer::Merged(_)
        | Answer::Queued
        | Answer::Branch(_)
        | Answer::Done => false,
    }
}
