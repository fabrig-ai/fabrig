//! jig's optional reference UI on the public client (jig/docs/design/client/domain/view.md, 1).
//! Holds presentation state, builds semantic trees and routes acts through bindings,
//! with default pages composed from jig's parts on skein-ui.

#![expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

mod binding;
pub mod boundary;
mod generated {
    //! Stored reference UI records (jig/docs/design/client/domain/view.md, 18.8).
    #[rustfmt::skip]
    #[expect(
        dead_code, clippy::unnecessary_wraps, clippy::ref_option,
        reason = "the generated record schema supplies uniform constructors, accessors and bounds beyond the view codec's calls"
    )]
    pub(crate) mod records_v1;
}

mod owner;
pub mod regions;
pub mod sizes {
    //! Tree envelopes (jig/docs/design/client/domain/view.md, 10 and 18.9).
    pub mod limits;
    #[expect(clippy::module_inception, reason = "sizes.rs keeps its role inside the sizes namespace (view.md, 2)")]
    pub(crate) mod sizes;
}

pub mod state {
    //! The reference UI's state (jig/docs/design/client/domain/view.md, 18).
    pub mod action;
    pub(crate) mod activity;
    pub(crate) mod acts;
    pub mod address;
    pub mod application;
    pub(crate) mod clock;
    pub(crate) mod confirm;
    pub(crate) mod drafts;
    pub mod fields;
    pub(crate) mod handles;
    pub(crate) mod intents;
    pub mod objects;
    pub(crate) mod outcome;
    pub mod page;
    pub mod queries;
    pub mod records;
    pub mod signin;
    pub mod store;
    pub(crate) mod ui;
    pub(crate) mod windows;
}

pub mod styles;
pub mod tree {
    //! Node builders (jig/docs/design/client/domain/view.md, 4).
    pub mod cards;
    pub(crate) mod confirmation;
    pub mod fragments;
    pub mod frame;
    pub(crate) mod notices;
    pub mod pages;
    pub mod sections;
    pub(crate) mod turns;
}

pub mod words {
    //! Phrases and each part's vocabulary (jig/docs/design/client/domain/view.md, 7).
    pub(crate) mod activity_words;
    pub(crate) mod amendment_words;
    pub(crate) mod authority_words;
    pub(crate) mod budget_words;
    pub(crate) mod chip_words;
    pub(crate) mod deployment_words;
    pub mod document_words;
    pub(crate) mod ended_pages_words;
    pub(crate) mod ended_words;
    pub(crate) mod history_words;
    pub(crate) mod inbox_words;
    pub mod literal_segment;
    pub(crate) mod live_call_words;
    pub(crate) mod milestone_words;
    pub(crate) mod outcome_words;
    pub(crate) mod person_task_words;
    pub mod phrases;
    pub(crate) mod project_words;
    pub(crate) mod proposal_words;
    pub(crate) mod proposed_task_words;
    pub(crate) mod result_words;
    pub(crate) mod tree_words;
    pub(crate) mod wake_words;
    #[expect(clippy::module_inception, reason = "words.rs keeps its role inside the words namespace (view.md, 2)")]
    pub(crate) mod words;
    pub use words::{Words, name};
}

pub use binding::{JigBindings, JigMeaning, decode};
pub use boundary::{Event, How, Refused, Request};
pub use owner::Owner;
pub use sizes::limits;
pub use sizes::limits::ui::{Config, valid};
pub use sizes::sizes::{
    arranged_nodes, banner_nodes, card_nodes, chip_nodes, confirmation_nodes, conversation_nodes, conversations_nodes,
    deployment_nodes, escalation_nodes, go_to_task_nodes, inbox_nodes, link_nodes, navigation_nodes, needs_nodes,
    not_the_work_nodes, notices_nodes, offered_nodes, page_nodes, party_nodes, person_task_nodes, project_header_nodes,
    project_nodes, project_section_nodes, projects_nodes, proposal_nodes, question_nodes, refusal_nodes, reply_nodes,
    result_nodes, task_header_nodes, task_nodes, task_section_nodes,
};
pub use state::action;
pub use state::address;
pub use state::application;
pub use state::fields;
pub use state::objects;
pub use state::page;
pub use state::queries;
pub use state::records;
pub use state::signin;
pub use state::store;
pub use state::ui::{Borrowed, Ui, fire, next_deadline, reclaim, step};
pub use styles::JigStyle;
pub use tree::cards;
pub use tree::fragments::authority;
pub use tree::fragments::budget;
pub use tree::frame;
pub use tree::pages;
pub use tree::pages::{after_page, jig};
pub use tree::sections;
pub use tree::turns::conversation;
pub use words::document_words;
pub use words::literal_segment;
pub use words::phrases;

#[cfg(test)]
mod tests;
