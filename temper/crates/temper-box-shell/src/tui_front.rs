//! The TUI started in front of a box (temper/docs/design/client/tui/box.md).
use crate::directories;
use skein_io::kernel::{Error, Exit, Fd};
use skein_lib::Secret;
use skein_shell::{Clock, Config, Host, Input, Kernel, Keys, Watch};
use skein_ui_screen_terminal::LEAVE;
use std::{
    io::Read,
    net::SocketAddr,
    os::unix::{ffi::OsStrExt, fs::PermissionsExt},
};
use temper_client_shell_tui::{
    limits,
    run::{Front, Run, Settings},
    start as tui,
};

pub(crate) fn handed(bytes: &[u8]) -> Option<Secret> {
    if bytes.is_empty() || bytes.len() > 4096 {
        return None;
    }
    Some(Secret::new(bytes.into()))
}

pub(crate) fn run(argument: &[u8], clock: &Clock, environment: &[crate::EnvironmentEntry], working: &[u8]) -> u8 {
    let result = (|| {
        let engine = tui::address(argument).map_err(|why| why.line("temper"))?;
        let link = (|| {
            let input = std::fs::File::open("/dev/fd/3").ok()?;
            let mut bytes = Vec::new();
            input.take(4097).read_to_end(&mut bytes).ok()?;
            handed(&bytes)
        })();
        skein_shell::close_startup_fd(Fd::new(3));
        let link = link.ok_or_else(|| "temper internal tui: fd 3: expected the local link\n".to_string())?;
        let terminal = skein_shell::open_terminal().map_err(terminal_refusal)?;
        let ready = skein_shell::take_foreground(terminal).and_then(|()| skein_shell::terminal_modes(terminal));
        if let Err(why) = ready {
            skein_shell::close_startup_fd(terminal);
            return Err(terminal_refusal(why));
        }
        start(engine, link, terminal, clock, environment, working)
    })();
    match result {
        Ok(code) => code,
        Err(why) => {
            eprint!("{why}");
            1
        }
    }
}

fn terminal_refusal(why: Error) -> String {
    format!("temper internal tui: terminal: {why:?}\n")
}

fn start(
    engine: SocketAddr,
    link: Secret,
    terminal: Fd,
    clock: &Clock,
    environment: &[crate::EnvironmentEntry],
    working: &[u8],
) -> Result<u8, String> {
    let directory = tui::directory(environment).map_err(|why| why.line("temper"))?;
    let user = skein_shell::effective_user();
    directories::make_private(&directory, user).map_err(|why| tui::Refused::Directory(why).line("temper"))?;
    let saved = skein_shell::read_private_file(&directory.join("address"), 64, user)
        .ok()
        .and_then(|bytes| tui::kept(bytes.expose()));
    let link_file = tui::link_path(environment, working);
    let note = tui::read_note(&directory, engine, user);
    let scheme = tui::scheme(environment);
    let depth = tui::depth(environment);
    let input = skein_shell::raw_input(terminal, Keys::Pass).map_err(terminal_refusal)?;
    let (kept, size) = match input {
        Input::Terminal { kept, size } => (kept, size),
        Input::NotATerminal => return Err("temper internal tui: terminal: not a terminal\n".into()),
    };
    skein_fail_stop::before_report(Some(kept.on_abort(terminal, LEAVE)));
    let parent = directory.parent().expect("TUI directory has a parent");
    let result = (|| -> Result<_, String> {
        let signals = skein_shell::open_signals(Watch::TerminationAndWindow).expect("TUI signals");
        let root = skein_shell::open_root(parent).map_err(|number| {
            skein_shell::close_startup_fd(signals);
            tui::Refused::Directory(format!("{}: cannot open its parent: errno {number}", directory.display()))
                .line("temper")
        })?;
        let files = skein_shell::open_root(parent).map_err(|number| {
            skein_shell::close_startup_fd(root);
            skein_shell::close_startup_fd(signals);
            tui::Refused::Directory(format!("{}: cannot open sign-in parent: errno {number}", directory.display()))
                .line("temper")
        })?;
        let record = if tui::recording(environment) {
            Some(skein_shell::open_root(parent).map_err(|number| {
                skein_shell::close_startup_fd(files);
                skein_shell::close_startup_fd(root);
                skein_shell::close_startup_fd(signals);
                tui::Refused::Directory(format!(
                    "{}: cannot open recording parent: errno {number}",
                    directory.display()
                ))
                .line("temper")
            })?)
        } else {
            None
        };
        let settings = Settings {
            record,
            front: Some(Front { link, lines: Fd::new(4) }),
            engine: skein_io::Addr::Ip(engine),
            directory: Box::from(b"tui".as_slice()),
            address: engine.to_string().into_bytes().into_boxed_slice(),
            scheme,
            boxed: true,
            link_file: link_file.map(|path| path.as_os_str().as_bytes().into()),
            note,
            files: Some(files),
            depth,
            size: skein_ui_screen::Size { rows: size.rows, columns: size.columns },
            limits: limits::tui(),
            opener: tui::opener(environment, cfg!(target_os = "linux"), &|path| {
                std::fs::metadata(path)
                    .is_ok_and(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
            }),
        };
        let seed = skein_shell::seed().expect("kernel seed");
        let mut run = local_run(settings, root, user, seed, terminal, signals);
        let mut kernel = Kernel::open(Config { operations: run.operations() }).expect("TUI kernel");
        let exit = skein_shell::drive(&mut kernel, clock, &mut run);
        Ok((run, exit, seed))
    })();
    skein_fail_stop::before_report(None);
    kept.restore().expect("restore terminal modes");
    let (run, exit, seed) = result?;
    if run.refused() {
        return Err(tui::Refused::Directory(format!("{}: cannot open its files", directory.display())).line("temper"));
    }
    if let Some(signed_in) = run.signed_in() {
        let mut bytes = b"http://".to_vec();
        bytes.extend_from_slice(signed_in);
        if tui::kept(&bytes) != saved {
            tui::keep_address(parent, bytes.into_boxed_slice(), user, seed, clock);
        }
    }
    Ok(match exit {
        Exit::Code(code) => code,
        Exit::Signal(_) => 1,
    })
}

fn local_run(mut settings: Settings, root: Fd, user: u32, seed: u64, terminal: Fd, signals: Fd) -> Run {
    let link = settings.link_file.take();
    let run = Run::new(settings, root, user, seed, terminal, terminal, signals).expect("TUI descriptors");
    let Some(path) = link else { return run };
    let Some(parent) = crate::path(&path).parent() else { return run };
    let Ok(root) = skein_shell::open_root(parent) else { return run };
    run.with_local_link(root, &path)
}
