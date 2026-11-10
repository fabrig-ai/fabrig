//! Bindings and acts (skein's ui/README.md, section 6).

use crate::Role;
use alloc::boxed::Box;
use skein_lib::Token;

/// A meaning kept by its owner's view for the last built tree.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Binding {
    pub owner: crate::tree::Owner,
    pub token: Token,
}

/// The shell's act, with a field's whole value owned by the event.
#[derive(PartialEq, Eq, Debug)]
pub enum Act {
    /// Press a button or follow a link.
    Press,
    /// Replace a field's typed value.
    Input { value: Box<[u8]> },
    /// Submit a form or composer.
    Submit,
    /// Toggle a choice or check.
    Toggle,
    /// Bytes of the drawing's owner, reported by its drawing code in a shell.
    Drawing { data: Box<[u8]> },
    /// A bound log's edge or a bound list's foot has come near the person's view.
    Near { edge: LogEdge },
}

/// A log's edges; a list reports only its foot (ui/README.md, section 6).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LogEdge {
    Head,
    Foot,
}

/// An act resolved to the owner that decodes it into a typed domain action.
#[derive(PartialEq, Eq, Debug)]
pub struct BoundAct {
    pub binding: Binding,
    pub act: Act,
}

/// Whether a bound role takes this act (ui/README.md, section 6).
#[must_use]
pub fn takes(role: Role, submits_on_enter: bool, act: &Act) -> bool {
    match role {
        Role::TextField | Role::TextArea => match act {
            Act::Input { .. } => true,
            Act::Submit => submits_on_enter,
            Act::Press | Act::Toggle | Act::Drawing { .. } | Act::Near { .. } => false,
        },
        Role::Check | Role::Choice => match act {
            Act::Toggle => true,
            Act::Press | Act::Input { .. } | Act::Submit | Act::Drawing { .. } | Act::Near { .. } => false,
        },
        Role::Drawing => match act {
            Act::Drawing { .. } => true,
            Act::Press | Act::Input { .. } | Act::Submit | Act::Toggle | Act::Near { .. } => false,
        },
        Role::Button | Role::Link | Role::Tab | Role::MenuItem | Role::Option => match act {
            Act::Press => true,
            Act::Input { .. } | Act::Submit | Act::Toggle | Act::Drawing { .. } | Act::Near { .. } => false,
        },
        Role::Log => match act {
            Act::Near { .. } => true,
            Act::Press | Act::Input { .. } | Act::Submit | Act::Toggle | Act::Drawing { .. } => false,
        },
        Role::List => match act {
            Act::Near { edge: LogEdge::Foot } => true,
            Act::Near { edge: LogEdge::Head }
            | Act::Press
            | Act::Input { .. }
            | Act::Submit
            | Act::Toggle
            | Act::Drawing { .. } => false,
        },
        Role::Page
        | Role::Banner
        | Role::Navigation
        | Role::Main
        | Role::Complementary
        | Role::ContentInfo
        | Role::Form
        | Role::Search
        | Role::Region
        | Role::Heading { .. }
        | Role::Paragraph
        | Role::Text
        | Role::Item
        | Role::Table
        | Role::Row
        | Role::Cell
        | Role::ColumnHeader
        | Role::RowHeader
        | Role::Select
        | Role::Term
        | Role::Description
        | Role::ChoiceGroup
        | Role::Dialog
        | Role::Status
        | Role::Alert
        | Role::Tree
        | Role::TreeItem
        | Role::Tabs
        | Role::Progress
        | Role::Time
        | Role::Code
        | Role::Quote
        | Role::Separator
        | Role::Menu
        | Role::Disclosure
        | Role::Image
        | Role::Figure
        | Role::Group
        | Role::Slot => false,
    }
}

/// A field's ASCII digits as a count; empty, non-digits and overflow are absent.
#[must_use]
pub fn number(bytes: &[u8]) -> Option<u64> {
    skein_lib::bytes::decimal(bytes)
}
