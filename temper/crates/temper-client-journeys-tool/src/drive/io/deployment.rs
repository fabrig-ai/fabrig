//! Private deployment files; all writes are beneath scratch (testing/qa/deployment.md, 3–6).
use super::super::{
    Plan,
    deployment::{self, Preparation, Values},
};
use std::{
    fs,
    io::Write,
    os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt},
    path::{Path, PathBuf},
};

pub(in crate::drive) fn directory(path: &Path) -> Result<(), String> {
    if let Ok(metadata) = fs::symlink_metadata(path) {
        return if metadata.is_dir() {
            Ok(())
        } else {
            Err(format!("{}: expected a directory, not a link", path.display()))
        };
    }
    fs::DirBuilder::new()
        .mode(0o700)
        .create(path)
        .and_then(|()| fs::set_permissions(path, fs::Permissions::from_mode(0o700)))
        .map_err(|why| format!("{}: {why}", path.display()))
}
pub(in crate::drive) fn write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let create = || -> std::io::Result<()> {
        let mut file = fs::OpenOptions::new().create_new(true).write(true).mode(0o600).open(path)?;
        file.set_permissions(fs::Permissions::from_mode(0o600))?;
        file.write_all(bytes)
    };
    create().map_err(|why| format!("{}: {why}", path.display()))
}
fn read(path: &Path, limit: u32) -> Result<Box<[u8]>, String> {
    skein_shell::read_file(path, limit).map_err(|why| format!("{}: {why:?}", path.display()))
}
fn beneath(scratch: &Path, relative: &Path) -> Result<PathBuf, String> {
    let mut path = scratch.to_path_buf();
    for part in relative.components() {
        let std::path::Component::Normal(name) = part else {
            return Err("expected a relative deployment path".into());
        };
        path.push(name);
        let metadata = fs::symlink_metadata(&path).map_err(|why| format!("{}: {why}", path.display()))?;
        if metadata.file_type().is_symlink() {
            return Err(format!("{}: deployment path is a link", path.display()));
        }
    }
    Ok(path)
}
pub(in crate::drive) fn prepare(plan: &Plan) -> Result<Values, String> {
    prepare_with(plan, None)
}
/// Tests supply the random bytes and never start a process.
pub(in crate::drive) fn prepare_with(plan: &Plan, random: Option<[u8; 32]>) -> Result<Values, String> {
    let scratch = &plan.options.scratch;
    let place = plan.place()?;
    let present = |name| fs::symlink_metadata(scratch.join(name)).is_ok();
    let preparation = deployment::preparation(present("config"), present("data"))?;
    if preparation == Preparation::Fresh {
        let source = plan.options.deployment.as_ref().expect("deployment directory");
        for entry in fs::read_dir(source).map_err(|why| format!("{}: {why}", source.display()))? {
            let entry = entry.map_err(|why| format!("{}: {why}", source.display()))?;
            if !["engine.json", "worker.json", "agent.json", "forgejo.json", "model.json"]
                .iter()
                .any(|name| entry.file_name() == *name)
            {
                return Err(format!("{}: unexpected deployment document", entry.path().display()));
            }
        }
        let mut input = vec![];
        for name in ["engine.json", "worker.json", "agent.json", "forgejo.json", "model.json"] {
            input.push((name, read(&source.join(name), deployment::DOCUMENT_BYTES)?));
        }
        let documents = deployment::documents(
            &input.iter().map(|(name, bytes)| (*name, bytes.as_ref())).collect::<Vec<_>>(),
            &place,
        )?;
        for relative in [
            "config",
            "config/engine",
            "config/worker",
            "data",
            "data/engine",
            "data/engine/secrets",
            "data/worker",
            "data/worker/secrets",
            "data/forgejo",
            "data/model",
        ] {
            directory(&scratch.join(relative))?;
        }
        for (path, bytes) in documents {
            write(&path, &bytes)?;
        }
        let seed = match random {
            Some(bytes) => bytes,
            None => skein_shell::crypto_seed()
                .map_err(|why| format!("worker secret randomness: {why}"))?
                .expose()
                .try_into()
                .expect("32-byte seed"),
        };
        let (secret, digest) = deployment::secret_pair(&seed);
        write(&scratch.join("data/worker/secrets/worker.secret"), &secret)?;
        write(&scratch.join("data/engine/secrets/worker.digest"), &digest)?;
    }
    // Validate held paths before using them; a copy is never rendered again.
    beneath(scratch, Path::new("config"))?;
    beneath(scratch, Path::new("data"))?;
    for relative in [
        "config/engine",
        "config/worker",
        "data/engine",
        "data/worker",
        "data/forgejo",
        "data/model",
        "data/bin",
        "cache",
        "cache/engine",
        "cache/worker",
    ] {
        directory(&scratch.join(relative))?;
    }
    let git = scratch.join("data/bin/git");
    match fs::remove_file(&git) {
        Ok(()) => {}
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => {}
        Err(why) => return Err(format!("{}: {why}", git.display())),
    }
    std::os::unix::fs::symlink(plan.options.bin.join("temper-box"), &git)
        .map_err(|why| format!("{}: {why}", git.display()))?;

    let forge = read(&beneath(scratch, Path::new("config/forgejo.json"))?, deployment::DOCUMENT_BYTES)?;
    let model = read(&beneath(scratch, Path::new("config/model.json"))?, deployment::DOCUMENT_BYTES)?;
    if !plan.options.scripts.is_empty() {
        let mut joined = temper_fake::llm::ScriptJoin::empty();
        for path in &plan.options.scripts {
            let source = skein_shell::read_file(path, temper_fake::DOCUMENT_BYTES)
                .map_err(|why| format!("{}: {}", path.display(), temper_shell::directories::unread(why)))?;
            joined.push(&model, &source).map_err(|why| {
                let line = why.line();
                let line = String::from_utf8_lossy(&line);
                let line = if why.words == b"past byte bound" { line.trim_end_matches('\n') } else { &line };
                format!("{}: {line}", path.display())
            })?;
        }
        write(&scratch.join("session/scripts.json"), &joined.into_bytes())?;
    }
    deployment::values(&place, &forge, &model)
}
/// Resolve bounded settings inputs before ctl starts, leaving flags and stdin unchanged.
pub(in crate::drive) fn settings_files(
    directory: Option<&Path>,
    mut args: Vec<Box<[u8]>>,
) -> Result<Vec<Box<[u8]>>, String> {
    use std::os::unix::ffi::OsStrExt;
    let paths = crate::line::settings_paths(&args);
    let directory = directory
        .ok_or_else(|| format!("{}: no settings files directory (--files DIR)", String::from_utf8_lossy(paths[0])))?;
    let directory = std::path::absolute(directory).map_err(|why| format!("{}: {why}", directory.display()))?;
    super::forge::read_files(&directory, &paths.iter().map(|path| Box::from(*path)).collect::<Vec<_>>())?;
    for arg in args.iter_mut().skip(2).take_while(|arg| arg.as_ref() == b"-" || !arg.starts_with(b"-")) {
        if arg.as_ref() != b"-" {
            *arg = directory.join(std::ffi::OsStr::from_bytes(arg)).as_os_str().as_bytes().into();
        }
    }
    Ok(args)
}
pub(in crate::drive) fn patch(
    plan: &Plan,
    document: crate::line::Document,
    path: &[u8],
) -> Result<deployment::RenderedFile, String> {
    let directory = plan.options.files.as_ref().ok_or("deployment patch: no files directory (--files DIR)")?;
    let files = super::forge::read_files(directory, &[path.into()])?;
    let path = beneath(&plan.options.scratch, Path::new(document.path()))?;
    let bytes = read(&path, deployment::DOCUMENT_BYTES)?;
    let change = files.first().expect("one patch file");
    let merged =
        deployment::merge(&bytes, change, &plan.place()?).map_err(|why| format!("{}: {why}", path.display()))?;
    Ok((path, merged))
}
pub(in crate::drive) fn replace(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let temporary = path.with_extension("json.patch");
    write(&temporary, bytes)?;
    fs::rename(&temporary, path).map_err(|why| {
        let _removed = fs::remove_file(&temporary);
        format!("{}: {why}", path.display())
    })
}
pub(in crate::drive) fn remove_link(scratch: &Path) -> Result<(), String> {
    let path = scratch.join("session/link");
    match fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(why) => Err(format!("{}: {why}", path.display())),
    }
}
