//! The demo and fake processes around forge-setup's commands (protocol/setup.md, 7.3).
use crate::demo::Demo;
use alloc::boxed::Box;
use skein_lib::{Decimal, List, Writer, bytes};
use temper_forge_setup::{
    Group, Kind, Stage, Step, Steps,
    group::{GroupFile, GroupKind},
    setup::{Mode, Setup},
};

/// Builds a fake forge's group (temper/docs/design/protocol/setup.md, 7.1–7.3 and 8.4).
/// Process listeners come from the fake entry and demo; serve listeners stay outside the group.
#[must_use]
#[expect(clippy::boxed_local, reason = "the group builder keeps the task's owned listener signature")]
pub fn group(
    file: &GroupFile,
    demo: Option<&Demo>,
    flags: &[Box<[u8]>],
    _listeners: Box<[core::net::SocketAddr]>,
) -> Group {
    let GroupKind::Fake { llm } = file.kind else { unreachable!("fake group builder requires a fake kind") };
    let Mode::Fake(fake) = &file.entry.mode else { unreachable!("a fake group has a fake entry") };
    let mut steps = List::with_capacity(3);
    if llm {
        let demo = demo.expect("a fake group with an LLM requires its demo");
        assert_eq!(demo.forge(), file.forge, "the demo belongs to the group's forge");
        steps
            .push((
                Stage::Run,
                command(b"fake llm", &[b"fake", b"llm"], flags, None, Kind::Long)
                    .listening(Box::new([demo.llm().listen()])),
            ))
            .expect("fake llm");
    }
    steps
        .push((Stage::Prepare, command(b"prepare", &[b"forge", b"prepare"], flags, Some(file.forge), Kind::Once)))
        .expect("fake preparation");
    steps
        .push((
            Stage::Run,
            command(b"fake forge", &[b"fake", b"forge"], flags, Some(file.forge), Kind::Long)
                .listening(Box::new([fake.listen])),
        ))
        .expect("fake forge");
    let label = if llm {
        bytes::copy_of(b"Playground")
    } else {
        let digits = Decimal::of(u64::from(file.forge));
        let mut writer = Writer::new(
            6_usize
                .checked_add(digits.as_bytes().len())
                .expect("bounded group label")
                .checked_add(10)
                .expect("bounded group label"),
        );
        writer.put(b"Forge ").expect("label prefix");
        writer.put(digits.as_bytes()).expect("forge number");
        writer.put(b" (Forgejo)").expect("label kind");
        writer.finish()
    };
    Group { forge: file.forge, label, recorded: false, before: 0, steps: steps.into_boxed() }
}

/// The demo's group, the fake LLM, then the demo forge's preparation
/// and its fake forge (`application.demo.forge`, 8.4), is always asked,
/// and writes no record: the demo's state is the engine's alone (8.4),
/// so a box started again runs it only when `temper serve` asks, which
/// serve does at the person's choice and at each of its starts while the
/// demo runs. Its `started` follows its long steps' start, as the steps'
/// order takes them (7.2). A fake forge other than the demo's runs in its
/// place, unasked.
/// The caller's first steps precede these steps and every asked group's insertion point (7.3).
#[must_use]
#[expect(clippy::manual_map, reason = "step code uses explicit matches, never closures")]
pub fn steps(
    setup: &Setup,
    demo: Option<&Demo>,
    flags: &[Box<[u8]>],
    listeners: Box<[core::net::SocketAddr]>,
    forgejo_listens: bool,
    first: Box<[Step]>,
) -> Steps {
    let given = temper_forge_setup::steps(setup, flags, listeners, forgejo_listens);
    let demo_forge = match demo {
        Some(demo) => Some(demo.forge()),
        None => None,
    };
    let first_len = u32::try_from(first.len()).expect("bounded first steps");
    let mut prefix = first_len;
    for entry in &setup.entries {
        if let Mode::Fake(_) = &entry.mode
            && demo_forge != Some(entry.forge)
        {
            prefix = prefix.checked_add(2).expect("bounded fake steps");
        }
    }
    let capacity =
        u32::try_from(given.always.len()).expect("bounded steps").checked_add(prefix).expect("bounded prefix");
    let mut always = List::with_capacity(capacity);
    for step in first {
        always.push(step).expect("first steps counted");
    }
    for entry in &setup.entries {
        if let Mode::Fake(_) = &entry.mode
            && demo_forge != Some(entry.forge)
        {
            let file = GroupFile { forge: entry.forge, kind: GroupKind::Fake { llm: false }, entry: entry.clone() };
            for (_, step) in group(&file, None, flags, Box::new([])).steps {
                always.push(step).expect("fake steps counted");
            }
        }
    }
    for step in given.always {
        always.push(step).expect("forge-setup steps counted");
    }
    let mut asked = List::with_capacity(
        u32::try_from(given.asked.len())
            .expect("bounded groups")
            .checked_add(u32::from(demo.is_some()))
            .expect("demo group"),
    );
    if let Some(demo) = demo {
        let file = GroupFile {
            forge: demo.forge(),
            kind: GroupKind::Fake { llm: true },
            entry: setup.entry(demo.forge()).expect("checked demo forge").clone(),
        };
        let mut built = group(&file, Some(demo), flags, Box::new([]));
        built.before = first_len;
        asked.push(built).expect("demo group counted");
    }
    for mut group in given.asked {
        group.before = group.before.checked_add(prefix).expect("bounded group position");
        asked.push(group).expect("owned groups counted");
    }
    Steps { always: always.into_boxed(), asked: asked.into_boxed() }
}

fn command(name: &[u8], words: &[&[u8]], flags: &[Box<[u8]>], forge: Option<u16>, kind: Kind) -> Step {
    let capacity = u32::try_from(words.len())
        .expect("command words")
        .checked_add(u32::try_from(flags.len()).expect("bounded flags"))
        .expect("bounded arguments")
        .checked_add(u32::from(forge.is_some()))
        .expect("forge number");
    let mut arguments = List::with_capacity(capacity);
    for word in words {
        arguments.push(bytes::copy_of(word)).expect("command words counted");
    }
    for flag in flags {
        arguments.push(flag.clone()).expect("flags counted");
    }
    let name = match forge {
        Some(number) => {
            let digits = Decimal::of(u64::from(number));
            let number = digits.as_bytes();
            arguments.push(bytes::copy_of(number)).expect("forge number counted");
            let length =
                name.len().checked_add(1).expect("name and space").checked_add(number.len()).expect("bounded name");
            let mut writer = Writer::new(length);
            writer.put(name).expect("name counted");
            writer.put(b" ").expect("space counted");
            writer.put(number).expect("number counted");
            writer.finish()
        }
        None => bytes::copy_of(name),
    };
    Step::new(name, arguments.into_boxed(), kind)
}
