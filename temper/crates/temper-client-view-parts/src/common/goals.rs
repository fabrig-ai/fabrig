//! Shared goals items (temper/docs/design/client/view-crates.md, 3.3).
use crate::words::{Word, Words};
use alloc::boxed::Box;
use jig_client::contract as c;
use skein_lib::List;

/// A project's named way of carrying out a goal (client/pages/README.md).
#[derive(Debug)]
pub struct Way {
    pub number: u32,
    pub name: Box<[u8]>,
    pub line: Option<Box<[u8]>>,
}

/// Tracked templates in their configured number order (client/pages/README.md).
#[must_use]
pub fn ways(me: &c::Me, project: u32, words: &Words) -> List<Way> {
    let mut result = List::uncapped();
    for member in &me.projects {
        if member.project != project {
            continue;
        }
        let mut order = List::uncapped();
        for (index, template) in member.templates.iter().enumerate() {
            if template.tracked.is_some() {
                order
                    .push((template.number, u32::try_from(index).expect("template index fits")))
                    .expect("ways grow with templates");
            }
        }
        let mut order = order.into_boxed();
        order.sort_unstable();
        for (_, index) in &order {
            let template = member.templates.get(*index).expect("ordered template");
            let way = match template.name.bytes() {
                b"goal" => Way {
                    number: template.number,
                    name: way_name(template.name.bytes(), words),
                    line: Some(words.say(Word::GoalTaskByTaskLine, &[])),
                },
                b"goal for a draft" => Way {
                    number: template.number,
                    name: way_name(template.name.bytes(), words),
                    line: Some(words.say(Word::GoalLandWholeLine, &[])),
                },
                name => Way { number: template.number, name: way_name(name, words), line: None },
            };
            result.push(way).expect("ways grow with templates");
        }
    }
    result
}

/// A template's name in the person's words (client/pages/README.md, client/pages/settings.md, 2).
#[must_use]
pub fn way_name(name: &[u8], words: &Words) -> Box<[u8]> {
    match name {
        b"goal" => words.say(Word::GoalTaskByTask, &[]),
        b"goal for a draft" => words.say(Word::GoalLandWhole, &[]),
        name => name.into(),
    }
}
