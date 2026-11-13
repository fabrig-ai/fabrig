//! Shared setup items (temper/docs/design/client/view-crates.md, 3.3).
use crate::words::{Word, Words};
use alloc::boxed::Box;
use skein_ui::phrases::Hole;
use temper_client_domain::pages::team;

#[expect(clippy::manual_find, reason = "step code uses explicit loops, not closures")]
#[must_use]
pub fn role(facts: &team::Facts, task: u64) -> Option<&team::Role> {
    for role in &facts.roles {
        if role.task == task {
            return Some(role);
        }
    }
    None
}

#[must_use]
pub fn named_holder(words: &Words, facts: Option<&team::Facts>, holder: &team::Holder) -> Box<[u8]> {
    match holder {
        team::Holder::Task(task) => {
            if let Some(facts) = facts
                && let Some(role) = role(facts, *task)
            {
                return role.name.clone();
            }
            words.say(Word::TeamTaskName, &[Hole::Number(*task)])
        }
        team::Holder::Party(party) => {
            if let Some(facts) = facts {
                for person in &facts.people {
                    if person.party == *party {
                        return if person.you { words.say(Word::TeamYou, &[]) } else { person.name.clone() };
                    }
                }
            }
            words.say(Word::MemberUnnamed, &[Hole::Number(*party)])
        }
        team::Holder::Role(role) => role_name(words, *role),
    }
}

#[must_use]
pub fn role_name(words: &Words, number: u32) -> Box<[u8]> {
    let word = match number {
        0 => Word::TeamRoleOwners,
        1 => Word::TeamRoleMaintainers,
        2 => Word::TeamRoleMembers,
        3 => Word::TeamRoleObservers,
        4.. => return words.say(Word::TeamRolePeople, &[Hole::Number(u64::from(number))]),
    };
    words.say(word, &[])
}
