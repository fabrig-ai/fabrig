//! Headless product stories reuse the shipped binary, person and provider hosts.
use super::{Process, Ran, Settling, window};
use skein_io::kernel::Exit;
use skein_lib::Duration;
use skein_shell::{Clock, Host};
use skein_world::{
    end_to_end::{Binary, Command, Mode, Streams},
    real,
};
use smith_local_process_world::{
    process::Proc,
    terminal::{Action, Terminal},
    world,
};
use smith_real_world::{Scratch, settled::assert_no_children};

#[derive(Clone, Copy)]
pub enum Reply {
    Wait,
    Accepted,
    Filtered,
}

#[must_use]
pub fn fake_provider_settings(scratch: &Scratch) -> Vec<u8> {
    let clock = Clock::new();
    let token = world::login_at(clock.now().wall, false);
    let document = serde_json::json!({"tokens": {"access_token": std::str::from_utf8(&token).expect("scratch token text"), "account_id": "acc", "refresh_token": "UNISSUED_NEVER_COPY"}});
    std::fs::write(scratch.path().join("auth.json"), serde_json::to_vec(&document).expect("scratch login encoding"))
        .expect("scratch login file");
    format!("credentials='{}'\nmodel='codex/gpt-6-luna'\neffort='low'\ninstructions='@local-shell Assist'\n[accounts.codex]\nsource='env'\nvariable='SMITH_FIXTURE_TOKEN'\n[endpoints.codex]\naddress='{}:34443'\nserver_name='skein.test'\ntrust.system=false\ntrust.der=['{}']\n", scratch.path().join("credentials").display(), smith_real_world::loopback(), scratch.path().join("root.der").display()).into_bytes()
}

#[must_use]
pub fn exec_with_provider(scratch: &Scratch, arguments: &[&str], input: &[u8], reply: Reply) -> Ran {
    run(scratch, arguments, input, Some(provider(scratch, reply)))
}

#[must_use]
pub fn exec_with_foreign_file(scratch: &Scratch, arguments: &[&str], input: &[u8]) -> Ran {
    run_fixture(
        scratch,
        arguments,
        Options {
            provider: Some(provider(scratch, Reply::Accepted)),
            foreign: true,
            person: person(input),
            ..Options::default()
        },
    )
    .0
}

#[must_use]
pub fn chat_modes(scratch: &Scratch, commands: Vec<Action>) -> (Ran, Vec<skein_fake_llm_domain::api::Query>) {
    run_fixture(
        scratch,
        &[],
        Options {
            provider: Some(provider(scratch, Reply::Accepted)),
            person: (Mode::Terminal, commands),
            watching: window::Story::Modes,
            ..Options::default()
        },
    )
}

#[must_use]
pub fn chat_resized(scratch: &Scratch, commands: Vec<Action>) -> (Ran, Vec<skein_fake_llm_domain::api::Query>) {
    run_fixture(
        scratch,
        &[],
        Options {
            provider: Some(provider(scratch, Reply::Accepted)),
            person: (Mode::Terminal, commands),
            watching: window::Story::Resize,
            ..Options::default()
        },
    )
}

#[must_use]
pub fn chat_waiting(scratch: &Scratch, commands: Vec<Action>) -> Ran {
    run_fixture(
        scratch,
        &[],
        Options {
            provider: Some(provider(scratch, Reply::Wait)),
            person: (Mode::Terminal, commands),
            ..Options::default()
        },
    )
    .0
}

#[must_use]
pub fn chat_signed_in(scratch: &Scratch, commands: Vec<Action>) -> Ran {
    run_fixture(
        scratch,
        &[],
        Options {
            provider: Some(world::change_provider(credential(scratch))),
            person: (Mode::Terminal, commands),
            sign_in: true,
            ..Options::default()
        },
    )
    .0
}

fn credential(scratch: &Scratch) -> skein_llm::Credential {
    let login = std::fs::read(scratch.path().join("auth.json")).expect("scratch login");
    let document: serde_json::Value = serde_json::from_slice(&login).expect("scratch login document");
    skein_llm::Credential {
        value: document
            .get("tokens")
            .and_then(|tokens| tokens.get("access_token"))
            .and_then(serde_json::Value::as_str)
            .expect("scratch access token")
            .as_bytes()
            .into(),
        account_id: b"acc".as_slice().into(),
        scheme: skein_llm::Scheme::Bearer,
    }
}

fn provider(scratch: &Scratch, reply: Reply) -> skein_fake_peers::llm::Peer {
    let credential = credential(scratch);
    match reply {
        Reply::Wait => world::headless_provider(true, credential),
        Reply::Accepted => world::headless_provider(false, credential),
        Reply::Filtered => smith_agent_process_world::fake::configured_transport(
            Box::new([skein_fake_llm_domain::api::Script {
                cue: b"@local-shell".as_slice().into(),
                turns: Box::new([skein_fake_llm_domain::api::Turn {
                    lines: Box::new([skein_fake_llm_domain::api::Line::Text { text: b"Filtered".as_slice().into() }]),
                    finish: skein_fake_llm_domain::api::Finish::ContentFilter,
                    tokens: 1,
                }]),
            }]),
            credential,
            Duration::ZERO,
            skein_fake_peers::Transport::Tls,
        ),
    }
}

#[must_use]
pub fn run(scratch: &Scratch, arguments: &[&str], input: &[u8], provider: Option<skein_fake_peers::llm::Peer>) -> Ran {
    run_fixture(scratch, arguments, Options { provider, person: person(input), ..Options::default() }).0
}

#[must_use]
pub fn run_scripted(scratch: &Scratch, arguments: &[&str], commands: Vec<Action>) -> Ran {
    run_fixture(
        scratch,
        arguments,
        Options { person: (Mode::Pipes, commands), watching: window::Story::ProgramRun, ..Options::default() },
    )
    .0
}

/// A program chat on a terminal, with the same scratch and teardown as pipes.
#[must_use]
pub fn program_chat(scratch: &Scratch, commands: Vec<Action>) -> Ran {
    run_fixture(
        scratch,
        &[],
        Options { person: (Mode::Terminal, commands), watching: window::Story::ProgramRun, ..Options::default() },
    )
    .0
}

/// An inline terminal chat whose fixture lends a key without an account id.
#[must_use]
pub fn key_chat(scratch: &Scratch, commands: Vec<Action>) -> Ran {
    let credential = credential(scratch);
    run_fixture(
        scratch,
        &[],
        Options {
            provider: Some(world::key_provider(credential.value)),
            person: (Mode::Terminal, commands),
            ..Options::default()
        },
    )
    .0
}

/// Start the product in a scratch repository with no commit, retaining provider requests.
#[must_use]
pub fn exec_without_head(scratch: &Scratch) -> (Ran, Vec<skein_fake_llm_domain::api::Query>) {
    drop(super::refusal_program_code(
        std::path::Path::new("/usr/bin/git"),
        scratch,
        ["init", "-q"].into_iter().map(Into::into).collect(),
        0,
    ));
    run_fixture(
        scratch,
        &["exec", "hello"],
        Options { provider: Some(provider(scratch, Reply::Accepted)), ..Options::default() },
    )
}

fn person(input: &[u8]) -> (Mode, Vec<Action>) {
    (Mode::Pipes, if input.is_empty() { vec![Action::Eof] } else { vec![Action::Send(input.into()), Action::Eof] })
}

struct Options {
    provider: Option<skein_fake_peers::llm::Peer>,
    foreign: bool,
    person: (Mode, Vec<Action>),
    sign_in: bool,
    watching: window::Story,
}

impl Default for Options {
    fn default() -> Self {
        Self { provider: None, foreign: false, person: person(b""), sign_in: false, watching: window::Story::None }
    }
}

fn run_fixture(
    scratch: &Scratch,
    arguments: &[&str],
    options: Options,
) -> (Ran, Vec<skein_fake_llm_domain::api::Query>) {
    let Options { provider, foreign, person, sign_in, watching } = options;
    assert_no_children();
    let mut command_arguments: Vec<std::ffi::OsString> = arguments.iter().map(std::ffi::OsString::from).collect();
    let smith = skein_scratch::paths::binary("smith");
    let smith = smith.to_str().expect("the binary's path is UTF-8");
    let program = match watching {
        window::Story::Resize => {
            let mut setup: Vec<std::ffi::OsString> = ["-c", "stty rows 24 cols 80; exec \"$@\"", "smith-window", smith]
                .into_iter()
                .map(Into::into)
                .collect();
            setup.append(&mut command_arguments);
            command_arguments = setup;
            "/bin/sh"
        }
        window::Story::None | window::Story::Modes | window::Story::ProgramRun | window::Story::ProgramCheck => smith,
    };
    let mut binary = Binary::start(
        Command {
            program: program.into(),
            arguments: command_arguments,
            environment: super::fixture_environment(
                scratch.path(),
                vec![
                    ("HOME".into(), scratch.path().into()),
                    ("XDG_CONFIG_HOME".into(), scratch.path().join("config").into()),
                    ("XDG_STATE_HOME".into(), scratch.path().join("state").into()),
                    ("XDG_RUNTIME_DIR".into(), scratch.path().join("runtime").into()),
                    ("PATH".into(), "/usr/bin:/bin".into()),
                ],
            ),
            directory: scratch.path().into(),
        },
        person.0,
        1_048_576,
    )
    .expect("headless product pipes");
    let streams = binary.take_streams().expect("product streams");
    let descriptors = streams.descriptors();
    let clock = Clock::new();
    let now = clock.now().now;
    let master = match &streams {
        Streams::Terminal { stream } => Some(*stream),
        Streams::Pipes { .. } => None,
    };
    let mut world = real::World::new(window::Judge::new(
        Settling {
            foreign: foreign.then(|| scratch.path().join("runtime")),
            trace: arguments
                .windows(2)
                .find(|pair| pair.first().is_some_and(|flag| *flag == "--trace"))
                .map(|pair| (scratch.path().join(pair.get(1).expect("trace path")), false)),
            ..Settling::new(now, now.saturating_add(watching.budget(sign_in)))
        },
        watching,
        master,
        scratch.path(),
    ));
    world.spawn_with_fds(binary.descriptors(), || Process::Binary(Box::new(binary)));
    world.spawn_with_fds(descriptors, || {
        let terminal = match streams {
            Streams::Pipes { input, output } => Terminal::piped(input, output, person.1),
            Streams::Terminal { stream } => Terminal::attached(stream, person.1),
        };
        Process::Script(Proc::Terminal(Box::new(terminal)))
    });
    if let Some(peer) = provider {
        world.spawn(|| Process::Script(Proc::Peer(Box::new(peer))));
    }
    if sign_in {
        world.spawn(|| {
            Process::Script(Proc::Issuer(Box::new(smith_local_process_world::process::IssuerProcess::with_token(
                skein_fake_peers::Transport::Plaintext,
                34_444,
                credential(scratch).value,
            ))))
        });
        world.spawn(|| Process::Script(Proc::Browser(Box::default())));
    }
    let outcome = world.run(
        &clock,
        match watching {
            window::Story::ProgramRun | window::Story::ProgramCheck => watching.budget(sign_in),
            window::Story::None | window::Story::Modes | window::Story::Resize => window::WORLD_PATIENCE,
        },
    );
    assert!(outcome.procs.iter().all(Host::is_empty), "product and peers drain");
    let Process::Binary(binary) = outcome.procs.first().expect("binary") else { unreachable!("binary") };
    let Process::Script(Proc::Terminal(person)) = outcome.procs.get(1).expect("person") else { unreachable!("person") };
    let Some(Exit::Code(code)) = binary.exit_status() else { panic!("exit code") };
    assert_no_children();
    let queries = outcome
        .procs
        .iter()
        .find_map(|process| match process {
            Process::Script(Proc::Peer(peer)) => Some(smith_local_process_world::llm::queries(peer).cloned().collect()),
            Process::Binary(_) | Process::Script(_) | Process::Sample(_) => None,
        })
        .unwrap_or_default();
    (Ran { code, stdout: person.shown().into(), stderr: binary.stderr().into() }, queries)
}

#[must_use]
pub fn program_interrupt(scratch: &Scratch, commands: Vec<Action>) -> Ran {
    run_fixture(
        scratch,
        &["--plain"],
        Options { person: (Mode::Pipes, commands), watching: window::Story::ProgramCheck, ..Options::default() },
    )
    .0
}
