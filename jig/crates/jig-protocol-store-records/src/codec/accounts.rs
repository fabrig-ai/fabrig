//! The accounts family's rows (protocol/store/README.md, 4.4): the installation's
//! identifier, and the accounts child's counter, accounts, sources and facts (item 9).
use crate::accounts_v2::{self as wire, Record};
use crate::generated::accounts as maps;
use alloc::boxed::Box;
use jig_domain_accounts as domain;
use skein_lib::Reader;

/// The installation's identifier, drawn at the first start (configuration.md, 7).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Installation {
    pub id: [u8; 16],
}

impl Installation {
    /// Encode this row with the accounts family's version.
    #[must_use]
    pub fn encode(&self) -> Box<[u8]> {
        let row = maps::installation_to_record(self, &()).expect("an identifier maps infallibly");
        row.encode(&wire::CEILINGS).expect("a fixed identifier has no bound to break")
    }

    /// Decode one row, refusing another version or trailing bytes.
    pub fn decode(bytes: &[u8]) -> Result<Self, wire::Problem> {
        let row = wire::Installation::decode(&wire::CEILINGS, &mut Reader::new(bytes))?;
        maps::installation_to_domain(row, &())
    }
}

/// Encode an accounts child record within its schema ceilings.
pub fn encode(row: &domain::Stored) -> Result<Box<[u8]>, wire::Problem> {
    wire::Row { value: maps::value_to_record(row, &())? }.encode(&wire::CEILINGS)
}

/// Decode an accounts child record; the child checks its restoration rules.
pub fn decode(bytes: &[u8]) -> Result<domain::Stored, wire::Problem> {
    let wire::Row { value } = wire::Row::decode(&wire::CEILINGS, &mut Reader::new(bytes))?;
    maps::value_to_domain(value, &())
}
