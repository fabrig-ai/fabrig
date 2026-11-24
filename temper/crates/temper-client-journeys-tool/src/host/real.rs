use std::{
    ffi::OsStr,
    fs::{self, File},
    io::{self, Read as _, Write as _},
    os::{
        fd::{AsFd as _, OwnedFd},
        unix::ffi::OsStrExt as _,
    },
    path::Path,
    process::{Child, Command, Stdio},
};

use super::{Answer, Ask, EnvironmentEntry, Program, Ran, Stream};

pub(super) fn ask(environment: &[EnvironmentEntry], ask: Ask) -> Answer {
    match ask {
        Ask::Run(program) => run(environment, &program),
        Ask::Directory(path) => changed(&path, fs::create_dir(&path)),
        Ask::Read(path) => match fs::read(&path) {
            Ok(bytes) => Answer::Bytes(bytes.into_boxed_slice()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Answer::Missing,
            Err(error) => file_failed(&path, &error),
        },
        Ask::Write(path, bytes) => changed(&path, fs::write(&path, bytes)),
        Ask::Remove(path) => {
            let result = match remove(&path) {
                Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
                other => other,
            };
            changed(&path, result)
        }
        Ask::Exists(path) => match path.try_exists() {
            Ok(exists) => Answer::Is(exists),
            Err(error) => file_failed(&path, &error),
        },
        Ask::Alive(pid) => match skein_shell::keeper::process_live(pid) {
            Ok(live) => Answer::Is(live),
            Err(error) => Answer::Failed(format!("cannot check process {pid}: {error:?}")),
        },
    }
}

fn changed(path: &Path, result: io::Result<()>) -> Answer {
    match result {
        Ok(()) => Answer::Done,
        Err(error) => file_failed(path, &error),
    }
}

fn file_failed(path: &Path, error: &io::Error) -> Answer {
    Answer::Failed(format!("{}: {error}", path.display()))
}

fn remove(path: &Path) -> io::Result<()> {
    // Do not descend into a final symlink, even if it names a directory.
    if fs::symlink_metadata(path)?.is_dir() { fs::remove_dir_all(path) } else { fs::remove_file(path) }
}

fn run(environment: &[EnvironmentEntry], program: &Program) -> Answer {
    let name = OsStr::from_bytes(&program.program);
    let streaming = program.stdout == Stream::Out || program.stderr == Stream::Out;
    assert!(
        !streaming || program.stdout == Stream::Err || program.stderr == Stream::Err,
        "cannot stream a child with two captured pipes (stdout: {:?}, stderr: {:?})",
        program.stdout,
        program.stderr,
    );
    let child = match start(environment, program) {
        Ok(child) => child,
        Err(error) => {
            let stderr = format!("{}: {error}\n", name.to_string_lossy()).into_bytes().into_boxed_slice();
            return Answer::Ran(Ran { stderr, ..Ran::new(None, b"") });
        }
    };
    let result = if streaming {
        relay(child, program.stdout == Stream::Out)
    } else {
        child.wait_with_output().map(|output| Ran {
            code: output.status.code(),
            stdout: output.stdout.into_boxed_slice(),
            stderr: output.stderr.into_boxed_slice(),
        })
    };
    match result {
        Ok(ran) => Answer::Ran(ran),
        Err(error) => Answer::Failed(format!("{}: {error}", name.to_string_lossy())),
    }
}

fn start(environment: &[EnvironmentEntry], program: &Program) -> io::Result<Child> {
    let mut command = Command::new(OsStr::from_bytes(&program.program));
    command.args(program.arguments.iter().map(|word| OsStr::from_bytes(word))).env_clear();
    for (name, value) in environment.iter().chain(&program.environment) {
        command.env(OsStr::from_bytes(name), OsStr::from_bytes(value));
    }
    command.env("LC_ALL", "C").stdin(Stdio::null());
    if let Some(directory) = &program.directory {
        command.current_dir(directory);
    }
    command.stdout(if program.stdout == Stream::Err {
        Stdio::from(io::stderr().as_fd().try_clone_to_owned()?)
    } else {
        Stdio::piped()
    });
    command.stderr(if program.stderr == Stream::Err { Stdio::inherit() } else { Stdio::piped() });
    command.spawn()
}

fn relay(mut child: Child, from_stdout: bool) -> io::Result<Ran> {
    let pipe = if from_stdout {
        OwnedFd::from(child.stdout.take().expect("streamed child stdout is piped"))
    } else {
        OwnedFd::from(child.stderr.take().expect("streamed child stderr is piped"))
    };
    let copied = copy_to_stdout(File::from(pipe));
    let code = child.wait()?.code();
    let bytes = copied?;
    let (stdout, stderr) = if from_stdout { (bytes, Box::default()) } else { (Box::default(), bytes) };
    Ok(Ran { code, stdout, stderr })
}

fn copy_to_stdout(mut pipe: File) -> io::Result<Box<[u8]>> {
    let mut buffer = [0; 8192];
    let mut kept = Vec::new();
    loop {
        match pipe.read(&mut buffer) {
            Ok(0) => return Ok(kept.into_boxed_slice()),
            Ok(read) => {
                let bytes = &buffer[..read];
                out(bytes)?;
                kept.extend_from_slice(bytes);
            }
            Err(error) if error.kind() == io::ErrorKind::Interrupted => (),
            Err(error) => return Err(error),
        }
    }
}

pub(super) fn out(bytes: &[u8]) -> io::Result<()> {
    let mut stdout = io::stdout().lock();
    stdout.write_all(bytes)?;
    stdout.flush()
}

pub(super) fn err(bytes: &[u8]) -> io::Result<()> {
    let mut stderr = io::stderr().lock();
    stderr.write_all(bytes)?;
    stderr.flush()
}
