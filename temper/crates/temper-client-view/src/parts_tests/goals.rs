use super::fixtures;

use crate::common::goals;


use skein_lib::List;




// The mock's row has six landed tasks out of nine (client/pages/README.md).

pub(super) use super::fixtures::goals::template;


const EMPTY: u32 = 0;

// The template fixture's bounds and identities, independent of its name.

const TASK_WAY: u32 = 2;

const WHOLE_WAY: u32 = 4;

const UNTRACKED_WAY: u32 = 7;

const OTHER_WAY: u32 = 9;


#[test]
fn ways_name_their_templates_in_number_order() {
    let root = fixtures::root_in_project(Some(fixtures::PROJECT));
    let mut me = root.pages().me(root.client()).unwrap().clone();
    let project = me.projects.get_mut(EMPTY).unwrap();
    project.templates = List::uncapped();
    for template in [
        template(OTHER_WAY, b"other way", true),
        template(WHOLE_WAY, b"goal for a draft", true),
        template(UNTRACKED_WAY, b"untracked", false),
        template(TASK_WAY, b"goal", true),
    ] {
        project.templates.push(template).unwrap();
    }
    for words in [fixtures::words(), fixtures::browser_words()] {
        let ways = goals::ways(&me, fixtures::PROJECT, &words);
        let mut actual = List::uncapped();
        for way in &ways {
            actual.push((way.name.as_ref(), way.line.as_deref())).unwrap();
        }
        assert_eq!(actual.as_slice(), [
            (b"Task by task".as_slice(), Some(b"The same plan; each task lands on its own once checked.".as_slice())),
            (b"Plan, tasks, land whole".as_slice(), Some(b"A plan from the draft, its tasks on a goal branch, landed as one change once all are checked.".as_slice())),
            (b"other way".as_slice(), None),
        ]);
        assert!(goals::ways(&me, u32::MAX, &words).is_empty());
    }
}
