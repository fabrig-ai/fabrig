use super::fixtures;

use crate::common::goals;

use skein_lib::List;

// The mock's row has six landed tasks out of nine (client/pages/README.md).

use super::fixtures::goals::{named_template, template};

const EMPTY: u32 = 0;

// The template fixture's bounds and identities, independent of its name.

const TASK_WAY: u32 = 2;

const WHOLE_WAY: u32 = 4;

const UNTRACKED_WAY: u32 = 7;

const OTHER_WAY: u32 = 9;
const UNNAMED_TASK_WAY: u32 = 10;
const UNNAMED_WHOLE_WAY: u32 = 11;
const TASK_LINE: &[u8] = b"The same plan; each task lands on its own once checked.";
const WHOLE_LINE: &[u8] = b"A plan, its tasks on a goal branch, landed as one change once all are checked.";

#[test]
fn ways_name_their_templates_in_number_order() {
    let root = fixtures::root_in_project(Some(fixtures::PROJECT));
    let mut me = root.pages().me(root.client()).unwrap().clone();
    for whole_shown in [b"goal landed whole".as_slice(), b"goal for a draft", b"Whole goals"] {
        let mut milestone = template(1, b"milestone", true);
        milestone.flags.ended_by = jig_client::contract::EndedBy::Requester;
        me.projects.get_mut(EMPTY).unwrap().templates = List::from_boxed(Box::new([
            milestone,
            named_template(OTHER_WAY, b"other-way", b"goal landed whole", true),
            named_template(WHOLE_WAY, b"draft-goal", whole_shown, true),
            template(UNTRACKED_WAY, b"untracked", false),
            named_template(TASK_WAY, b"goal", b"Separate goals", true),
            template(UNNAMED_TASK_WAY, b"goal", true),
            template(UNNAMED_WHOLE_WAY, b"goal for a draft", true),
        ]));
        for words in [fixtures::words(), fixtures::browser_words()] {
            let ways = goals::ways(&me, fixtures::PROJECT, &words);
            let mut labels = List::with_capacity(ways.len());
            for way in &ways {
                labels.push((way.number, way.name.as_ref(), way.line.as_deref())).unwrap();
            }
            assert_eq!(
                labels.as_slice(),
                [
                    (TASK_WAY, b"Task by task".as_slice(), Some(TASK_LINE)),
                    (WHOLE_WAY, b"Plan, tasks, land whole".as_slice(), Some(WHOLE_LINE)),
                    (OTHER_WAY, b"goal landed whole".as_slice(), None),
                    (UNNAMED_TASK_WAY, b"goal".as_slice(), None),
                    (UNNAMED_WHOLE_WAY, b"goal for a draft".as_slice(), None),
                ]
            );
            assert!(goals::ways(&me, u32::MAX, &words).is_empty());
        }
    }
}
