use super::{
    chats,
    fixtures::{self, task as f},
};

use crate::{
    frame::Bindings,
    look,
    pages::goal,
    words::{Times, Words},
};

use jig_client::{Down, Event as ClientEvent, Readable, Readout, UnlinkedRequest, Up, contract as c};

use skein_lib::{Env, List, Queue, Time, Wall};

use skein_ui::{Builder, Role, Run, Style, Tree};

use temper_client_domain::{Address, Domain, Event, Request, pages::Act};


pub(super) use super::fixtures::goal::{
    DONE, DRAFT_WORDS, MADE, MINUTE, NOW, Side, TASK, TOTAL, document, offers, paused_document, press, question, root,
    root_with_side, sent_attempt, text, with_goal_answer,
};

#[test]
fn an_agents_close_uses_the_same_sheet() {
    let mut task = f::delegate();
    task.acts = offers(&[c::ActKind::Cancel]);
    let mut root = f::agent_root(task, Some(f::tree()), true);
    press(&mut root, Act::Close);
    let view = fixtures::browser_page_view(&root);
    assert!(fixtures::find(view.tree(), Role::Dialog, b"Close this task?").is_some());
    assert!(
        skein_lib::bytes::find(
            &fixtures::text(view.tree()),
            "You can still read it, but it can’t be reopened.".as_bytes()
        )
        .is_some()
    );
    for name in [b"Keep it".as_slice(), b"Close task"] {
        assert!(fixtures::find(view.tree(), Role::Button, name).is_some());
    }
}
