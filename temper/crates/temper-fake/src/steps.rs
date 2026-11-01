//! The demo and fake processes around forge-setup's commands (protocol/setup.md, 7.3).
use alloc::boxed::Box;
use core::net::SocketAddr;
use skein_lib::{Decimal, List, Writer, bytes};
use temper_forge_setup::{
    Group, Kind, Stage, Step, Steps,
    setup::{Entry, Fake, Mode},
};

/// The playground's recorded group, before position zero (protocol/setup.md, 8.4; notes contracts T3).
#[must_use]
pub fn playground_group(forge: u16, fake: &Fake, llm: SocketAddr, flags: &[Box<[u8]>]) -> Group {
    Group {
        forge,
        label: bytes::copy_of(b"Playground"),
        recorded: true,
        before: 0,
        steps: Box::new([
            (Stage::Run, command(b"fake llm", &[b"fake", b"llm"], flags, None, Kind::Long).listening(Box::new([llm]))),
            (Stage::Prepare, command(b"prepare", &[b"forge", b"prepare"], flags, Some(forge), Kind::Once)),
            (
                Stage::Run,
                command(b"fake forge", &[b"fake", b"forge"], flags, Some(forge), Kind::Long)
                    .listening(Box::new([fake.listen])),
            ),
        ]),
    }
}
/// Picks one recorded group by the entry's mode (notes contracts T3).
#[must_use]
pub fn group(entry: &Entry, llm: Option<SocketAddr>, flags: &[Box<[u8]>], forgejo_listens: bool) -> Option<Group> {
    match &entry.mode {
        Mode::Owned(owned) => Some(temper_forge_setup::step::owned_group(entry.forge, owned, flags, forgejo_listens)),
        Mode::Fake(fake) => Some(playground_group(entry.forge, fake, llm?, flags)),
        Mode::Managed { .. } => None,
    }
}

/// The caller's fake backend steps, followed by serve. Groups are loaded separately from their files.
#[must_use]
pub fn steps(flags: &[Box<[u8]>], listeners: Box<[SocketAddr]>, first: Box<[Step]>) -> Steps {
    let given = temper_forge_setup::steps(flags, listeners);
    let capacity = u32::try_from(first.len()).expect("bounded first steps").checked_add(1).expect("serve step");
    let mut always = List::with_capacity(capacity);
    for step in first {
        always.push(step).expect("first steps counted");
    }
    for step in given.always {
        always.push(step).expect("serve counted");
    }
    Steps { always: always.into_boxed(), asked: Box::new([]) }
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
