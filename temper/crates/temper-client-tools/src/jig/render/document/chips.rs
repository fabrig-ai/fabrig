//! Concrete integer JSON for the client contract (contract.md, 5).
use skein_json::writer::Encoder;
pub(super) fn encode_chip(value: &jig_client::contract::Chip, out: &mut Encoder, budget: &mut super::Cut) {
    out.object_start();
    out.key(b"task");
    out.unsigned(value.task);
    out.key(b"title");
    super::encode_document_text(&value.title, out, budget);
    out.key(b"executor");
    super::encode_executor(&value.executor, out, budget);
    out.key(b"phase");
    super::encode_phase(value.phase, out, budget);
    out.key(b"details");
    encode_chip_phase(&value.details, out, budget);
    out.object_end();
}

pub(super) fn encode_chip_phase(value: &jig_client::contract::ChipPhase, out: &mut Encoder, budget: &mut super::Cut) {
    encode_chip_phase_owner(&value.phase, &value.details, out, budget);
}

fn encode_chip_phase_owner(
    phase: &jig_client::contract::OwnerChipPhase,
    details: &jig_client::contract::OwnerChipDetails,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    out.object_start();
    out.key(b"owner");
    out.object_start();
    out.key(b"phase");
    super::encode_owner_chip_phase(phase, out, budget);
    out.key(b"details");
    super::encode_owner_chip_details(details, out, budget);
    out.object_end();
    out.object_end();
}

pub(super) fn encode_closing(value: &jig_client::contract::Closing, out: &mut Encoder, budget: &mut super::Cut) {
    out.object_start();
    out.key(b"delegates");
    out.unsigned(u64::from(value.delegates));
    out.key(b"connectors");
    match &value.connectors {
        Some(item_1) => {
            out.array_start();
            for item_2 in item_1 {
                out.unsigned(u64::from(*(item_2)));
            }
            out.array_end();
        }
        None => out.null(),
    }
    out.key(b"stage");
    match &value.stage {
        Some(item_3) => {
            encode_closing_stage(item_3, out, budget);
        }
        None => out.null(),
    }
    out.key(b"ending");
    match &value.ending {
        Some(item_4) => {
            super::encode_ending(item_4, out, budget);
        }
        None => out.null(),
    }
    out.object_end();
}

pub(super) fn encode_closing_stage(
    value: &jig_client::contract::ClosingStage,
    out: &mut Encoder,
    _budget: &mut super::Cut,
) {
    match value {
        jig_client::contract::ClosingStage::Run { attempt } => {
            out.object_start();
            out.key(b"run");
            out.object_start();
            out.key(b"attempt");
            out.unsigned(*(attempt));
            out.object_end();
            out.object_end();
        }
        jig_client::contract::ClosingStage::Delegates => out.string(b"delegates"),
        jig_client::contract::ClosingStage::Effects => out.string(b"effects"),
        jig_client::contract::ClosingStage::Releases => out.string(b"releases"),
        jig_client::contract::ClosingStage::Settled => out.string(b"settled"),
    }
}
