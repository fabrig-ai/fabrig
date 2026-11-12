//! The delivery log, not outgoing posts (testing/fakes/github.md, 1.7).
use super::{Entry, Store};
use crate::{
    Limits,
    image::{
        Refusal,
        codec::{Encoder, count, data, flag, num, optional_data, optional_num},
    },
};
use skein_forge::github::{
    delivery::{Account, Change, Ci, Delivery, Pull},
    log,
};
use skein_lib::{List, Reader, Rng, Wall, usize_of};

#[derive(Debug)]
pub(crate) struct State {
    entries: List<Option<Entry>>,
    rng: Rng,
    next: u64,
    revision: u64,
}
impl Store {
    pub(crate) fn encode_image(&self, out: &mut Encoder) {
        out.num(self.rng.state());
        out.num(self.next);
        out.num(self.revision);
        out.num(u64::from(self.entries.len()));
        for entry in self.entries.iter().flatten() {
            let row = &entry.row;
            out.num(row.id);
            out.bytes(&row.guid);
            out.bytes(&row.event);
            out.optional_data(row.action.as_deref());
            out.optional_num(row.installation_id);
            out.optional_num(row.repository_id);
            out.num(u64::from(row.status_code));
            out.num(row.delivered_at.as_nanos());
            out.flag(row.redelivery);
            write_payload(out, &entry.payload);
            out.bytes(&entry.signature);
            out.num(entry.visible.as_nanos());
            out.flag(entry.listed);
        }
    }
    pub(crate) fn decode_image(&self, input: &mut Reader<'_>, limits: &Limits) -> Result<State, Refusal> {
        let rng = Rng::new(num(input, "delivery rng")?);
        let next = num(input, "next delivery")?;
        let revision = num(input, "delivery revision")?;
        if next == 0 || next > 9_999_999_999_999_999_999 {
            return Err(Refusal::Malformed("next delivery"));
        }
        let cap = self.config.log_entries.min(limits.deliveries.entries);
        let length = count(input, cap, "delivery entries")?;
        let mut entries = List::with_capacity(cap);
        let mut prior = 0;
        for _ in 0..length {
            let id = num(input, "delivery id")?;
            if id <= prior || id >= next {
                return Err(Refusal::Malformed("delivery id"));
            }
            prior = id;
            let guid =
                data(input, 16, "delivery guid")?.as_ref().try_into().or(Err(Refusal::Malformed("delivery guid")))?;
            let event = data(input, usize_of(limits.deliveries.payload_bytes), "delivery event")?;
            let action = optional_data(input, usize_of(limits.deliveries.payload_bytes), "delivery action")?;
            let installation_id = optional_num(input, "delivery installation")?;
            let repository_id = optional_num(input, "delivery repository")?;
            let status_code =
                u16::try_from(num(input, "delivery status")?).or(Err(Refusal::Malformed("delivery status")))?;
            let delivered_at = Wall::from_nanos(num(input, "delivery time")?);
            let redelivery = flag(input, "redelivery")?;
            let payload = read_payload(input, usize_of(limits.deliveries.payload_bytes))?;
            // Record admits an encoded payload under this bound, including padding.
            let encode_limits = skein_forge::github::limits::Limits {
                body_bytes: limits.deliveries.payload_bytes,
                string_bytes: limits.deliveries.payload_bytes,
                writer: skein_json::writer::Limits { length: limits.deliveries.payload_bytes, ..limits.github.writer },
                ..limits.github
            };
            let sent = skein_forge::github::server::encode(&payload, &encode_limits)
                .or(Err(Refusal::Limits("delivery payload")))?;
            if sent.event != event.as_ref() {
                return Err(Refusal::Malformed("delivery event"));
            }
            let signature = data(input, 71, "delivery signature")?
                .as_ref()
                .try_into()
                .or(Err(Refusal::Malformed("delivery signature")))?;
            let visible = Wall::from_nanos(num(input, "delivery visibility")?);
            let listed = flag(input, "delivery listed")?;
            entries
                .push(Some(Entry {
                    row: log::Row {
                        id,
                        guid,
                        event,
                        action,
                        installation_id,
                        repository_id,
                        status_code,
                        delivered_at,
                        redelivery,
                    },
                    payload,
                    signature,
                    visible,
                    listed,
                }))
                .expect("admitted delivery entries");
        }
        Ok(State { entries, rng, next, revision })
    }
    pub(crate) fn restore_image(&mut self, state: State) {
        self.entries = state.entries;
        self.rng = state.rng;
        self.next = state.next;
        self.revision = state.revision;
        self.closed = false;
    }
}

fn write_payload(out: &mut Encoder, payload: &Delivery) {
    match payload {
        Delivery::Issue { repository, number, body } => {
            out.num(0);
            out.num(*repository);
            out.num(*number);
            match body {
                skein_forge::text::Body::Kept(text) => out.bytes(text),
                skein_forge::text::Body::Long(_) => unreachable!("record refuses long bodies"),
            }
        }
        Delivery::Comment { repository, number, body } => {
            out.num(1);
            out.num(*repository);
            out.num(*number);
            match body {
                skein_forge::text::Body::Kept(text) => out.bytes(text),
                skein_forge::text::Body::Long(_) => unreachable!("record refuses long bodies"),
            }
        }
        Delivery::Pull { repository, number, action } => {
            out.num(2);
            out.num(*repository);
            out.num(*number);
            out.bytes(&action.action);
            out.optional_data(action.reason.as_deref());
        }
        Delivery::Review { repository, number, body } => {
            out.num(3);
            out.num(*repository);
            out.num(*number);
            match body {
                skein_forge::text::Body::Kept(text) => out.bytes(text),
                skein_forge::text::Body::Long(_) => unreachable!("record refuses long bodies"),
            }
        }
        Delivery::Remark { repository, number } => {
            out.num(4);
            out.num(*repository);
            out.num(*number);
        }
        Delivery::Push { repository, branch, after, deleted, sender } => {
            out.num(5);
            out.num(*repository);
            out.bytes(branch);
            out.bytes(after);
            out.flag(*deleted);
            out.num(*sender);
        }
        Delivery::Ref { repository, branch, created } => {
            out.num(6);
            out.num(*repository);
            out.bytes(branch);
            out.flag(*created);
        }
        Delivery::Ci { repository, commit, event } => {
            out.num(7);
            out.num(*repository);
            out.bytes(commit);
            out.num(match event {
                Ci::CheckRun => 0,
                Ci::CheckSuite => 1,
                Ci::Status => 2,
                Ci::WorkflowJob => 3,
                Ci::WorkflowRun => 4,
                Ci::ChecksRequested => 5,
                Ci::Destroyed => 6,
            });
        }
        Delivery::Installation { installation, account, change } => {
            out.num(8);
            out.num(*installation);
            out.bytes(&account.login);
            out.num(account.id);
            match change {
                Change::Created => out.num(0),
                Change::Deleted => out.num(1),
                Change::Suspended => out.num(2),
                Change::Unsuspended => out.num(3),
                Change::Repositories => out.num(4),
                Change::Renamed { login } => {
                    out.num(5);
                    out.bytes(login);
                }
            }
        }
        Delivery::Other => {
            out.num(9);
        }
    }
}
fn read_payload(input: &mut Reader<'_>, bound: usize) -> Result<Delivery, Refusal> {
    Ok(match num(input, "delivery kind")? {
        0 => Delivery::Issue {
            repository: num(input, "delivery repository")?,
            number: num(input, "delivery number")?,
            body: skein_forge::text::Body::Kept(data(input, bound, "delivery body")?),
        },
        1 => Delivery::Comment {
            repository: num(input, "delivery repository")?,
            number: num(input, "delivery number")?,
            body: skein_forge::text::Body::Kept(data(input, bound, "delivery body")?),
        },
        2 => Delivery::Pull {
            repository: num(input, "delivery repository")?,
            number: num(input, "delivery number")?,
            action: Pull {
                action: data(input, bound, "pull action")?,
                reason: optional_data(input, bound, "pull reason")?,
            },
        },
        3 => Delivery::Review {
            repository: num(input, "delivery repository")?,
            number: num(input, "delivery number")?,
            body: skein_forge::text::Body::Kept(data(input, bound, "delivery body")?),
        },
        4 => {
            Delivery::Remark { repository: num(input, "delivery repository")?, number: num(input, "delivery number")? }
        }
        5 => Delivery::Push {
            repository: num(input, "delivery repository")?,
            branch: data(input, bound, "delivery branch")?,
            after: data(input, 32, "delivery after")?
                .as_ref()
                .try_into()
                .or(Err(Refusal::Malformed("delivery after")))?,
            deleted: flag(input, "delivery deleted")?,
            sender: num(input, "delivery sender")?,
        },
        6 => Delivery::Ref {
            repository: num(input, "delivery repository")?,
            branch: data(input, bound, "delivery branch")?,
            created: flag(input, "delivery created")?,
        },
        7 => Delivery::Ci {
            repository: num(input, "delivery repository")?,
            commit: data(input, 32, "delivery commit")?
                .as_ref()
                .try_into()
                .or(Err(Refusal::Malformed("delivery commit")))?,
            event: match num(input, "delivery event")? {
                0 => Ci::CheckRun,
                1 => Ci::CheckSuite,
                2 => Ci::Status,
                3 => Ci::WorkflowJob,
                4 => Ci::WorkflowRun,
                5 => Ci::ChecksRequested,
                6 => Ci::Destroyed,
                _ => return Err(Refusal::Malformed("delivery event")),
            },
        },
        8 => Delivery::Installation {
            installation: num(input, "delivery installation")?,
            account: Account {
                login: data(input, bound, "delivery account login")?,
                id: num(input, "delivery account id")?,
            },
            change: match num(input, "delivery change")? {
                0 => Change::Created,
                1 => Change::Deleted,
                2 => Change::Suspended,
                3 => Change::Unsuspended,
                4 => Change::Repositories,
                5 => Change::Renamed { login: data(input, bound, "renamed login")? },
                _ => return Err(Refusal::Malformed("delivery change")),
            },
        },
        9 => Delivery::Other,
        _ => return Err(Refusal::Malformed("delivery kind")),
    })
}
