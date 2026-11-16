//! Print and read the joint world's terminal pages (client/line.md, 5).
use jig_client_kit::harness::{ClientId, Until, World};
use skein_ui_text::{Command, Print};
use skein_world::domain::Verdict;
use temper_client_world::{app::Temper, fixtures, front::Joint};

type TerminalWorld = World<Temper, Joint>;

fn print(world: &TerminalWorld, client: ClientId, limits: &temper_client_domain::Limits) -> Print {
    let words = fixtures::phrasebooks(limits);
    let bounds = fixtures::view_bounds(limits, &words);
    let mut print = Print::new(&bounds.tree).expect("terminal print bounds");
    skein_ui_text::render(world.tree(client), b"temper", None, &mut print);
    print
}

pub fn printed(world: &TerminalWorld, client: ClientId, limits: &temper_client_domain::Limits) -> String {
    let print = print(world, client, limits);
    String::from_utf8(print.text.as_slice().to_vec()).expect("printed UTF-8")
}

/// Lines without their Enter, dispatched as the terminal composition dispatches them.
pub fn line(world: &mut TerminalWorld, client: ClientId, bytes: &[u8]) -> String {
    let limits = fixtures::terminal::reading_limits();
    let print = print(world, client, &limits);
    let command = skein_ui_text::command(
        bytes,
        Some(print.numbers.as_slice()),
        world.tree(client),
        temper_client_shell_terminal::limits::terminal().line,
    );
    match command {
        Command::Act { node, first, submit } => {
            world.act(client, node, first);
            if submit {
                world.act(client, node, skein_ui::Act::Submit);
            }
        }
        command => panic!(
            "expected a terminal act for {bytes:?}, got {command:?}: {}",
            String::from_utf8_lossy(print.text.as_slice())
        ),
    }
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
    printed(world, client, &limits)
}
