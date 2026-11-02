//! The demo and fake processes around forge-setup's commands (protocol/setup.md, 7.3).
use alloc::boxed::Box;
use skein_lib::{Decimal, List, Writer, bytes};
use temper_forge_setup::{
    Group, Kind, Stage, Step,
    machine::{BoxKind, FakeMachine},
};

/// The fake kind's slot: the playground's fakes, bound to a forge by the box
/// (temper/docs/design/protocol/setup.md, 7.1).
#[must_use]
pub fn fake_slot(machine: &FakeMachine, llm: core::net::SocketAddr, flags: &[Box<[u8]>]) -> Group {
    let forge = Some(temper_forge_setup::step::FORGE_ARGUMENT);
    Group {
        kind: Some(BoxKind::Fake),
        remover: Some(argument_command(b"remove", &[b"forge", b"remove"], flags, forge, Kind::Once)),
        forge: 0,
        label: bytes::copy_of(b"Playground"),
        recorded: false,
        before: 0,
        steps: Box::new([
            (Stage::Run, command(b"fake llm", &[b"fake", b"llm"], flags, None, Kind::Long).listening(Box::new([llm]))),
            (Stage::Prepare, argument_command(b"prepare", &[b"forge", b"prepare"], flags, forge, Kind::Once)),
            (
                Stage::Run,
                argument_command(b"fake forge", &[b"fake", b"forge"], flags, forge, Kind::Long)
                    .listening(Box::new([machine.listen])),
            ),
        ]),
    }
}

fn command(name: &[u8], words: &[&[u8]], flags: &[Box<[u8]>], forge: Option<u16>, kind: Kind) -> Step {
    match forge {
        Some(number) => argument_command(name, words, flags, Some(Decimal::of(u64::from(number)).as_bytes()), kind),
        None => argument_command(name, words, flags, None, kind),
    }
}

fn argument_command(name: &[u8], words: &[&[u8]], flags: &[Box<[u8]>], forge: Option<&[u8]>, kind: Kind) -> Step {
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
