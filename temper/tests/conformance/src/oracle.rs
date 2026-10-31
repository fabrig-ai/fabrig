//! Public readback through the conformance harness (temper testing.md, 2).
use crate::{Event, Temper};
use jig_conformance::{Harness, referee::Violation};
use jig_domain_core::{documents, reads};
use skein_lib::{ReplyTo, Token};
use std::collections::BTreeMap;
use temper_engine_domain::{Delivery, Event as Input, PartyInput};
use temper_engine_domain_forge as forge;
use temper_engine_domain_world::oracle;

const REPLY: u64 = 0x4e34_0000;

#[derive(Default)]
pub(super) struct Replies {
    reads: BTreeMap<u64, reads::Answer>,
    documents: BTreeMap<u64, forge::Document>,
}
impl Replies {
    #[expect(clippy::wildcard_enum_match_arm, reason = "only oracle read and watch replies are captured")]
    pub(super) fn receive(&mut self, delivery: &Delivery) {
        match delivery {
            Delivery::Read { reply, answer } if *reply == ReplyTo::new(Token::new(REPLY)) => {
                self.reads.insert(REPLY, answer.clone());
            }
            Delivery::ForgeDocument { watcher, document } if watcher.raw() >= REPLY => {
                self.documents.insert(watcher.raw(), *document.clone());
            }
            Delivery::Panel { watcher, panel } if watcher.raw() >= REPLY => {
                self.documents.insert(watcher.raw(), forge::Document::Change(*panel.clone()));
            }
            _ => {}
        }
    }
}

fn read(world: &mut Harness<Temper>, sign_in: u64, read: reads::Read) -> Result<reads::Answer, Violation> {
    world.send(Event::Root(Input::Party(PartyInput::Read {
        reply_to: ReplyTo::new(Token::new(REPLY)),
        sign_in,
        read,
    })));
    world.drain()?;
    Ok(world.peers.oracle.reads.remove(&REPLY).expect("oracle read answered through the harness"))
}

fn document(
    world: &mut Harness<Temper>,
    sign_in: u64,
    project: u32,
    subject: u64,
) -> Result<Option<forge::Document>, Violation> {
    let (configuration, limits) = crate::config::root(world.peers.seed, world.peers.scenario, world.peers.story);
    let watcher = Token::new(REPLY);
    world.send(Event::Root(Input::Party(PartyInput::Watch {
        watcher,
        sign_in,
        subject: documents::Watched::Application {
            scope: jig_domain_people::ApplicationScope::Project(project),
            connector: configuration.core.connectors[0],
            subject: Token::new(subject),
        },
        window: documents::Window { rows: limits.documents.rows, after: None },
    })));
    world.drain()?;
    let answer = world.peers.oracle.documents.remove(&REPLY);
    world.send(Event::Root(Input::Party(PartyInput::Unwatch { watcher })));
    world.drain()?;
    Ok(answer)
}

pub(super) fn close(world: &mut Harness<Temper>) -> Result<(), Violation> {
    let mut snapshot = oracle::Snapshot::new(&world.store.rows, skein_lib::Wall::from_nanos(world.clock().now));
    for (sign_in, query) in &snapshot.queries {
        snapshot.answers.push(read(world, *sign_in, query.clone())?);
    }
    let subjects = oracle::forge_subjects(&world.store.rows);
    let mut documents = Vec::new();
    for (sign_in, project, subject) in &subjects {
        documents.push(document(world, *sign_in, *project, *subject)?);
    }
    let forges = world.domain.root.forges();
    world.crash()?;
    world.drain()?;
    oracle::compare_forges(forges, world.domain.root.forges());
    for (index, (sign_in, query)) in snapshot.queries.iter().enumerate() {
        snapshot.compare_answer(index, read(world, *sign_in, query.clone())?);
    }
    for ((sign_in, project, subject), before) in subjects.into_iter().zip(documents) {
        oracle::compare_forge(project, subject, before, document(world, sign_in, project, subject)?);
    }
    Ok(())
}
