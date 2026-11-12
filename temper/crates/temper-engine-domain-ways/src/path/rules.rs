//! Coverage and gate order (temper/docs/design/domain/forge/change/gates.md, 1 and 3 and temper/docs/design/domain/forge/change/branch-path.md).
use super::{Inputs, scopes::Coverage};
use alloc::boxed::Box;
use skein_lib::List;
use temper_engine_domain_forge::{GateLast, settings::Held};

#[derive(Debug)]
pub(super) struct Gate {
    pub number: u32,
    pub check: bool,
    pub blocking: bool,
    pub coverage: Coverage,
}

// The forge's branch_resource_name and covers use the same segment comparison.
// With no repository, four placeholders admit branch patterns alone (8.13).
pub(super) fn covering(inputs: &Inputs, branch: &[u8]) -> List<Held> {
    let mut name: List<Box<[u8]>> = List::uncapped();
    let parts: [&[u8]; 4] = match &inputs.repository {
        Some(repository) => [b"forge", &repository.host, &repository.owner, &repository.name],
        None => [b"forge", b"-", b"-", b"-"],
    };
    for part in parts {
        if part.is_empty() {
            return List::uncapped();
        }
        name.push(part.into()).expect("repository segments");
    }
    name.push(b"branch".as_slice().into()).expect("branch marker");
    let mut start = 0;
    for at in 0..=branch.len() {
        if at == branch.len() || branch.get(at) == Some(&b'/') {
            let part = branch.get(start..at).expect("branch segment within input");
            if part.is_empty() {
                return List::uncapped();
            }
            name.push(part.into()).expect("branch segments");
            start = at.saturating_add(1);
        }
    }
    let mut result = List::uncapped();
    for held in &inputs.rules {
        let pattern = &held.rule.pattern;
        let is_branch = match pattern.segments.first() {
            Some(first) => first.as_ref() == b"branch",
            None => false,
        };
        if inputs.repository.is_none() && !is_branch {
            continue;
        }
        let name =
            if is_branch { name.as_slice().get(4..).expect("four repository segments") } else { name.as_slice() };
        if name.get(..pattern.segments.len()) != Some(pattern.segments.as_ref()) {
            continue;
        }
        let covered = match &pattern.last {
            GateLast::Exact(last) => {
                name.len().checked_sub(1) == Some(pattern.segments.len())
                    && name.get(pattern.segments.len()) == Some(last)
            }
            GateLast::Open(last) => match name.get(pattern.segments.len()) {
                Some(segment) => segment.starts_with(last),
                None => false,
            },
        };
        if covered {
            result.push(held.clone()).expect("covering held rules");
        }
    }
    result
}

pub(super) fn gates(inputs: &Inputs, held: &[Held]) -> List<Gate> {
    let mut gates: List<Gate> = List::uncapped();
    for held in held {
        for gate in &held.rule.gates {
            let mut present = false;
            for prior in gates.iter_mut() {
                if prior.number == gate.number {
                    prior.blocking |= held.rule.enforce && gate.blocking;
                    prior.coverage.add(inputs, &held.rule);
                    present = true;
                    break;
                }
            }
            if !present {
                let mut coverage = Coverage::new();
                coverage.add(inputs, &held.rule);
                gates
                    .push(Gate {
                        number: gate.number,
                        check: false,
                        blocking: held.rule.enforce && gate.blocking,
                        coverage,
                    })
                    .expect("held gates");
            }
        }
    }
    if let Some(repository) = &inputs.repository
        && !repository.ci
    {
        for &number in &repository.checks {
            let mut present = false;
            for gate in gates.iter_mut() {
                if gate.number == number {
                    gate.check = true;
                    gate.coverage.all = true;
                    gate.blocking = true;
                    present = true;
                    break;
                }
            }
            if !present {
                let mut coverage = Coverage::new();
                coverage.all = true;
                gates.push(Gate { number, check: true, blocking: true, coverage }).expect("repository check gates");
            }
        }
    }
    gates
}
