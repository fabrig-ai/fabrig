//! Reference client UI words, owner, regions and styles (jig/docs/design/client/domain/view-crates.md, 2.4).

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

pub mod owner;
pub mod regions;
pub mod styles;

pub mod words {
    //! Phrases and each part's vocabulary (jig/docs/design/client/domain/view.md, 7).
    pub mod activity_words;
    pub mod amendment_words;
    pub mod authority_words;
    pub mod budget_words;
    pub mod chip_words;
    pub mod deployment_words;
    pub mod document_words;
    pub mod ended_pages_words;
    pub mod ended_words;
    pub mod fragments;
    pub mod history_words;
    pub mod inbox_words;
    pub mod lifecycle_words;
    pub mod literal_segment;
    pub mod live_call_words;
    pub mod outcome_words;
    pub mod person_task_words;
    pub mod phrases;
    pub mod project_words;
    pub mod proposal_words;
    pub mod proposed_task_words;
    pub mod result_words;
    pub mod tree_words;
    pub mod wake_words;
    #[expect(clippy::module_inception, reason = "words.rs keeps its role inside the words namespace (view.md, 2)")]
    pub(crate) mod words;
    pub use words::{Words, name};
}

pub use owner::Owner;

#[cfg(test)]
mod tests;
