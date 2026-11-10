//! What apt's pass answers from a mirror, over bytes already read (hearth/docs/design/testing/tool-fakes.md, 1).
use crate::{
    face::{self, Inst, Outcome, Simulate, Source},
    resolve::{self, Request, Selected, Variant},
};
use hearth_fake_machine::hearth::{
    mirror::{Mirror, Release},
    universe::Universe,
};

pub type Lists = Vec<(Box<[u8]>, Box<[u8]>)>;
/// The lists apt fetched and its outcome; no mirror within reach answers `Outcome::Unreachable`.
#[must_use]
pub fn pass(mirror: Option<&Mirror>, simulate: &Simulate, sources: &[Source]) -> (Lists, Outcome) {
    let Some(mirror) = mirror else {
        return (Vec::new(), Outcome::Unreachable);
    };
    let mut files = Vec::new();
    let mut universes = Vec::new();
    let mut releases = Vec::new();
    for (line, source) in sources.iter().enumerate() {
        let Some(suite) = mirror.suite(&source.archive, source.at, &source.suite) else {
            return (files, Outcome::NotFound { line: u32::try_from(line + 1).expect("source line fits") });
        };
        files.push((face::list_name(source, b"InRelease"), suite.in_release().into()));
        for component in &source.components {
            let universe = suite.universe(component).expect("source component served");
            universes.push(universe);
            releases.push(suite.release());
            let path = [component.as_ref(), b"/binary-", &simulate.architecture, b"/Packages"].concat();
            files.push((face::list_name(source, &path), suite.index(component).expect("source index").into()));
        }
    }
    let request = Request { universes: &universes, variant: simulate.variant, include: &simulate.include };
    let outcome = match resolve::closure(&request) {
        Ok(selected) => Outcome::Resolved(insts(&universes, &releases, &selected)),
        Err(mut failure) => {
            let essential =
                resolve::closure(&Request { universes: &universes, variant: Variant::Essential, include: &[] })
                    .expect("essential set resolves before the apt pass");
            let mut providers = Vec::new();
            if let resolve::Failure::NoCandidate(name) = &failure {
                for (source, universe) in universes.iter().enumerate() {
                    providers.extend(universe.providers(name).iter().map(|package| Selected {
                        universe: u32::try_from(source).expect("source fits"),
                        package: *package,
                    }));
                }
            }
            if let resolve::Failure::Unsatisfiable(broken) = &mut failure {
                for broken in broken {
                    let name = broken.relation.split(|b| *b == b' ').next().expect("relation name");
                    if !universes.iter().any(|universe| universe.package(name).is_some()) {
                        broken.against = None;
                    }
                }
            }
            Outcome::Failed {
                essential: insts(&universes, &releases, &essential),
                providers: insts(&universes, &releases, &providers),
                failure,
            }
        }
    };
    (files, outcome)
}
fn insts(universes: &[&Universe], releases: &[&Release], selected: &[Selected]) -> Box<[Inst]> {
    selected
        .iter()
        .map(|selected| {
            let package = &universes[usize::try_from(selected.universe).expect("source fits")].packages()
                [usize::try_from(selected.package).expect("package fits")];
            let mut origins = Vec::new();
            for (universe, release) in universes.iter().zip(releases) {
                if universe.package(&package.name).is_some_and(|other| other.version == package.version) {
                    if !origins.is_empty() {
                        origins.extend_from_slice(b", ");
                    }
                    origins.extend_from_slice(&release.origin);
                    origins.push(b':');
                    if let Some(version) = &release.version {
                        origins.extend_from_slice(version);
                        origins.push(b'/');
                    }
                    origins.extend_from_slice(&release.suite);
                }
            }
            Inst {
                name: package.name.clone(),
                version: package.version.clone(),
                architecture: package.architecture.clone(),
                origins: origins.into(),
            }
        })
        .collect()
}
