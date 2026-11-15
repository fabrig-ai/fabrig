//! Search evidence shown to clients (temper/docs/design/domain/landing-search-by-test.md, 8).
use alloc::boxed::Box;
use core::mem::size_of_val;
use temper_engine_domain_forge_landing::Combine;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct LandingRound {
    pub branch: Box<[u8]>,
    pub pull: Option<u64>,
    pub upstream: Option<[u8; 32]>,
    pub combined: Option<Combine>,
    pub failure: Box<[u8]>,
    pub search: Option<LandingSearch>,
}
impl LandingRound {
    #[must_use]
    pub fn new(branch: Box<[u8]>) -> Self {
        Self { branch, pull: None, upstream: None, combined: None, failure: Box::default(), search: None }
    }
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct LandingSearch {
    pub green: u16,
    pub red: u16,
    pub tests: LandingSearchTests,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum LandingSearchTests {
    Whole,
    ByTest { gates: Box<[SearchSelection]>, passes: Box<[SearchPass]> },
    WholeAgain,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct SearchSelection {
    pub gate: u16,
    pub name: Box<[u8]>,
    pub count: u32,
    pub ids: Box<[Box<[u8]>]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct SearchPass {
    pub member: u16,
    pub prefix: [u8; 32],
    pub gate: u16,
    pub name: Box<[u8]>,
    pub run: u64,
}
pub(super) fn search_bytes(search: Option<&LandingSearch>) -> usize {
    let Some(search) = search else { return 0 };
    match &search.tests {
        LandingSearchTests::Whole | LandingSearchTests::WholeAgain => 0,
        LandingSearchTests::ByTest { gates, passes } => {
            let mut bytes = size_of_val(gates.as_ref()).saturating_add(size_of_val(passes.as_ref()));
            for gate in gates {
                bytes = bytes.saturating_add(gate.name.len()).saturating_add(size_of_val(gate.ids.as_ref()));
                for id in &gate.ids {
                    bytes = bytes.saturating_add(id.len());
                }
            }
            for pass in passes {
                bytes = bytes.saturating_add(pass.name.len());
            }
            bytes
        }
    }
}
