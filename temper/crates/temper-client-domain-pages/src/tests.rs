//! Tests staged for the pages piece (client/domain-crates.md, 4.3).
mod calls;
mod chat_document;
mod choice;
pub(crate) mod fixtures;
mod goal;
mod report;

#[test]
fn root_home_is_the_only_project_or_all_projects() {
    use crate::{Address, pages::rail::root_home};
    assert_eq!(root_home(None), Address::AllHome);
    let mut me = fixtures::chats::me();
    me.projects.get_mut(0).expect("one project").project = 7;
    assert_eq!(root_home(Some(&me)), Address::Home { project: 7 });
    let mut other = me.projects.get(0).expect("one project").clone();
    other.project = 8;
    let mut projects = skein_lib::List::with_capacity(2);
    projects.push(me.projects.get(0).unwrap().clone()).unwrap();
    projects.push(other).unwrap();
    me.projects = projects;
    assert_eq!(root_home(Some(&me)), Address::AllHome);
    me.projects.clear();
    assert_eq!(root_home(Some(&me)), Address::AllHome);
}
