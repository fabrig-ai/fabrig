//! The box's groups from their files (temper/docs/design/draft/playground-reset.md, 3 and 4; notes contracts T3).
use skein_io::kernel::{Complete, Exit, Submit};
use skein_lib::{Queue, Time, Wall};
use std::{fs, path::Path};
use temper_box::host::{Loaded, Service};
use temper_forge_setup::{
    Group,
    check::Checked,
    setup::{Entry, GROUP_BYTES, Mode},
};

pub(crate) struct Context {
    pub directories: crate::Directories,
    pub limits: Checked,
    pub llm: Option<std::net::SocketAddr>,
    pub forgejo_listens: bool,
}

/// Reads a group's settings from its private file.
pub(crate) fn entry(directories: &crate::Directories, limits: &Checked, forge: u16) -> Result<Option<Entry>, String> {
    let path = crate::path(&directories.data).join(format!("box/groups/{forge}"));
    let bytes = match skein_shell::read_private_file(&path, GROUP_BYTES, skein_shell::effective_user()) {
        Ok(bytes) => bytes,
        Err(skein_shell::Unread::Absent) => return Ok(None),
        Err(why) => return Err(format!("{}: {}", path.display(), crate::unread(why))),
    };
    let entry = Entry::group(bytes.expose(), &limits.limits, &directories.jig()).map_err(|why| {
        let words = temper_forge_setup::problem::lines(&why);
        let words = String::from_utf8_lossy(&words);
        format!("box/groups/{forge}: {}", words.lines().next().unwrap_or("invalid settings"))
    })?;
    if entry.forge != forge {
        return Err(format!("box/groups/{forge}: forge {}'s settings", entry.forge));
    }
    Ok(Some(entry))
}

/// A command requires the forge's recorded settings.
pub(crate) fn required(directories: &crate::Directories, limits: &Checked, forge: u16) -> Result<Entry, String> {
    entry(directories, limits, forge)?.ok_or_else(|| format!("forge {forge} has no settings: box/groups/{forge}"))
}

/// `box/groups/N` as the box's load seam answers it.
pub(crate) fn load(context: &Context, forge: u16) -> Loaded {
    let entry = match entry(&context.directories, &context.limits, forge) {
        Ok(Some(entry)) => entry,
        Ok(None) => return Loaded::Absent,
        Err(why) => return Loaded::Refused(why.into_bytes().into_boxed_slice()),
    };
    let why = match &entry.mode {
        Mode::Managed { .. } => Some("a managed forge runs no group"),
        Mode::Fake(_) if context.llm.is_none() => Some("no playground LLM"),
        Mode::Owned(_) | Mode::Fake(_) => None,
    };
    if let Some(why) = why {
        return Loaded::Refused(format!("box/groups/{forge}: {why}").into_bytes().into_boxed_slice());
    }
    let mut group =
        temper_fake::steps::group(&entry, context.llm, &context.directories.flags(), context.forgejo_listens)
            .expect("group mode and playground LLM checked");
    if let Some((address, notice)) = super::owned_notice(&entry) {
        for (_, step) in &mut group.steps {
            if step.ready() == Some(address) {
                step.set_started_notice(notice.clone());
            }
        }
    }
    Loaded::Group(group)
}

/// A group's files, in T3's order; the cache stays.
pub(crate) fn remove(data: &Path, forge: u16) -> Result<(), Box<[u8]>> {
    for (name, directory) in [
        (format!("box/started/{forge}"), false),
        (format!("forge-{forge}"), true),
        (format!("box/groups/{forge}"), false),
    ] {
        let path = data.join(name);
        let result = if directory { fs::remove_dir_all(&path) } else { fs::remove_file(&path) };
        match result {
            Ok(()) => {}
            Err(why) if why.kind() == std::io::ErrorKind::NotFound => {}
            Err(why) => return Err(format!("{}: {why}", path.display()).into_bytes().into_boxed_slice()),
        }
    }
    Ok(())
}

/// The recorded groups at the box's start, and the terminal lines they leave.
pub(crate) fn recorded(context: &Context, before: u32) -> Result<(Vec<Group>, Vec<String>), String> {
    let path = crate::path(&context.directories.data).join("box/started");
    let entries = match fs::read_dir(&path) {
        Ok(entries) => entries,
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => return Ok((Vec::new(), Vec::new())),
        Err(why) => return Err(format!("{}: {why}", path.display())),
    };
    let mut records = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|why| format!("{}: {why}", path.display()))?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        if name.is_empty() || !name.bytes().all(|byte| byte.is_ascii_digit()) {
            continue;
        }
        let Ok(forge) = name.parse::<u16>() else { continue };
        records.push((forge, entry.path()));
    }
    records.sort();
    let mut groups = Vec::new();
    let mut lines = Vec::new();
    for (forge, path) in records {
        match load(context, forge) {
            Loaded::Group(mut group) => {
                if groups.iter().any(|previous: &Group| previous.forge == forge) {
                    continue;
                }
                group.recorded = true;
                group.before = before;
                groups.push(group);
            }
            Loaded::Absent => {
                fs::remove_file(&path).map_err(|why| format!("{}: {why}", path.display()))?;
                lines.push(format!("box: group {forge}: no settings; its started record is removed"));
            }
            Loaded::Refused(words) => lines.push(format!("box: group {forge}: {}", String::from_utf8_lossy(&words))),
        }
    }
    Ok((groups, lines))
}

/// The box under its shell: the seam answered after each iteration.
pub(crate) struct Boxed {
    pub service: Service,
    pub context: Context,
}
impl skein_shell::Host for Boxed {
    fn iterate(&mut self, now: Time, wall: Wall) {
        self.service.iterate(now, wall);
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        self.service.completions()
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        self.service.submissions()
    }
    fn work_pending(&self, now: Time) -> bool {
        self.service.work_pending(now)
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        self.service.next_mechanism_deadline()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        self.service.next_policy_deadline()
    }
    fn is_empty(&self) -> bool {
        self.service.is_empty()
    }
    fn exit(&self) -> Option<Exit> {
        self.service.exit()
    }
    fn operations(&self) -> u32 {
        self.service.operations()
    }
    fn state_digest(&self) -> Option<[u8; 32]> {
        self.service.state_digest()
    }
    fn drain(&mut self) {
        self.service.drain();
        while let Some(forge) = self.service.take_load() {
            let loaded = load(&self.context, forge);
            self.service.loaded(forge, loaded);
        }
        while let Some(forge) = self.service.take_removal() {
            let removed = remove(crate::path(&self.context.directories.data), forge);
            self.service.removed(forge, removed);
        }
    }
}
