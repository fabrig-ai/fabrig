//! A hold's printed way out uses ctl's shipped grammar.
use jig_client_world::harness::{ClientId, Settings, Until, World};
use jig_domain_tasks as tasks;
use skein_fake_person::{Find, Step};
use skein_world::domain::Verdict;
use temper_client_tools::jig::{Printed, call::Parsed};
use temper_client_world::{app::Temper, fixtures, front::Joint};

type ClientWorld = World<Temper, Joint>;

fn call(world: &mut ClientWorld, agent: ClientId, words: &[&[u8]]) -> Printed {
    let number = fixtures::give(world, agent, words);
    let answer = fixtures::answered(world, agent, number);
    assert_eq!(answer.status, 0, "call {words:?}: {answer:?}");
    answer
}

fn offered(world: &mut ClientWorld, agent: ClientId, task: u64) -> Vec<Vec<String>> {
    let panel = call(world, agent, &[b"change", b"show", task.to_string().as_bytes(), b"--project", b"1"]);
    let task_read = call(world, agent, &[b"task", b"show", task.to_string().as_bytes()]);
    let text = std::str::from_utf8(&panel.stdout).expect("panel text");
    let reason = text.lines().find_map(|line| line.strip_prefix("held: ")).expect("held panel");
    let (cancel, commands) = reason.split_once("; grant more tasks: ").expect("amend command");
    let cancel = cancel.split_once("; cancel: ").expect("cancel command").1;
    let (amend, release) = commands.split_once("; then release it: ").expect("release command");
    let release = release.split_once("; or ask afresh; bare release holds again").expect("release's consequence").0;
    let task_text = std::str::from_utf8(&task_read.stdout).expect("task text");
    assert!(task_text.contains("its 0 tasks are made"), "the cause survives the summary's cut: {task_text}");
    assert!(task_text.contains(cancel), "task read offers the same cancellation: {task_text}");
    assert_eq!(reason.matches("task ").count(), 3, "every offered command is checked: {reason}");
    [release, cancel, amend].into_iter().map(command_words).collect()
}

fn command_words(command: &str) -> Vec<String> {
    let words: Vec<String> = command
        .split_ascii_whitespace()
        .map(|word| if word == "REASON" { "because" } else { word }.to_owned())
        .collect();
    let argv: Vec<&[u8]> = words.iter().map(String::as_bytes).collect();
    assert!(matches!(temper_client_tools::table::parse(&argv), Ok(Parsed::Call { .. })), "{command}");
    words
}

#[test]
fn a_holds_offered_commands_are_ones_ctl_takes() {
    let (mut world, agent, task) = tasks_hold();
    call(&mut world, agent, &[b"task", b"show", task.to_string().as_bytes(), b"--until", b"held"]);
    let before = world.front().engine.task(task).expect("held change").clone();
    assert!(matches!(before.phase, tasks::Phase::Held { .. }));
    offered(&mut world, agent, task);
    let hold = before.revisions.hold.to_string();
    call(&mut world, agent, &[b"task", b"release", task.to_string().as_bytes(), b"--hold", hold.as_bytes()]);
    call(&mut world, agent, &[b"task", b"show", task.to_string().as_bytes(), b"--until", b"held"]);
    let after = world.front().engine.task(task).expect("held again");
    assert!(matches!(after.phase, tasks::Phase::Held { .. }));
    assert!(after.revisions.hold > before.revisions.hold);
    assert_eq!(after.made, before.made, "bare release makes no delegate");
    let printed = call(&mut world, agent, &[b"change", b"show", task.to_string().as_bytes(), b"--project", b"1"]);
    assert!(printed.stdout.windows(b"released without more tasks".len()).any(|w| w == b"released without more tasks"));
    let commands = offered(&mut world, agent, task);
    let cancel: Vec<&[u8]> = commands[1].iter().map(String::as_bytes).collect();
    call(&mut world, agent, &cancel);
    let ended = call(&mut world, agent, &[b"task", b"show", task.to_string().as_bytes(), b"--until", b"ended"]);
    assert!(ended.stdout.windows(b"Cancelled".len()).any(|w| w == b"Cancelled"), "{ended:?}");
    assert!(world.front().engine.task(task).is_none(), "cancel ends the change");
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
}

#[test]
fn the_long_specs_amend_hint_and_forge_connect_hint_parse() {
    let words = fixtures::phrasebooks(&fixtures::domain_limits());
    let hint = words.say(temper_client_view::words::Word::SpecTooLong, &[]);
    let hint = std::str::from_utf8(&hint).expect("spec hint");
    let command = hint.split_once("`temper ctl DIR ").expect("ctl command").1.split('`').next().expect("command");
    let forge = temper_client_domain::forge::words::app_terminal(1);
    let forge = std::str::from_utf8(&forge).expect("forge hint");
    let connect = forge.split_once("temper ctl DIR ").expect("ctl command").1.split(',').next().expect("command");
    for command in [command, connect] {
        let argv: Vec<&[u8]> = command
            .split_ascii_whitespace()
            .map(|word| match word {
                "TASK" | "REV" | "ID" | "KEY" => b"1".as_slice(),
                "FILE" => b"spec.txt".as_slice(),
                "REASON" => b"because".as_slice(),
                word => word.as_bytes(),
            })
            .collect();
        assert!(matches!(temper_client_tools::table::parse(&argv), Ok(Parsed::Call { .. })), "{command}");
    }
}

fn refused_change(open: bool) -> (ClientWorld, u64) {
    let mut front = Joint::with_change_landing(fixtures::SEED, &fixtures::domain_limits());
    front.engine.forge =
        Some(temper_engine_domain_world::forge_stand::ForgeStand::with_checks(fixtures::REPOSITORIES, &[b"build"]));
    fixtures::adopt_change_repository(&mut front);
    front.engine.forge.as_mut().expect("fake forge").push_branch(7, b"temper/refused");
    let task = if open {
        fixtures::goal_with_pushed_change_without_open_grant(&mut front, b"temper/refused")
    } else {
        fixtures::goal_with_pushed_change_without_review_authority(&mut front, b"temper/refused")
    };
    (
        World::new(Settings { seed: fixtures::SEED, words: fixtures::decision_words(), ..Settings::default() }, front),
        task,
    )
}

fn task_commands(world: &mut ClientWorld, agent: ClientId, task: u64, expected: &[u8]) -> Vec<Vec<String>> {
    let printed = call(world, agent, &[b"task", b"show", task.to_string().as_bytes()]);
    assert!(printed.stdout.windows(expected.len()).any(|word| word == expected), "{printed:?}");
    let text = std::str::from_utf8(&printed.stdout).expect("ctl text");
    let line = text.lines().find_map(|line| line.strip_prefix("way out: ")).expect("way out");
    let release = &line[line.find("task release ").expect("release command")..];
    let (release, cancel) = release.split_once("; or cancel it: ").expect("cancel command");
    [release, cancel].into_iter().map(command_words).collect()
}

#[test]
fn refused_batch_and_vetoed_effect_cards_and_ctl_name_the_reason_and_offer_working_commands() {
    for (open, expected) in
        [(false, b"executor outside its delegation authority".as_slice()), (true, b"open pull request")]
    {
        let (mut world, task) = refused_change(open);
        let agent = fixtures::agent(&mut world);
        for _ in 0..100 {
            let until = world.now().saturating_add(skein_lib::Duration::from_millis(100));
            assert_eq!(world.run(Until::Time(until)).verdict, Verdict::Passed);
            if matches!(world.front().engine.task(task).expect("change").phase, tasks::Phase::Held { .. }) {
                break;
            }
        }
        assert!(
            matches!(world.front().engine.task(task).expect("change").phase, tasks::Phase::Held { .. }),
            "open={open}, task {:?}",
            world.front().engine.task(task)
        );
        call(&mut world, agent, &[b"task", b"show", task.to_string().as_bytes(), b"--until", b"held"]);
        let client = world.client(fixtures::PARTY, fixtures::task_page(task));
        world.person(client, vec![Step::See { find: Find::text(expected), within: fixtures::WITHIN }]);
        assert_eq!(world.run(Until::Done).verdict, Verdict::Passed, "hold card names its effect or reason");
        // The next actions are in ctl; leave the person's waiting card explicitly.
        world.go(client, fixtures::chats_page());
        let commands = task_commands(&mut world, agent, task, expected);
        call(&mut world, agent, &commands[0].iter().map(String::as_bytes).collect::<Vec<_>>());
        call(&mut world, agent, &[b"task", b"show", task.to_string().as_bytes(), b"--until", b"held"]);
        let commands = task_commands(&mut world, agent, task, expected);
        call(&mut world, agent, &commands[1].iter().map(String::as_bytes).collect::<Vec<_>>());
        call(&mut world, agent, &[b"task", b"show", task.to_string().as_bytes(), b"--until", b"ended"]);
        assert!(world.front().engine.task(task).is_none());
    }
}

fn tasks_hold() -> (ClientWorld, ClientId, u64) {
    let mut front = Joint::with_change_landing(fixtures::SEED, &fixtures::domain_limits());
    front.engine.forge =
        Some(temper_engine_domain_world::forge_stand::ForgeStand::with_checks(fixtures::REPOSITORIES, &[b"build"]));
    fixtures::adopt_change_repository(&mut front);
    front.engine.forge.as_mut().expect("fake forge").push_branch(7, b"temper/held");
    let task = fixtures::goal_with_pushed_change_with_tasks(&mut front, b"temper/held", 0);
    let mut world = World::<Temper, Joint>::new(
        Settings { seed: fixtures::SEED, words: fixtures::decision_words(), ..Settings::default() },
        front,
    );
    assert!(world.front().engine.task(task).is_some(), "created change exists");
    let agent = fixtures::agent(&mut world);
    for _ in 0..100 {
        let until = world.now().saturating_add(skein_lib::Duration::from_millis(100));
        assert_eq!(world.run(Until::Time(until)).verdict, Verdict::Passed);
        if matches!(world.front().engine.task(task).expect("live change").phase, tasks::Phase::Held { .. }) {
            break;
        }
    }
    (world, agent, task)
}

#[test]
fn a_printed_tasks_amendment_and_release_admit_a_fresh_review() {
    let (mut world, agent, task) = tasks_hold();
    call(&mut world, agent, &[b"task", b"show", task.to_string().as_bytes(), b"--until", b"held"]);
    let before = world.front().engine.task(task).expect("held change").clone();
    let commands = offered(&mut world, agent, task);
    let amend: Vec<&[u8]> = commands[2].iter().map(String::as_bytes).collect();
    let number = fixtures::give(&mut world, agent, &amend);
    assert_eq!(fixtures::answered(&mut world, agent, number).status, 1, "the pool has no widening room");
    call(&mut world, agent, &[b"project", b"fund", b"1", b"1000", b"--role", b"0", b"--key", b"fund-widening"]);
    let amended = call(&mut world, agent, &amend);
    assert!(amended.stdout.starts_with(b"done:"), "{amended:?}");
    let widened = world.front().engine.task(task).expect("amended change").clone();
    assert!(widened.revisions.spec > before.revisions.spec);
    let mut authority = before.authority.clone();
    authority.delegation.tasks = widened.authority.delegation.tasks;
    assert_eq!(widened.authority, authority, "every other authority control is preserved");
    assert!(widened.authority.delegation.tasks > before.authority.delegation.tasks);
    assert_eq!(widened.made, before.made);
    let commands = offered(&mut world, agent, task);
    let release: Vec<&[u8]> = commands[0].iter().map(String::as_bytes).collect();
    call(&mut world, agent, &release);
    for _ in 0..100 {
        let until = world.now().saturating_add(skein_lib::Duration::from_millis(100));
        assert_eq!(world.run(Until::Time(until)).verdict, Verdict::Passed);
        if world.front().engine.task(task).expect("released change").made > before.made {
            break;
        }
    }
    let after = world.front().engine.task(task).expect("released change").clone();
    assert_eq!(after.made, before.made + 1, "a review is admitted without resetting the lifetime count");
    assert!(!matches!(after.phase, tasks::Phase::Held { .. }));
}
