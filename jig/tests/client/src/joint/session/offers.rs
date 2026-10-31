//! Observations of engine and component offered lists (client/domain/testing.md, 5).
//! These ordinary observations never decide or manufacture an offer.
use super::{CardOffers, Incoming};
use crate::ClientId;
use jig_client_domain::contract as client;
use jig_client_test_domain::Event;
use jig_domain_core::{documents, reads};
use std::collections::BTreeMap;

pub(super) type Captured = BTreeMap<(ClientId, client::ObjectKey), CardOffers>;

pub(super) fn read(captured: &mut Option<Captured>, source: &reads::Answer, incoming: &Incoming) {
    let Some(captured) = captured else { return };
    let Incoming::Callback {
        client,
        event:
            Event::Client {
                event:
                    jig_client::Event::Up {
                        up:
                            jig_client::Up::ReadDone {
                                result: jig_client::Readout::Jig(client::ReadResult::Value(native)),
                                ..
                            },
                    },
            },
    } = incoming
    else {
        return;
    };
    match (source, native) {
        (reads::Answer::Document(source), native) => {
            let (key, original, received) = match (source.as_ref(), native) {
                (documents::Document::Proposal(source), client::ReadValue::OwnerProposal(native)) => {
                    (client::ObjectKey::Proposal(native.key), &source.acts, &native.acts)
                }
                (documents::Document::Escalation(source), client::ReadValue::OwnerEscalation(native)) => {
                    (client::ObjectKey::Escalation(source.task.chip.number), &source.acts, &native.acts)
                }
                (documents::Document::PersonTask(source), client::ReadValue::OwnerPersonTask(native)) => {
                    (client::ObjectKey::PersonTask(source.task.number), &source.acts, &native.acts)
                }
                (documents::Document::Question(source), client::ReadValue::Question(native)) => (
                    client::ObjectKey::Question { task: source.task.number, message: source.message },
                    &source.acts,
                    &native.acts,
                ),
                (documents::Document::Result(source), client::ReadValue::Result(native)) => {
                    (client::ObjectKey::Result(source.task.number), &source.acts, &native.acts)
                }
                _ => return,
            };
            captured.insert((*client, key), CardOffers { source: original.clone(), native: received.clone() });
        }
        (reads::Answer::Page(page), client::ReadValue::Notes(native)) => {
            for row in &page.rows {
                let reads::PageRow::Note(source) = row else { continue };
                let project = match source.scope {
                    documents::NoteLocation::Deployment => 0,
                    documents::NoteLocation::Project(project)
                    | documents::NoteLocation::Goal { project, .. }
                    | documents::NoteLocation::Resources { project, .. } => project,
                };
                let received =
                    native.rows.iter().find(|row| row.number == source.number).expect("same admitted note identity");
                captured.insert(
                    (*client, client::ObjectKey::Note { project, note: source.number }),
                    CardOffers { source: source.acts.clone(), native: received.acts.clone() },
                );
            }
        }
        _ => {}
    }
}
