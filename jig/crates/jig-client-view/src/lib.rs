//! jig's optional reference UI on the public client (jig/docs/design/client/domain/view.md, 1).
//! Holds presentation state, builds semantic trees and routes acts through bindings,
//! with default pages composed from jig's parts on skein-ui.

#![expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

pub use jig_client_view_codec::SCHEMA;


mod binding;
pub mod boundary;
mod generated {
    //! Stored reference UI records (jig/docs/design/client/domain/view-state.md, 8).
    pub(crate) use jig_client_view_codec::records_v1;
}

use jig_client_view_words::owner;
pub mod sizes {
    //! Tree envelopes (jig/docs/design/client/domain/view.md, 10, and jig/docs/design/client/domain/view-state.md, 9).
    pub mod limits;
    #[expect(clippy::module_inception, reason = "sizes.rs keeps its role inside the sizes namespace (view.md, 2)")]
    pub(crate) mod sizes;
}

pub mod state {
    //! The reference UI's state (jig/docs/design/client/domain/view-state.md).
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

pub use jig_client_view_words::{regions, styles, words};

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
