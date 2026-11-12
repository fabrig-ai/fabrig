//! The recipe commands' io, over the reader and the walk's actions.
use super::{Chain, Reader, SETUPS, Walk, hex, walk};
use crate::{EnvironmentEntry, drive};
use std::{collections::VecDeque, ffi::OsStr, fs, os::unix::ffi::OsStrExt, path::PathBuf};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mode {
    Walk,
    Hash,
    List,
}
pub(super) struct Arguments {
    pub(super) name: Option<String>,
    pub(super) setups: PathBuf,
    pub(super) from: Option<String>,
    pub(super) drive: Option<drive::Options>,
}
impl Arguments {
    pub(super) fn parse(mode: Mode, arguments: &[Box<[u8]>]) -> Result<Self, String> {
        let mut name = None;
        let mut setups = None;
        let mut from = None;
        let mut session = vec![];
        let mut args = arguments.iter();
        while let Some(argument) = args.next() {
            match argument.as_ref() {
                b"--setups" => {
                    if setups.is_some() { return Err("repeated --setups".into()); }
                    setups = Some(PathBuf::from(OsStr::from_bytes(value(&mut args, "--setups DIR")?)));
                }
                b"--from" if mode == Mode::Walk => {
                    if from.is_some() { return Err("repeated --from".into()); }
                    from = Some(text(value(&mut args, "--from ANCESTOR")?)?);
                }
                b"--bin" | b"--bundle" | b"--scratch" if mode == Mode::Walk => {
                    session.push(argument.clone());
                    session.push(value(&mut args, &String::from_utf8_lossy(argument))?.into());
                }
                b"--strict" if mode == Mode::Walk => session.push(argument.clone()),
                flag if flag.starts_with(b"--") => return Err(format!("unknown recipe option {}", String::from_utf8_lossy(flag))),
                bytes => {
                    if mode == Mode::List || name.is_some() { return Err("unexpected recipe argument".into()); }
                    name = Some(text(bytes)?);
                }
            }
        }
        if mode != Mode::List && name.is_none() { return Err("missing recipe NAME".into()); }
        Ok(Self {
            name,
            setups: setups.unwrap_or_else(|| SETUPS.into()),
            from,
            drive: if mode == Mode::Walk { Some(drive::Options::parse(&session)?) } else { None },
        })
    }
}
fn value<'a>(args: &mut std::slice::Iter<'a, Box<[u8]>>, flag: &str) -> Result<&'a [u8], String> {
    args.next().filter(|arg| !arg.is_empty() && !arg.starts_with(b"--")).map(AsRef::as_ref).ok_or_else(|| format!("missing {flag}"))
}
fn text(bytes: &[u8]) -> Result<String, String> {
    std::str::from_utf8(bytes).map(str::to_owned).map_err(|_| "recipe name is not UTF-8".into())
}
pub(crate) fn run(mode: Mode, arguments: &[Box<[u8]>], environment: &[EnvironmentEntry]) -> drive::Exit {
    match execute(mode, arguments, environment) {
        Ok(exit) => exit,
        Err(why) => drive::Exit::own(why),
    }
}
fn execute(mode: Mode, arguments: &[Box<[u8]>], environment: &[EnvironmentEntry]) -> Result<drive::Exit, String> {
    let args = Arguments::parse(mode, arguments)?;
    if mode == Mode::List {
        let names = fs::read_dir(&args.setups).map_err(|error| format!("{}: {error}", args.setups.display()))?
            .map(|entry| entry.map(|entry| entry.file_name().to_string_lossy().into_owned()).map_err(|error| error.to_string()))
            .collect::<Result<Vec<_>, _>>()?;
        return Ok(drive::Exit { status: 0, shown: catalogue(&Reader::Directory(&args.setups), names), error: None });
    }
    let name = args.name.as_deref().expect("name required");
    let chain = Chain::read(&args.setups, name).map_err(|why| why.to_string())?;
    if mode == Mode::Hash {
        return Ok(drive::Exit { status: 0, shown: vec![hex(chain.hash())], error: None });
    }
    let (mut walk, actions) = Walk::start(&chain, args.from.as_deref()).map_err(|why| why.to_string())?;
    let mut options = args.drive.expect("walk options");
    let directory = |name: &str| -> Result<bool, String> {
        let path = options.scratch.join(name);
        match fs::symlink_metadata(&path) {
            Ok(_) => Ok(args.from.is_none() || path.is_dir()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(format!("{}: {error}", path.display())),
        }
    };
    walk::check_scratch(args.from.as_deref(), directory("config")?, directory("data")?)
        .map_err(|why| format!("{}: {why}", options.scratch.display()))?;
    options = session_options(&chain, &args.setups, options);
    let mut session = match drive::Session::start(options, environment) {
        Ok(session) => Some(session),
        Err(exit) => return Ok(exit),
    };
    let mut actions: VecDeque<_> = actions.into();
    while let Some(action) = actions.pop_front() {
        match action {
            walk::Action::Print(line) => println!("{line}"),
            walk::Action::Line(line) => {
                match session.as_mut().expect("live session").line(&line) {
                    Ok(answer) => actions.extend(walk.step(walk::Event::Answer(answer))),
                    Err(_) => actions.push_back(walk::Action::End),
                }
            }
            walk::Action::End => {
                let exit = session.take().expect("end once").end();
                actions.extend(walk.step(walk::Event::Ended(exit)));
            }
            walk::Action::Finished(exit) => return Ok(exit),
        }
    }
    unreachable!("walk finishes after ending session")
}
pub(super) fn session_options(chain: &Chain, setups: &std::path::Path, mut options: drive::Options) -> drive::Options {
    options.scripts = chain.scripts(setups);
    options
}
pub(super) fn catalogue(reader: &Reader<'_>, mut names: Vec<String>) -> Vec<String> {
    names.sort();
    names.into_iter().map(|name| {
        let describe = || -> Result<String, String> {
            let chain = Chain::read_with(reader, &name).map_err(|why| why.to_string())?;
            let recipe = chain.recipes.last().expect("read chain");
            let readme = reader.bytes(&name, "README.md").map_err(|why| format!("{name}/README.md: {why}"))?;
            let readme = std::str::from_utf8(&readme).map_err(|why| format!("{name}/README.md: {why}"))?;
            let paragraph = readme.split_whitespace().collect::<Vec<_>>().join(" ");
            let sentence = paragraph.split_inclusive(['.', '!', '?']).next().unwrap_or_default();
            let after = recipe.after.as_ref().map_or(String::new(), |parent| format!(" (after {parent})"));
            Ok(format!("{name}{after}: {sentence}"))
        };
        describe().unwrap_or_else(|why| format!("{name}: refused: {why}"))
    }).collect()
}
