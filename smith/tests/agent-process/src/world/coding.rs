//! A native coding fixture with shipped standard limits and a fake check
//! program. It observes files at the check's spawn, never domain state.
//! Contract: smith/docs/design/domain/run.md, section 8.

use super::{Machine, Parent, World};
use skein_fake_machine::{How, Item, Opened};
use skein_lib::{List, Queue, Reader, Writer};

// A coding model's output allowance, as in the standard native coding setup.
pub(super) const OUTPUT: u32 = 32_768;
const CHECK: &[u8] = b".smith-test/check";

pub(super) fn configuration() -> smith_agent_service::Config {
    let profile = smith_agent_service::profile::standard();
    let mut declarations = crate::declarations(0);
    declarations.endpoints[0].models[0].output = OUTPUT;
    let limits = smith_agent_service::profile::derive(&profile, &declarations).expect("standard coding limits");
    let mut config = crate::configuration_with_limits(skein_llm::Provider::OpenAiCodex, &limits);
    config.domain.models[0].output = OUTPUT;
    config.domain.models[0].protocol_allowance = smith_agent_service::profile::endpoint_allowance(
        &profile,
        &declarations.endpoints[0],
        &limits.domain.session.tools,
    )
    .expect("standard endpoint allowance");
    config
}

impl World {
    /// Adopt a writable workspace with standard production-derived coding limits.
    pub fn standard_workspace(&mut self) {
        self.coding = true;
    }

    /// File contents seen by each fake check program at its spawn.
    #[must_use]
    pub fn checked_files(&self) -> &[Vec<u8>] {
        &self.outcome.as_ref().expect("settled coding scenario").machine.checked_files
    }

    /// Read a final file beneath the coding fixture's first startup root.
    pub fn workspace_file(&mut self, path: &[u8]) -> Vec<u8> {
        let machine = &mut self.outcome.as_mut().expect("settled coding scenario").machine.files;
        // The parent's root is zero; the first adopted workspace root is one.
        let root = machine.reopen_root(1);
        let file = machine.open(root, path, How::ReadNoFollow).expect("workspace file");
        let size = u32::try_from(machine.stat(file).size).expect("fixture file size");
        let bytes = machine.read(file, 0, size).expect("final file bytes");
        machine.close(file);
        machine.close(root);
        bytes
    }
}

pub(super) fn start(parent: &mut Parent, charter: &[u8]) {
    let frame = crate::start_workspace(charter, 8, true);
    let frame = crate::wire::edit_start(&frame, |parts| {
        let mut workspace = parts.workspace.take().expect("workspace").into_parts();
        let directory = workspace.directories.get_mut(0).expect("coding root");
        let mut root = directory.clone().into_parts();
        root.writable = true;
        *directory = smith_channel::Directory::new(&smith_channel::CEILINGS, root).expect("writable root");
        parts.workspace = Some(smith_channel::Workspace::new(&smith_channel::CEILINGS, workspace).expect("workspace"));
    });
    parent.peer = crate::host_with_limits(configuration().limits.channel);
    parent.peer.play(frame);
    parent.coding_deliveries = Some(0);
}

pub(super) fn answer_delivery(parent: &mut Parent) {
    let Some(answered) = parent.coding_deliveries else { return };
    let calls: Vec<_> =
        parent.peer.observed().iter().filter(|frame| frame.kind == 0x0107).skip(answered).cloned().collect();
    for frame in calls {
        let limits = &smith_channel::CEILINGS;
        let call = smith_channel::Call::decode(limits, &mut Reader::new(&frame.body)).expect("delivery call");
        assert!(matches!(call.ask(), smith_channel::Ask::Deliver(_)), "the coding host only receives delivery");
        let mut receipts = List::with_capacity(1);
        receipts
            .push(
                smith_channel::Receipt::new(
                    limits,
                    smith_channel::ReceiptParts { directory: 0, text: b"checked files kept".as_slice().into() },
                )
                .expect("receipt"),
            )
            .expect("one writable root");
        let delivered =
            smith_channel::Delivered::new(limits, smith_channel::DeliveredParts { receipts }).expect("delivered");
        let reply = smith_channel::DeliveryReply::new(
            limits,
            smith_channel::DeliveryReplyParts { value: smith_channel::Delivery::Delivered(delivered) },
        )
        .expect("delivery reply");
        let answer = smith_channel::HostAnswer::new(
            limits,
            smith_channel::HostAnswerParts { name: call.name().clone(), reply: smith_channel::Reply::Delivery(reply) },
        )
        .expect("host delivery answer");
        let mut body = Writer::new(usize::try_from(answer.measure()).expect("answer size"));
        answer.encode(&mut body).expect("answer bytes");
        let mut frame = skein_channel::frame_writer(0x0102, answer.measure()).expect("answer frame");
        frame.put(&body.finish()).expect("answer body");
        parent.peer.play(frame.finish().expect("answer frame"));
        parent.coding_deliveries = Some(parent.coding_deliveries.expect("coding host") + 1);
    }
}

pub(super) fn files() -> Vec<Item> {
    vec![
        Item::file(b"original.txt", b"original\n"),
        Item::directory(b".smith-test"),
        Item::file(CHECK, b"#!/bin/sh\n").mode(0o700),
    ]
}

pub(super) fn check(machine: &mut Machine, call: &skein_sim::Call, answers: &mut Queue<skein_sim::Answer>) -> bool {
    let skein_sim::Ask::Spawn { root, program, args, .. } = &call.ask else { return false };
    if program.as_ref() != CHECK {
        return false;
    }
    assert!(args.is_empty(), "the check runs directly without shell arguments");
    let file = machine.files.open(Opened::new(root.raw()), b"result.txt", How::ReadNoFollow).expect("written result");
    let size = u32::try_from(machine.files.stat(file).size).expect("fixture file size");
    machine.checked_files.push(machine.files.read(file, 0, size).expect("check reads the written file"));
    machine.files.close(file);
    answers.push(skein_sim::Answer {
        ticket: call.ticket,
        result: Ok(skein_sim::Reply::Program(skein_sim::Program::Exit(0))),
    });
    true
}
