//! Commands through the QA entry over the runner's world (temper/docs/design/testing/qa/README.md, 6).
mod builds;
mod commands;
mod gaps;
mod hold;
mod passes;
mod place;

use std::path::Path;
use temper_qa_tool::{
    CommandFaces, answer,
    faces::{Environment, FilesFace, ProcessOutput, RealArchive, RealDrive, WriteOptions},
    testing::{Agents, Clock, Git, Tree, World},
};

#[derive(Debug)]
struct Reply {
    code: u8,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

fn run(world: &World, arguments: &[&str], session: Option<&str>, input: Option<&[u8]>) -> Reply {
    invoke(world, world.agents(), Git::new(world.files()), arguments, session, input, &[])
}

fn invoke(
    world: &World,
    processes: Agents,
    git: Git,
    arguments: &[&str],
    session: Option<&str>,
    input: Option<&[u8]>,
    extra_environment: &[(&str, &str)],
) -> Reply {
    let files = world.files().at(&world.home.directory).expect("command directory");
    let mut environment = vec![(
        b"TEMPER_INTERNAL_QA_HOME".as_slice().into(),
        world.home.directory.as_os_str().as_encoded_bytes().into(),
    )];
    if let Some(session) = session {
        environment.push((b"TEMPER_INTERNAL_QA_SESSION".as_slice().into(), session.as_bytes().into()));
    }
    environment.extend(extra_environment.iter().map(|(key, value)| (key.as_bytes().into(), value.as_bytes().into())));
    let environment: Environment = environment;
    let mut clock = Clock::default();
    clock.time = 100;
    // The drive and archive adapters build commands; Agents answers each command in the tree.
    let faces = CommandFaces {
        git,
        processes,
        clock,
        files,
        drive: RealDrive { environment: environment.clone() },
        archive: RealArchive { environment: environment.clone() },
    };
    let arguments: Vec<Box<[u8]>> = arguments.iter().map(|word| word.as_bytes().into()).collect();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = answer(
        faces,
        "0.1.0",
        "0123456789ab",
        &arguments,
        &environment,
        &mut input.unwrap_or_default(),
        &mut stdout,
        &mut stderr,
    );
    Reply { code, stdout, stderr }
}

fn output(code: i32, stdout: impl Into<Vec<u8>>) -> ProcessOutput {
    ProcessOutput {
        exit: temper_qa_tool::Exit::Code(code),
        status: format!("exit status: {code}"),
        stdout: stdout.into(),
        stderr: Vec::new(),
    }
}

fn put(files: &mut Tree, path: &Path, bytes: impl AsRef<[u8]>) {
    files.write(path, bytes.as_ref(), WriteOptions::REPLACE, false).expect("write fixture");
}

fn success(reply: &Reply, stdout: &[u8]) {
    assert_eq!(reply.code, 0, "{reply:?}");
    assert_eq!(reply.stdout, stdout);
    assert_eq!(reply.stderr, b"");
}
