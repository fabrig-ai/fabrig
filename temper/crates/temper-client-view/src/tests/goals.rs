use super::fixtures;

use crate::{
    frame::Bindings,
    look,
    pages::goals,
    words::{Times, Words},
};

use jig_client::contract as c;

use skein_lib::{List, Queue, Wall};

use skein_ui::{Builder, Role, Style, Tree};

use temper_client_domain::{
    Address, GoalsFilter,
    pages::{goals::State, report::Report},
};


// The mock's row has six landed tasks out of nine (client/pages/README.md).

pub(super) use super::fixtures::goals::{TEXT_BYTES, TOTAL, line, template};
use crate::parts_tests::goals::{EMPTY};
use crate::work_tests::goals::{CLOSED_TASK, CURSOR_BYTES, FAILED_TASK, FUNDS_TASK, LANDED_TASK, LIVE_ROWS, NEEDS_TASK, ONE, PAGE_ROWS, PAUSED_TASK, PLANNING_TASK, Page, RUNNING_TASK, live_lines, page_line, tree};


#[test]
fn an_unanswered_goals_page_claims_no_empty() {
    let mut page = Page::new();
    page.live(&[], false);
    for view in [fixtures::page_view(&page.root), fixtures::browser_page_view(&page.root)] {
        let main = fixtures::main_text(view.tree());
        for name in [b"No goals yet.".as_slice(), b"None."] {
            assert!(skein_lib::bytes::find(&main, name).is_none());
        }
        for node in view.tree().nodes() {
            assert_ne!(node.role, Role::Heading { level: 1 });
        }
    }
}

#[test]
fn a_lost_ended_read_keeps_the_ended_rows() {
    let mut page = Page::new();
    let ended = page_line(RUNNING_TASK, State::Closed);
    page.live(&[], false);
    page.ended(core::slice::from_ref(&ended), false);
    page.live(&[], false);
    let read = page.read(true);
    page.up(jig_client::Up::ReadDone {
        read,
        result: jig_client::Readout::Jig(c::ReadResult::None(c::ReadNone::Unreachable)),
    });
    for view in [fixtures::page_view(&page.root), fixtures::browser_page_view(&page.root)] {
        assert!(skein_lib::bytes::find(&fixtures::main_text(view.tree()), ended.chip.title.bytes()).is_some());
    }
}
