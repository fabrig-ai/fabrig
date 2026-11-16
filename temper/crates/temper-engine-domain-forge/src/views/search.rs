//! Bounded search summaries (temper/docs/design/domain/landing-search-by-test.md, 8).
use super::{Domain, LandingSearch, LandingSearchTests, SearchPass, SearchSelection, policy};
use skein_lib::List;

impl Domain {
    pub(super) fn search_view(&self, row: &policy::Landing, search: &policy::Search) -> Option<LandingSearch> {
        let tests = match &search.tests {
            policy::SearchTests::Whole => LandingSearchTests::Whole,
            policy::SearchTests::WholeAgain => LandingSearchTests::WholeAgain,
            policy::SearchTests::ByTest { gates, probed } => {
                let member = row.members.first()?;
                let repository = skein_forge::api::Repository {
                    forge: member.repository.forge,
                    repository: member.repository.repository,
                };
                let configured = self.landing.gates(row.project, repository);
                let mut selections = List::with_capacity(self.landing.limits.gates);
                for (index, selection) in gates.iter().enumerate() {
                    let Some(selection) = selection else { continue };
                    let mut ids = List::with_capacity(3);
                    for id in selection.iter().take(3) {
                        ids.push(id.clone()).expect("three displayed ids");
                    }
                    selections
                        .push(SearchSelection {
                            gate: u16::try_from(index).ok()?,
                            name: configured.get(index)?.name.clone(),
                            count: u32::try_from(selection.len()).ok()?,
                            ids: ids.into_boxed(),
                        })
                        .expect("admitted gates");
                }
                let mut passes =
                    List::with_capacity(self.landing.limits.landing_members.checked_mul(self.landing.limits.gates)?);
                for pass in probed {
                    if !pass.passed {
                        continue;
                    }
                    let member = row.members.get(usize::from(pass.member.checked_sub(1)?))?;
                    passes
                        .push(SearchPass {
                            member: pass.member,
                            prefix: member.candidate?,
                            gate: pass.gate,
                            name: configured.get(usize::from(pass.gate))?.name.clone(),
                            run: pass.run,
                        })
                        .expect("admitted probes");
                }
                LandingSearchTests::ByTest { gates: selections.into_boxed(), passes: passes.into_boxed() }
            }
        };
        Some(LandingSearch { green: search.green, red: search.red, tests })
    }
}
