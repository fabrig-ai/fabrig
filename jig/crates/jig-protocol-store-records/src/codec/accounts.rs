//! The accounts family's rows (protocol/store/README.md, 4.4): the installation's
//! identifier, and the accounts child's counter, accounts, sources and facts (item 9).
use crate::accounts_v2 as wire;
use alloc::boxed::Box;
use jig_domain_accounts as domain;
use skein_lib::{Duration, List, Reader, Wall, Writer};

/// The installation's identifier, drawn at the first start (configuration.md, 7).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Installation {
    pub id: [u8; 16],
}

impl Installation {
    /// Encode this row with the accounts family's version.
    #[must_use]
    pub fn encode(&self) -> Box<[u8]> {
        let row = wire::Installation::new(&wire::CEILINGS, wire::InstallationParts { id: self.id })
            .expect("a fixed identifier has no bound to break");
        let mut writer = Writer::new(skein_lib::length::usize_of(row.measure()));
        row.encode(&mut writer).expect("measured room");
        writer.finish()
    }

    /// Decode one row, refusing another version or trailing bytes.
    pub fn decode(bytes: &[u8]) -> Result<Self, wire::Problem> {
        let wire::InstallationParts { id } =
            wire::Installation::decode(&wire::CEILINGS, &mut Reader::new(bytes))?.into_parts();
        Ok(Self { id })
    }
}

/// Encode one of the accounts child's records (protocol/store/README.md 4.4 item 9) within the
/// schema's ceilings, which are the child's own bounds.
pub fn encode(row: &domain::Stored) -> Result<Box<[u8]>, wire::Problem> {
    let limits = &wire::CEILINGS;
    let value = match row {
        domain::Stored::Facts { account, facts } => {
            let mut values = List::with_capacity(domain::FACTS);
            for fact in facts {
                if values.push(encode_fact(fact)?).is_err() {
                    return Err(wire::Problem { path: wire::Path::FactsFacts, reason: skein_codec::Reason::Bound });
                }
            }
            wire::Value::Facts(wire::Facts::new(limits, wire::FactsParts { account: *account, facts: values })?)
        }
        domain::Stored::Issued { number, holder, name, signed_out } => {
            let holder = match holder {
                domain::Holder::Deployment => wire::Holder::Deployment,
                domain::Holder::Party(party) => {
                    wire::Holder::Party(wire::Party::new(limits, wire::PartyParts { party: *party })?)
                }
            };
            wire::Value::Issued(wire::Issued::new(
                limits,
                wire::IssuedParts { number: *number, holder, name: name.clone(), signed_out: *signed_out },
            )?)
        }
        domain::Stored::Next { number } => {
            wire::Value::Next(wire::Next::new(limits, wire::NextParts { number: *number })?)
        }
        domain::Stored::Account { number, provider, holder, place, uses, subject, display, list } => {
            let holder = match holder {
                domain::Holder::Deployment => wire::Holder::Deployment,
                domain::Holder::Party(party) => {
                    wire::Holder::Party(wire::Party::new(limits, wire::PartyParts { party: *party })?)
                }
            };
            wire::Value::Account(wire::Account::new(
                limits,
                wire::AccountParts {
                    number: *number,
                    provider: *provider,
                    holder,
                    place: *place,
                    uses: encode_uses(uses.as_deref())?,
                    subject: subject.clone(),
                    display: display.clone(),
                    list: encode_list(*list)?,
                },
            )?)
        }
        domain::Stored::Source { account, number, place, kind, name, held } => {
            let kind = match kind {
                domain::Kind::SignIn => wire::Kind::SignIn,
                domain::Kind::Key => wire::Kind::Key,
                domain::Kind::Program => wire::Kind::Program,
                domain::Kind::None => wire::Kind::None,
            };
            let held = match held {
                None => None,
                Some(domain::Held::Limited { reset }) => {
                    let reset = match reset {
                        Some(reset) => Some(reset.as_nanos()),
                        None => None,
                    };
                    Some(wire::Held::Limited(wire::Limited::new(limits, wire::LimitedParts { reset })?))
                }
                Some(domain::Held::Refused) => Some(wire::Held::Refused),
                Some(domain::Held::Moving) => Some(wire::Held::Moving),
            };
            wire::Value::Source(wire::Source::new(
                limits,
                wire::SourceParts { account: *account, number: *number, place: *place, kind, name: name.clone(), held },
            )?)
        }
    };
    let row = wire::Row::new(limits, wire::RowParts { value })?;
    let mut writer = Writer::new(skein_lib::length::usize_of(row.measure()));
    row.encode(&mut writer).expect("measured room");
    Ok(writer.finish())
}

/// Decode one of the accounts child's records, refusing another version or
/// trailing bytes. The child checks the record's rules as it restores it.
pub fn decode(bytes: &[u8]) -> Result<domain::Stored, wire::Problem> {
    let wire::RowParts { value } = wire::Row::decode(&wire::CEILINGS, &mut Reader::new(bytes))?.into_parts();
    Ok(match value {
        wire::Value::Facts(facts) => {
            let wire::FactsParts { account, facts } = facts.into_parts();
            let mut values = List::with_capacity(domain::FACTS);
            for fact in facts.into_boxed() {
                values.push(decode_fact(fact)?).expect("schema bounds facts");
            }
            domain::Stored::Facts { account, facts: values.into_boxed() }
        }
        wire::Value::Issued(issued) => {
            let wire::IssuedParts { number, holder, name, signed_out } = issued.into_parts();
            let holder = match holder {
                wire::Holder::Deployment => domain::Holder::Deployment,
                wire::Holder::Party(party) => domain::Holder::Party(party.into_parts().party),
            };
            domain::Stored::Issued { number, holder, name, signed_out }
        }
        wire::Value::Next(next) => {
            let wire::NextParts { number } = next.into_parts();
            domain::Stored::Next { number }
        }
        wire::Value::Account(account) => {
            let wire::AccountParts { number, provider, holder, place, uses, subject, display, list } =
                account.into_parts();
            let holder = match holder {
                wire::Holder::Deployment => domain::Holder::Deployment,
                wire::Holder::Party(party) => domain::Holder::Party(party.into_parts().party),
            };
            domain::Stored::Account {
                number,
                provider,
                holder,
                place,
                uses: decode_uses(uses),
                subject,
                display,
                list: decode_list(list),
            }
        }
        wire::Value::Source(source) => {
            let wire::SourceParts { account, number, place, kind, name, held } = source.into_parts();
            let kind = match kind {
                wire::Kind::SignIn => domain::Kind::SignIn,
                wire::Kind::Key => domain::Kind::Key,
                wire::Kind::Program => domain::Kind::Program,
                wire::Kind::None => domain::Kind::None,
            };
            let held = match held {
                None => None,
                Some(wire::Held::Limited(limited)) => {
                    let reset = match limited.into_parts().reset {
                        Some(reset) => Some(Wall::from_nanos(reset)),
                        None => None,
                    };
                    Some(domain::Held::Limited { reset })
                }
                Some(wire::Held::Refused) => Some(domain::Held::Refused),
                Some(wire::Held::Moving) => Some(domain::Held::Moving),
            };
            domain::Stored::Source { account, number, place, kind, name, held }
        }
    })
}

fn encode_uses(uses: Option<&[domain::Use]>) -> Result<Option<List<wire::Use>>, wire::Problem> {
    let Some(uses) = uses else { return Ok(None) };
    let mut result = List::with_capacity(domain::USES);
    for payer in uses {
        let payer = match payer {
            domain::Use::Party(party) => {
                wire::Use::Party(wire::Party::new(&wire::CEILINGS, wire::PartyParts { party: *party })?)
            }
            domain::Use::Project(project) => {
                wire::Use::Project(wire::Project::new(&wire::CEILINGS, wire::ProjectParts { project: *project })?)
            }
        };
        if result.push(payer).is_err() {
            return Err(wire::Problem { path: wire::Path::AccountUses, reason: skein_codec::Reason::Bound });
        }
    }
    Ok(Some(result))
}
fn decode_uses(uses: Option<List<wire::Use>>) -> Option<Box<[domain::Use]>> {
    let uses = uses?;
    let mut result = List::with_capacity(domain::USES);
    for payer in uses.into_boxed() {
        let payer = match payer {
            wire::Use::Party(party) => domain::Use::Party(party.into_parts().party),
            wire::Use::Project(project) => domain::Use::Project(project.into_parts().project),
        };
        result.push(payer).expect("schema bounds uses");
    }
    Some(result.into_boxed())
}

fn encode_list(value: Option<domain::ListRecord>) -> Result<Option<wire::ListRecord>, wire::Problem> {
    let Some(value) = value else { return Ok(None) };
    let failure = match value.failure {
        None => None,
        Some(failure) => Some(match failure {
            domain::ReadFailure::Refused { status } => {
                wire::ReadFailure::Refused(wire::Refused::new(&wire::CEILINGS, wire::RefusedParts { status })?)
            }
            domain::ReadFailure::Unreachable => wire::ReadFailure::Unreachable,
            domain::ReadFailure::Unreadable => wire::ReadFailure::Unreadable,
            domain::ReadFailure::Deadline => wire::ReadFailure::Deadline,
            domain::ReadFailure::Agent => wire::ReadFailure::Agent,
            domain::ReadFailure::Room => wire::ReadFailure::Room,
        }),
    };
    Ok(Some(wire::ListRecord::new(
        &wire::CEILINGS,
        wire::ListRecordParts {
            names: value.names,
            read: match value.read {
                Some(at) => Some(at.as_nanos()),
                None => None,
            },
            tried: match value.tried {
                Some(at) => Some(at.as_nanos()),
                None => None,
            },
            failure,
            left_out: value.left_out,
        },
    )?))
}
fn decode_list(value: Option<wire::ListRecord>) -> Option<domain::ListRecord> {
    let value = value?.into_parts();
    Some(domain::ListRecord {
        names: value.names,
        read: match value.read {
            Some(at) => Some(Wall::from_nanos(at)),
            None => None,
        },
        tried: match value.tried {
            Some(at) => Some(Wall::from_nanos(at)),
            None => None,
        },
        failure: match value.failure {
            None => None,
            Some(failure) => Some(match failure {
                wire::ReadFailure::Refused(value) => domain::ReadFailure::Refused { status: value.into_parts().status },
                wire::ReadFailure::Unreachable => domain::ReadFailure::Unreachable,
                wire::ReadFailure::Unreadable => domain::ReadFailure::Unreadable,
                wire::ReadFailure::Deadline => domain::ReadFailure::Deadline,
                wire::ReadFailure::Agent => domain::ReadFailure::Agent,
                wire::ReadFailure::Room => domain::ReadFailure::Room,
            }),
        },
        left_out: value.left_out,
    })
}

fn encode_fact(fact: &domain::AccountFact) -> Result<wire::Fact, wire::Problem> {
    let limits = &wire::CEILINGS;
    let value = match &fact.value {
        domain::FactValue::Text(text) => {
            wire::FactValue::Text(wire::FactText::new(limits, wire::FactTextParts { text: text.clone() })?)
        }
        domain::FactValue::Count(count) => {
            wire::FactValue::Count(wire::FactCount::new(limits, wire::FactCountParts { count: *count })?)
        }
        domain::FactValue::Instant(at) => {
            wire::FactValue::Instant(wire::FactInstant::new(limits, wire::FactInstantParts { at: at.as_nanos() })?)
        }
        domain::FactValue::Span(span) => {
            wire::FactValue::Span(wire::FactSpan::new(limits, wire::FactSpanParts { nanos: span.as_nanos() })?)
        }
        domain::FactValue::Share(basis) => {
            wire::FactValue::Share(wire::FactShare::new(limits, wire::FactShareParts { basis: *basis })?)
        }
        domain::FactValue::Window(window) => {
            let length = match window.length {
                Some(length) => Some(length.as_nanos()),
                None => None,
            };
            let resets = match window.resets {
                Some(resets) => Some(resets.as_nanos()),
                None => None,
            };
            wire::FactValue::Window(wire::FactWindow::new(
                limits,
                wire::FactWindowParts { length, used: window.used, resets, state: window.state },
            )?)
        }
    };
    wire::Fact::new(
        limits,
        wire::FactParts {
            kind: fact.kind,
            label: fact.label.clone(),
            value,
            seen: fact.seen.as_nanos(),
            source: fact.source,
            origin: fact.origin,
        },
    )
}

fn decode_fact(fact: wire::Fact) -> Result<domain::AccountFact, wire::Problem> {
    let wire::FactParts { kind, label, value, seen, source, origin } = fact.into_parts();
    let value = match value {
        wire::FactValue::Text(text) => domain::FactValue::Text(text.into_parts().text),
        wire::FactValue::Count(count) => domain::FactValue::Count(count.into_parts().count),
        wire::FactValue::Instant(at) => domain::FactValue::Instant(Wall::from_nanos(at.into_parts().at)),
        wire::FactValue::Span(span) => domain::FactValue::Span(Duration::from_nanos(span.into_parts().nanos)),
        wire::FactValue::Share(share) => {
            let basis = share.into_parts().basis;
            if basis > domain::SHARE_WHOLE {
                return Err(wire::Problem { path: wire::Path::FactShareBasis, reason: skein_codec::Reason::Bound });
            }
            domain::FactValue::Share(basis)
        }
        wire::FactValue::Window(window) => {
            let wire::FactWindowParts { length, used, resets, state } = window.into_parts();
            if let Some(used) = used
                && used > domain::SHARE_WHOLE
            {
                return Err(wire::Problem { path: wire::Path::FactWindowUsed, reason: skein_codec::Reason::Bound });
            }
            let length = match length {
                Some(length) => Some(Duration::from_nanos(length)),
                None => None,
            };
            let resets = match resets {
                Some(resets) => Some(Wall::from_nanos(resets)),
                None => None,
            };
            domain::FactValue::Window(domain::FactWindow { length, used, resets, state })
        }
    };
    Ok(domain::AccountFact { kind, label, value, seen: Wall::from_nanos(seen), source, origin })
}
