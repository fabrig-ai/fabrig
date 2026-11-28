//! Same-decision assembly of the application's client documents.
use crate::domain::Work;
use crate::{ClientDocument, ClientMissing, ClientSubject, Domain};
use alloc::boxed::Box;
use jig_ops_domain_connectors::{Connector, connector, number};
use jig_ops_domain_infrastructure as infrastructure;
use jig_ops_domain_observability as observability;
use skein_lib::{Queue, Token};

pub(crate) fn prepare(token: Token, subject: ClientSubject, work: &mut Queue<Work>) -> u16 {
    match subject {
        ClientSubject::Infrastructure(subject) => {
            work.push(Work::Infrastructure(infrastructure::Event::DocumentPrepare { token, subject }));
            number(Connector::Infrastructure)
        }
        ClientSubject::Observability(subject) => {
            work.push(Work::Observability(observability::Event::DocumentPrepare { token, subject }));
            number(Connector::Observability)
        }
    }
}

pub(crate) fn forget(number: u16, subject: Token, work: &mut Queue<Work>) {
    match connector(number) {
        Some(Connector::Infrastructure) => {
            work.push(Work::Infrastructure(infrastructure::Event::DocumentForget { subject }));
        }
        Some(Connector::Observability) => {
            work.push(Work::Observability(observability::Event::DocumentForget { subject }));
        }
        None => unreachable!("configured connector"),
    }
}

pub(crate) fn take(domain: &mut Domain, number: u16, subject: Token) -> ClientDocument {
    match connector(number) {
        Some(Connector::Infrastructure) => ClientDocument::Infrastructure(Box::new(
            domain.infrastructure.take_document(subject).expect("ready document belongs to its connector"),
        )),
        Some(Connector::Observability) => ClientDocument::Observability(Box::new(
            domain.observability.take_document(subject).expect("ready document belongs to its connector"),
        )),
        None => unreachable!("configured connector"),
    }
}

pub(crate) fn infrastructure(
    answer: Result<infrastructure::ClientDocument, infrastructure::DocumentMissing>,
) -> Result<ClientDocument, ClientMissing> {
    match answer {
        Ok(document) => Ok(ClientDocument::Infrastructure(Box::new(document))),
        Err(infrastructure::DocumentMissing::Unknown) => Err(ClientMissing::Unknown),
        Err(infrastructure::DocumentMissing::NotAllowed) => Err(ClientMissing::NotAllowed),
    }
}

pub(crate) fn observability(
    answer: Result<observability::ClientDocument, observability::DocumentMissing>,
) -> Result<ClientDocument, ClientMissing> {
    match answer {
        Ok(document) => Ok(ClientDocument::Observability(Box::new(document))),
        Err(observability::DocumentMissing::Unknown) => Err(ClientMissing::Unknown),
        Err(observability::DocumentMissing::NotAllowed) => Err(ClientMissing::NotAllowed),
    }
}
