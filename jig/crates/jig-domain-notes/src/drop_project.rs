//! Project index lines are erased a bounded page at a time
//! (temper/docs/design/draft/playground-reset.md, 5). Globally named entries
//! remain; they may also be indexed outside the project. The parent commits
//! each page before its next load, and completion follows an empty page.

use skein_lib::{Queue, Token};

use crate::{Domain, Key, Limits, Range, Record, Refusal, Request, Rows, Scope};

#[derive(Debug)]
pub(crate) struct Dropping {
    owner: Token,
    project: u32,
    after: Option<Key>,
    load_owner: Token,
}

pub(crate) fn start(domain: &mut Domain, owner: Token, project: u32, out: &mut Queue<Request>) {
    if domain.busy() {
        out.push(Request::Refused { owner, why: Refusal::Busy });
        return;
    }
    let load_owner = domain.next_load_token();
    let dropping = Dropping { owner, project, after: None, load_owner };
    ask(&dropping, out);
    domain.dropping = Some(dropping);
}

pub(crate) fn failed(domain: &mut Domain, owner: Token, out: &mut Queue<Request>) {
    let mut dropping = domain.dropping.take().expect("project drop has an outstanding load");
    if dropping.load_owner == owner {
        dropping.load_owner = domain.next_load_token();
        ask(&dropping, out);
    }
    domain.dropping = Some(dropping);
}

pub(crate) fn loaded(domain: &mut Domain, limits: &Limits, owner: Token, rows: Rows, out: &mut Queue<Request>) {
    let mut dropping = domain.dropping.take().expect("project drop has an outstanding load");
    assert!(dropping.load_owner == owner, "a store page echoes its load token");
    assert!(rows.records.capacity() <= limits.load_rows, "the store page fits its bound");
    if rows.records.is_empty() {
        out.push(Request::ProjectDropped { owner: dropping.owner });
        return;
    }
    for record in rows.records.into_boxed() {
        match record {
            Record::Line(line) => {
                let project = match &line.scope {
                    Scope::Deployment => unreachable!("project load excludes deployment lines"),
                    Scope::Project { project } | Scope::Goal { project, .. } | Scope::Resources { project, .. } => {
                        *project
                    }
                };
                assert!(project == dropping.project, "project load returns its own lines");
                domain.invalidate(&line.scope);
                let key = Key::Line { scope: line.scope, name: line.name };
                if let Some(after) = &dropping.after {
                    assert!(&key > after, "project lines are ordered");
                }
                dropping.after = Some(key.clone());
                out.push(Request::Erase { key });
            }
            Record::Entry(_) => unreachable!("project load returns index lines"),
        }
    }
    dropping.load_owner = domain.next_load_token();
    ask(&dropping, out);
    domain.dropping = Some(dropping);
}

fn ask(dropping: &Dropping, out: &mut Queue<Request>) {
    out.push(Request::Load {
        owner: dropping.load_owner,
        range: Range::Project { project: dropping.project, after: dropping.after.clone() },
    });
}
