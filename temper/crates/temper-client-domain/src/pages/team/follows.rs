//! Follows on their owner's row and the selected role's card (client/pages/team.md, 1).
use alloc::boxed::Box;
use skein_lib::{List, bytes::concat};
use temper_client_protocol_wire::v1 as w;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Owner {
    Person(u64),
    Task(u64),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub after: [u8; 12],
    pub state: Box<[u8]>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Follow {
    pub owner: Owner,
    pub source: Box<[u8]>,
    pub status: Box<[u8]>,
    pub line: Box<[u8]>,
    pub entries: List<Entry>,
    pub more: u32,
}

/// Plain facts, in the document's order; the queue is already oldest first.
#[must_use]
pub fn gather(panel: Option<&w::FollowsPanel>) -> List<Follow> {
    let mut follows = List::uncapped();
    if let Some(panel) = panel {
        for row in panel.follows() {
            let owner = match row.setting().owner() {
                w::FollowOwner::Person(person) => Owner::Person(person.person()),
                w::FollowOwner::Task(task) => Owner::Task(task.task()),
            };
            let status = if row.stopped().is_some() {
                Box::from(b"stopped".as_slice())
            } else {
                concat(&[&crate::forge::follows::lag(row.lag()), b" \xc2\xb7 ", &crate::forge::follows::parity(row.parity().as_ref())])
            };
            let mut entries = List::uncapped();
            for entry in row.entries() {
                entries.push(Entry {
                    after: crate::forge::follows::after12(entry.after()),
                    state: crate::forge::follows::state(entry),
                }).expect("entries grow");
            }
            follows.push(Follow {
                owner,
                source: row.source().into(),
                line: concat(&[b"Follows ", row.source(), b" \xc2\xb7 ", &status]),
                status,
                entries,
                more: 0,
            }).expect("follows grow");
        }
    }
    follows
}

/// Only listed owners have lines. The selected card keeps at most eight entries per follow.
pub(super) fn parts(facts: &super::Facts, selected: Option<u64>) -> (List<Follow>, List<Follow>) {
    let mut rows = List::uncapped();
    let mut card = List::uncapped();
    for follow in &facts.follows {
        let mut listed = false;
        match follow.owner {
            Owner::Person(party) => {
                for person in &facts.people {
                    listed |= person.party == party;
                }
            }
            Owner::Task(task) => {
                for role in &facts.roles {
                    listed |= role.task == task;
                }
                if listed && selected == Some(task) {
                    let mut shown = follow.clone();
                    shown.entries = List::uncapped();
                    for entry in &follow.entries {
                        if shown.entries.len() < 8 {
                            shown.entries.push(entry.clone()).expect("entries grow");
                        }
                    }
                    shown.more = follow.entries.len().saturating_sub(shown.entries.len());
                    card.push(shown).expect("card follows grow");
                }
            }
        }
        if listed {
            let mut line = follow.clone();
            line.entries = List::with_capacity(0);
            rows.push(line).expect("owner lines grow");
        }
    }
    (rows, card)
}
