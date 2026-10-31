//! Shared card facts, acts and part slots (client/domain/objects.md, sections 2–9).
use crate::words::Words;
use crate::words::phrases::jig::JigPhrase;
use crate::{JigBindings, JigMeaning, Owner};

use skein_ui::phrases::Hole;
use skein_ui::{Builder, Role, SlotRef};

use crate::{
    Borrowed,
    state::{
        action::Act,
        fields::Purpose,
        objects::{Card, Object},
    },
};
use jig_client::contract::{ActKind, Admission, ApplicationPart, Offered, Recipient};
use skein_lib::{Id, List};
/// The phrase for every offered act, matched whole.
#[must_use]
pub const fn act_phrase(act: ActKind) -> JigPhrase {
    match act {
        ActKind::Message => JigPhrase::ActWrite,
        ActKind::Answer => JigPhrase::ActAnswer,
        ActKind::Take => JigPhrase::ActTake,
        ActKind::HandBack => JigPhrase::ActHandBack,
        ActKind::Accept => JigPhrase::ActAccept,
        ActKind::Reject => JigPhrase::ActReject,
        ActKind::PassUp => JigPhrase::ActPassUp,
        ActKind::Release => JigPhrase::ActRelease,
        ActKind::LeaveHeld => JigPhrase::ActLeaveHeld,
        ActKind::Amend => JigPhrase::ActAmend,
        ActKind::Withdraw => JigPhrase::ActWithdraw,
        ActKind::Stop => JigPhrase::ActStop,
        ActKind::Cancel => JigPhrase::ActCancel,
        ActKind::Move => JigPhrase::ActTakeOver,
        ActKind::Reorder => JigPhrase::ActReorder,
        ActKind::Seen => JigPhrase::ActMarkSeen,
        ActKind::EditNote => JigPhrase::ActEditNote,
        ActKind::DeleteNote => JigPhrase::ActDeleteNote,
        ActKind::Adopt => JigPhrase::ActAdopt,
        ActKind::RemoveProject => JigPhrase::ActRemoveProject,
        ActKind::Rename => JigPhrase::ActRename,
        ActKind::ChangePolicy => JigPhrase::ActChangePolicy,
        ActKind::SetRoles => JigPhrase::ActSetRoles,
        ActKind::MakeService => JigPhrase::ActMakeService,
        ActKind::SetPool => JigPhrase::ActSetPool,
    }
}
/// Render only document-admitted acts on cards able to open a confirmation.
pub fn acts(
    b: &mut Builder,
    card: &Card,
    rows: &List<Offered>,
    object: Id<Object>,
    words: &Words,
    bindings: &mut JigBindings,
) {
    let text_limit = bindings.text_limit();
    match card {
        Card::Open | Card::NotSent { .. } | Card::Refused { .. } => {}
        Card::Deciding { .. } | Card::Unknown { .. } | Card::Leaving { .. } => return,
    }
    if admitted(rows, ActKind::Amend) && admitted(rows, ActKind::Release) {
        crate::tree::frame::control(
            b,
            words,
            bindings,
            Role::Button,
            JigPhrase::ActAmendRelease,
            &[],
            Act::AmendRelease { object: object.token() },
        );
    }
    for offered in rows {
        if offered.act == ActKind::RemoveProject {
            continue;
        }
        match &offered.admission {
            Admission::Allowed | Admission::Proposed { .. } | Admission::Bounded { .. } => crate::tree::frame::control(
                b,
                words,
                bindings,
                Role::Button,
                act_phrase(offered.act),
                &[],
                Act::Intend { object: object.token(), act: offered.act },
            ),
            Admission::Refused(why) => {
                words.text(b, act_phrase(offered.act), &[]);
                words.refusal(b, why, text_limit);
            }
        }
    }
}
/// Leave a slot and retain the typed part that its owning view root will resolve.
pub fn part(b: &mut Builder, part: &ApplicationPart, words: &Words, bindings: &mut JigBindings) {
    let connector = match part {
        ApplicationPart::Connector { connector }
        | ApplicationPart::Resource { connector, .. }
        | ApplicationPart::Kind { connector, .. }
        | ApplicationPart::Panel { connector, .. }
        | ApplicationPart::Result { connector, .. }
        | ApplicationPart::Effect { connector, .. }
        | ApplicationPart::OwnerEffect { connector, .. }
        | ApplicationPart::Outbox { connector, .. }
        | ApplicationPart::Hold { connector, .. }
        | ApplicationPart::Project { connector, .. }
        | ApplicationPart::Deployment { connector } => *connector,
    };
    let token = bindings.insert(JigMeaning::ApplicationPart(part.clone())).token;
    b.slot(
        SlotRef { owner: (Owner::Application { number: connector }).opaque(), token },
        &words.say(JigPhrase::LoadingPart, &[]),
    );
}
/// Closing keeps only engine-owned delegate counts and the connectors it waits on.
/// The connector's words fill a name-only part; effect details remain in its task panel.
pub fn closing(b: &mut Builder, closing: &jig_client::contract::Closing, words: &Words, bindings: &mut JigBindings) {
    words.text(b, JigPhrase::Closing, &[Hole::Number(u64::from(closing.delegates))]);
    match closing.stage {
        Some(jig_client::contract::ClosingStage::Run { attempt }) => {
            words.text(b, JigPhrase::ClosingRun, &[Hole::Number(attempt)]);
        }
        Some(jig_client::contract::ClosingStage::Delegates) => words.text(b, JigPhrase::ClosingDelegates, &[]),
        Some(jig_client::contract::ClosingStage::Effects) => words.text(b, JigPhrase::ClosingStageEffects, &[]),
        Some(jig_client::contract::ClosingStage::Releases) => words.text(b, JigPhrase::ClosingReleases, &[]),
        Some(jig_client::contract::ClosingStage::Settled) => words.text(b, JigPhrase::ClosingSettled, &[]),
        None => words.text(b, JigPhrase::ClosingStageUnknown, &[]),
    }
    if let Some(ending) = &closing.ending {
        words.text(b, JigPhrase::ClosingPendingEnding, &[]);
        words.ending(b, ending);
    }
    match &closing.connectors {
        Some(connectors) => {
            if connectors.is_empty() {
                words.text(b, JigPhrase::ClosingConnectorsEmpty, &[]);
            }
            for connector in connectors {
                words.text(b, JigPhrase::ClosingConnector, &[]);
                part(b, &ApplicationPart::Connector { connector: *connector }, words, bindings);
                if closing.stage == Some(jig_client::contract::ClosingStage::Effects) {
                    words.text(b, JigPhrase::ClosingEffects, &[]);
                }
            }
        }
        None => words.text(b, JigPhrase::ClosingConnectorsUnknown, &[]),
    }
}
/// Say where an object is waiting, without deriving its policy recipient.
pub fn recipient(b: &mut Builder, to: &Recipient, words: &Words) {
    match to {
        Recipient::Task(task) => words.text(b, JigPhrase::WaitingTask, &[Hole::Number(*task)]),
        Recipient::Party(party) => words.text(b, JigPhrase::WaitingParty, &[Hole::Number(*party)]),
        Recipient::Role { project, role } => {
            words.text(b, JigPhrase::WaitingRole, &[Hole::Number(u64::from(*project)), Hole::Number(u64::from(*role))]);
        }
        Recipient::Roles { project, roles } => {
            words.text(b, JigPhrase::WaitingRoles, &[Hole::Number(u64::from(*project))]);
            for role in roles {
                words.text(b, JigPhrase::RoleNumber, &[Hole::Number(u64::from(*role))]);
            }
        }
    }
}

/// Render a form field with its domain write count; the shell preserves typing.
pub fn field(
    b: &mut Builder,
    client: &Borrowed<'_>,
    purpose: Purpose,
    phrase: JigPhrase,
    words: &Words,
    bindings: &mut JigBindings,
) {
    b.open(Role::TextArea);
    b.name(&words.say(phrase, &[]));
    if let Some(field) = client.fields().get(purpose) {
        b.value(field.text.bytes(), field.writes);
    } else {
        b.value(&[], 0);
    }
    b.bind(bindings.insert(JigMeaning::Field(purpose)));
    b.close();
}
/// Required fields and choices are visible once this object's confirmation is open.
#[must_use]
pub fn answering(client: &Borrowed<'_>, object: Id<Object>) -> bool {
    match client.confirmation() {
        Some(open) => open.object == object && open.act == ActKind::Answer,
        None => false,
    }
}
/// Display any kept durable answer even on a card whose object remains open.
pub fn answer(b: &mut Builder, object: &Object, words: &Words, text_limit: u32) {
    if let Some(answer) = &object.answer {
        match answer {
            jig_client::contract::Answer::Done(outcome) => words.outcome(b, outcome),
            jig_client::contract::Answer::Refused { refusal: why, .. } => words.refusal(b, why, text_limit),
        }
    }
}

fn admitted(rows: &List<Offered>, act: ActKind) -> bool {
    match jig_client::facts::offer(rows.as_slice(), act) {
        Some(Admission::Allowed | Admission::Proposed { .. } | Admission::Bounded { .. }) => true,
        Some(Admission::Refused(_)) | None => false,
    }
}
