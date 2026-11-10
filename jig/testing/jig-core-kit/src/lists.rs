//! An account list's worker seam (jig/docs/design/domain/engine/settings.md, 12).
use crate::world::World;
use jig_domain_accounts::ProviderList;
use jig_domain_core as core;
use jig_domain_fleet as fleet;
use jig_domain_link::{Grant, ListOutcome, ListedModel};
use jig_test_domain as root;
use skein_lib::Token;

#[derive(Clone, Debug)]
pub struct Reading {
    pub channel: Token,
    pub read: u64,
    pub endpoint: u32,
    pub grant: Grant,
}

/// Give a fixture provider a list, with its configured endpoint unchanged.
pub fn set_list(config: &mut root::Config, provider: u32, keep: &[&[u8]], drop: &[&[u8]]) {
    let provider = config
        .core
        .settings
        .llm_providers
        .providers
        .iter_mut()
        .find(|row| row.number == provider)
        .expect("fixture provider");
    provider.list = Some(ProviderList {
        keep: keep.iter().map(|pattern| Box::from(*pattern)).collect(),
        drop: drop.iter().map(|pattern| Box::from(*pattern)).collect(),
    });
}

#[must_use]
pub fn listed(names: &[&[u8]]) -> ListOutcome {
    ListOutcome::Listed(
        names
            .iter()
            .map(|name| ListedModel {
                name: Box::from(*name),
                shown: None,
                window: None,
                output: None,
                efforts: Box::new([]),
            })
            .collect(),
    )
}

impl World {
    /// Answer the latest placed read by the worker hub's event.
    pub fn answer_list(&mut self, outcome: ListOutcome) {
        let reading = self.list_reads.last().expect("worker received a list read");
        self.send(root::Event::Core(core::Event::Fleet(fleet::Event::ListRead {
            channel: reading.channel,
            read: reading.read,
            outcome,
        })));
    }
}
