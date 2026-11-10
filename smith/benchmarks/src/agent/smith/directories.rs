//! Author mounts resolve only within this attempt's frozen private seed.
//! The first mount is the native working directory; write authority stays
//! explicit. No user path or runtime filesystem fact is known here
//! (../../../../docs/design/benchmarks/README.md, section 10; domain/tools.md, section 2).

use crate::{Fault, Input, format, input, io, refuse_if};
use std::collections::BTreeSet;
use std::path::{Component, PathBuf};

use serde::Deserialize;
use serde_json::{Value, json};

use crate::agent::Context;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Directory {
    name: String,
    path: PathBuf,
    writable: bool,
    git: bool,
}

pub(super) fn render(context: &Context<'_>) -> Result<Value, Fault> {
    let requested = context
        .variant
        .and_then(|variant| variant.smith.get("workspace.directories"))
        .or_else(|| context.task.smith.get("workspace.directories"));
    let Some(requested) = requested else {
        let working = crate::isolate::working_directory(context.guard, context.private, context.task)?;
        return Ok(json!([{"name":"repo","path":working,"writable":true,"git":true}]));
    };
    let directories: Vec<Directory> =
        requested.clone().try_into().map_err(|error| format(format!("Smith directories: {error}")))?;
    refuse_if(directories.is_empty(), Input, "Smith directories must name at least one seed directory")?;
    let workspace = context.private.workspace.canonicalize()?;
    let mut names = BTreeSet::new();
    let mut hosts = BTreeSet::new();
    let mut rendered = Vec::new();
    for directory in directories {
        if directory.name.is_empty()
            || directory.name == "."
            || directory.name == ".."
            || directory.name.bytes().any(|byte| byte == 0 || byte == b'/')
            || !names.insert(directory.name.clone())
        {
            return Err(input("Smith directories require unique safe single-component names"));
        }
        if directory.path.as_os_str().is_empty()
            || directory.path.components().any(|part| !matches!(part, Component::Normal(_) | Component::CurDir))
        {
            return Err(input(format!("Smith directory {} must use a seed-relative path", directory.name)));
        }
        let candidate = workspace.join(&directory.path);
        let host = context.guard.write_path(&candidate)?;
        let host = host.canonicalize().map_err(|error| io(format!("Smith directory {}: {error}", directory.name)))?;
        if !host.is_dir() || !host.starts_with(&workspace) || !hosts.insert(host.clone()) {
            return Err(input("Smith directories require distinct existing directories inside the private seed"));
        }
        if hosts.iter().any(|earlier| earlier != &host && (earlier.starts_with(&host) || host.starts_with(earlier))) {
            return Err(input("Smith seed directories must be mounted side by side"));
        }
        rendered.push(json!({"name":directory.name,"path":host,"writable":directory.writable,"git":directory.git}));
    }
    if context.task.working_directory.is_some() {
        let working = crate::isolate::working_directory(context.guard, context.private, context.task)?;
        if rendered[0]["path"] != json!(working) {
            return Err(input("the first Smith directory must be the task's working directory"));
        }
    }
    Ok(json!(rendered))
}
