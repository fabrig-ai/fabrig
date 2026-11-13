//! Smith's JSON shapes for jig's scoped note and recall tools.
//! The root checks bounds and authority before any value becomes durable.

use jig_domain_notes as notes;
use skein_lib::List;

use crate::{Problem, json::Node};

pub(crate) fn new(value: Node<'_>) -> Result<jig_domain_core_vocabulary::NamedAction, Problem> {
    let first = value.field(b"first")?;
    let remove = value.field(b"remove")?;
    if first.is_some() != remove.is_some() {
        return Err(Problem { field: if first.is_some() { b"remove" } else { b"first" }, why: crate::Why::Range });
    }
    let join = match value.field(b"join")? {
        Some(join) => {
            if first.is_none() {
                return Err(Problem { field: b"first", why: crate::Why::Range });
            }
            join.boolean()?
        }
        None => false,
    };
    let range = match first {
        Some(first) => {
            let first = first.small()?;
            if first == 0 {
                return Err(Problem { field: b"first", why: crate::Why::Range });
            }
            Some(notes::LineRange { first, remove: remove.expect("range members come together").small()?, join })
        }
        None => None,
    };
    let recalled = match value.field(b"recalled")? {
        Some(revision) => Some(revision.small()?),
        None => None,
    };
    if range.is_some() && recalled.is_none() {
        return Err(Problem { field: b"recalled", why: crate::Why::Range });
    }
    let name = match value.field(b"name")? {
        Some(name) => name.number()?,
        None => 0,
    };
    if (recalled.is_some() && name == 0) || (recalled.is_none() && name != 0) {
        return Err(Problem { field: b"name", why: crate::Why::Range });
    }
    let references = match value.field(b"references")? {
        Some(value) => numbers(value)?,
        None => List::with_capacity(0),
    };
    let entry = notes::New {
        name,
        scope: scope(value.required(b"scope")?)?,
        description: value.required(b"description")?.text()?,
        body: value.required(b"body")?.text()?,
        references,
        author: notes::Author::Task { task: 0, attempt: 0 },
    };
    Ok(match range {
        Some(range) => jig_domain_core_vocabulary::NamedAction::NoteLines {
            entry,
            recalled: recalled.expect("range names its revision"),
            range,
        },
        None => jig_domain_core_vocabulary::NamedAction::Note { entry, recalled },
    })
}

pub(crate) fn recall(value: Node<'_>) -> Result<(notes::Recall, u32), Problem> {
    let page = match value.field(b"page")? {
        Some(page) => page.small()?,
        None => 0,
    };
    let by = match value.required(b"kind")?.text()?.as_ref() {
        b"name" => {
            let name = value.required(b"name")?.number()?;
            if name == 0 {
                return Err(Problem { field: b"name", why: crate::Why::Range });
            }
            notes::Recall::Name { name }
        }
        b"search" => {
            let mut values = value.required(b"scopes")?.array()?;
            let Ok(capacity) = u32::try_from(values.len()) else {
                return Err(Problem { field: b"", why: crate::Why::TooLarge });
            };
            let mut scopes = List::with_capacity(capacity);
            for _ in 0..values.len() {
                let value = values.next().expect("counted array element");
                if scopes.push(scope(value)?).is_err() {
                    return Err(Problem { field: b"", why: crate::Why::TooLarge });
                }
            }
            notes::Recall::Search { scopes, query: value.required(b"query")?.text()? }
        }
        _ => return Err(Problem { field: b"kind", why: crate::Why::Range }),
    };
    Ok((by, page))
}

fn scope(value: Node<'_>) -> Result<notes::Scope, Problem> {
    match value.required(b"kind")?.text()?.as_ref() {
        b"deployment" => Ok(notes::Scope::Deployment),
        b"project" => Ok(notes::Scope::Project { project: value.required(b"project")?.small()? }),
        b"goal" => Ok(notes::Scope::Goal {
            project: value.required(b"project")?.small()?,
            goal: value.required(b"goal")?.number()?,
        }),
        b"resources" => {
            let pattern = value.required(b"pattern")?;
            let mut values = pattern.required(b"segments")?.array()?;
            let Ok(capacity) = u32::try_from(values.len()) else {
                return Err(Problem { field: b"", why: crate::Why::TooLarge });
            };
            let mut segments = List::with_capacity(capacity);
            for _ in 0..values.len() {
                let segment = values.next().expect("counted array element");
                if segments.push(segment.text()?).is_err() {
                    return Err(Problem { field: b"", why: crate::Why::TooLarge });
                }
            }
            let last = match pattern.required(b"terminal")?.text()?.as_ref() {
                b"exact" => notes::Last::Exact(pattern.required(b"last")?.text()?),
                b"open" => notes::Last::Open(pattern.required(b"last")?.text()?),
                _ => return Err(Problem { field: b"kind", why: crate::Why::Range }),
            };
            Ok(notes::Scope::Resources {
                project: value.required(b"project")?.small()?,
                connector: value.required(b"connector")?.narrow()?,
                pattern: notes::Pattern { segments: segments.into_boxed(), last },
            })
        }
        _ => Err(Problem { field: b"kind", why: crate::Why::Range }),
    }
}

fn numbers(value: Node<'_>) -> Result<List<u64>, Problem> {
    let mut values = value.array()?;
    let Ok(capacity) = u32::try_from(values.len()) else {
        return Err(Problem { field: b"", why: crate::Why::TooLarge });
    };
    let mut out = List::with_capacity(capacity);
    for _ in 0..values.len() {
        let value = values.next().expect("counted array element");
        let number = value.number()?;
        if number == 0 {
            return Err(Problem { field: b"references", why: crate::Why::Range });
        }
        if out.push(number).is_err() {
            return Err(Problem { field: b"", why: crate::Why::TooLarge });
        }
    }
    Ok(out)
}
