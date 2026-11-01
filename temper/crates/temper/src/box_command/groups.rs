//! File and machine groups (temper/docs/plan/playground-reset-r4/notes/contracts.md, 3, 6).
use temper_forge_setup::{Group, setup::Entry};

/// The groups of `<data>/box/groups`, but the document's numbers, by
/// their kinds' builders; the caller places each before serve (contracts, 3).
pub(super) fn files(
    directories: &super::super::Directories,
    checked: &temper_forge_setup::check::Checked,
    demo: Option<&temper_fake::demo::Demo>,
    flags: &[Box<[u8]>],
    forgejo_listens: bool,
) -> Result<Vec<(Group, Entry)>, String> {
    let directory = super::super::path(&directories.data).join("box/groups");
    let entries = match std::fs::read_dir(&directory) {
        Ok(entries) => entries,
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(why) => return Err(format!("{}: {why}", directory.display())),
    };
    let mut paths = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|why| format!("{}: {why}", directory.display()))?;
        let name = entry.file_name();
        let forge = name.to_str().and_then(|name| name.parse::<u16>().ok());
        let Some(forge) = forge.filter(|forge| name == forge.to_string().as_str()) else {
            eprintln!("{}: not a canonical forge number; skipped", entry.path().display());
            continue;
        };
        if checked.setup.entry(forge).is_none() {
            paths.push((forge, entry.path()));
        }
    }
    paths.sort_by_key(|(forge, _)| *forge);
    let mut groups = Vec::new();
    for (forge, path) in paths {
        let bytes = skein_shell::read_file(&path, temper_forge_setup::file::BYTES)
            .map_err(|why| format!("{}: {why:?}", path.display()))?;
        let file =
            temper_forge_setup::file::read(&bytes, forge, &checked.limits, &directories.jig()).map_err(|why| {
                format!("{}: {}", path.display(), String::from_utf8_lossy(&temper_forge_setup::problem::lines(&why)))
            })?;
        let llm = if file.playground {
            Some(
                demo.ok_or_else(|| format!("{}: playground requires application.demo", path.display()))?.llm().listen(),
            )
        } else {
            None
        };
        let group = temper_forge_setup::group(&file.entry, flags, forgejo_listens)
            .or_else(|| temper_fake::steps::group(&file.entry, llm, flags))
            .ok_or_else(|| format!("{}: setup.mode: requires an owned or fake group", path.display()))?;
        groups.push((group, file.entry));
    }
    Ok(groups)
}

/// The kinds' groups, numbered `u16::MAX`, with their source entries for notices.
pub(super) fn kinds(
    owned: Option<&temper_forge_setup::setup::Template>,
    demo: Option<&temper_fake::demo::Demo>,
    flags: &[Box<[u8]>],
    forgejo_listens: bool,
) -> Vec<(Group, Entry)> {
    let mut groups = Vec::new();
    if let Some(owned) = owned {
        let entry = owned.entry(u16::MAX);
        let group = temper_forge_setup::group(&entry, flags, forgejo_listens).expect("checked owned template");
        groups.push((group, entry));
    }
    if let Some(demo) = demo
        && let Some(fake) = demo.fake()
    {
        let entry = fake.entry(u16::MAX);
        let group = temper_fake::steps::group(&entry, Some(demo.llm().listen()), flags).expect("checked fake template");
        groups.push((group, entry));
    }
    groups
}
