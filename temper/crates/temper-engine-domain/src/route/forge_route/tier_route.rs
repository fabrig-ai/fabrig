//! Deployment-started tier run tasks (temper/docs/design/domain/after-landing.md, 3, 10).
use super::{
    AFTER_LANDING_CODE, Box, Domain, EffectDescription, EffectForm, Env, Family, Limits, List, ProcedureStep, Recovery,
    StepEnd, Work, authority, effect_access, forge, resource_name, tasks,
};

pub(super) fn start_tier_run(
    domain: &mut Domain,
    env: &Env<Limits>,
    repository: skein_forge::api::Repository,
    tier: Box<[u8]>,
    number: u64,
    tip: skein_forge::api::Commit,
) {
    if !make_tier_run(domain, env, repository, &tier, number, tip) {
        domain.work.push(Work::Forge(forge::Event::TierRefused { repository, tier, number }));
    }
}

fn make_tier_run(
    domain: &mut Domain,
    env: &Env<Limits>,
    repository: skein_forge::api::Repository,
    tier: &[u8],
    number: u64,
    tip: skein_forge::api::Commit,
) -> bool {
    let Some(adopted) = domain.forge.repository(repository) else { return false };
    let project = adopted.project;
    let Some(words) = domain.forge.tier_specification(repository, tier, number, tip) else { return false };
    let Some(seed) = domain.core.procedure_seed(project) else { return false };
    let task = seed.number();
    domain.work.push(Work::Core(jig_domain_core::Event::ConnectorRepair {
        seed: Box::new(seed),
        executor: tasks::Executor::Procedure { connector: domain.config.forge_connector, code: AFTER_LANDING_CODE },
        spec: tasks::Spec {
            job: None,
            asks: false,
            words,
            parameters: Box::new([
                tasks::Parameter::Resource {
                    name: 1,
                    connector: domain.config.forge_connector,
                    resource: forge::tag(repository.forge, repository.repository),
                },
                tasks::Parameter::Bytes { name: 8, value: super::commit_text(&tip) },
                tasks::Parameter::Bytes { name: 10, value: Box::from(tier) },
                tasks::Parameter::Number { name: 11, value: number },
            ]),
            inputs: Box::new([]),
        },
        contract: tasks::Contract::Report { words: env.limits.tasks.result_bytes },
        holdings: Box::new([]),
    }));
    domain.work.push(Work::Forge(forge::Event::TierTask { repository, tier: Box::from(tier), number, task }));
    true
}

pub(crate) fn start_tier_step(domain: &mut Domain, env: &Env<Limits>, context: &tasks::RunContext) {
    let task = context.task;
    let released = match domain.core.tasks.task(task) {
        Some(row) => row.due_reason == tasks::Why::Released,
        None => false,
    };
    if !released && projection_needed(domain, task) {
        tier_decision(domain, env, task, forge::TierDecision::Wait { until: Some(env.wall) }, None);
        return;
    }
    let Some(entry) = crate::fresh(&mut domain.core.counters, Family::ConnectorRow) else { return };
    domain.forge.offered_tier(task);
    domain.work.push(Work::Forge(forge::Event::StepTier {
        task,
        entry,
        heard: forge::TierHeard { released, message_bytes: env.limits.tasks.message_bytes },
    }));
}

pub(super) fn tier_decision(
    domain: &mut Domain,
    env: &Env<Limits>,
    task: u64,
    choice: forge::TierDecision,
    projection: Option<Box<[u8]>>,
) {
    let Some((connector, code, step)) = domain.core.tasks.procedure_due(task) else { return };
    if connector != domain.config.forge_connector || code != AFTER_LANDING_CODE {
        return;
    }
    domain.forge.offered_tier(task);
    let mut decision = match choice {
        forge::TierDecision::Wait { until } => {
            domain.forge.wait_tier(&super::super::environment_forge(env), task, until);
            ProcedureStep::new(StepEnd::Wait)
        }
        forge::TierDecision::Held { words } => super::holds::decision(domain, env, task, words),
        forge::TierDecision::Done { words } => {
            let bound = if let tasks::Contract::Report { words } =
                &domain.core.tasks.task(task).expect("due tier task").contract
            {
                *words
            } else {
                env.limits.tasks.result_bytes
            };
            let words = report_words(&words, bound);
            ProcedureStep::new(StepEnd::Result(tasks::TaskResult::Report { words }))
        }
    };
    let projection = projection.or(domain.forge.tier_execution_words(task));
    if let Some(projection) = projection {
        decision.words = projected_words(domain, task, &projection);
    }
    domain.work.push(Work::Core(jig_domain_core::Event::ProcedureStep { task, step, connector, code, decision }));
}

pub(super) fn describe_tier_effect(
    domain: &Domain,
    env: &Env<Limits>,
    task: u64,
    entry: u64,
) -> Option<EffectDescription> {
    let (provider, what, described) = domain.forge.describe_tier_effect(task, entry)?;
    let repository = domain.forge.repository(provider)?;
    let context = domain.core.tasks.delegation(task)?;
    if repository.project != context.project {
        return None;
    }
    Some(EffectDescription {
        waits_for_dependencies: false,
        connector: domain.config.forge_connector,
        purpose: entry,
        effect: authority::Effect {
            connector: domain.config.forge_connector,
            kind: described.kind,
            name: resource_name(repository, &what, env.limits.authority.segments)?,
            state: described.state,
            price: None,
            access: effect_access(repository, &what, described.kind),
            additional: Box::new([]),
            guards: Box::new([]),
        },
        form: match described.form {
            forge::EffectForm::Creation => EffectForm::Creation,
            forge::EffectForm::Transition => EffectForm::Transition,
            forge::EffectForm::Set => EffectForm::Set,
        },
        recovery: match described.recovery {
            forge::Recovery::Keyed => Recovery::Keyed,
            forge::Recovery::Conditional => Recovery::Conditional,
            forge::Recovery::Idempotent => Recovery::Idempotent,
            forge::Recovery::Unrecoverable => Recovery::Unrecoverable,
        },
    })
}

fn projected_words(domain: &Domain, task: u64, projection: &[u8]) -> Option<Box<[u8]>> {
    let row = domain.core.tasks.task(task)?;
    let bound = u32::try_from(row.spec.words.len().checked_add(projection.len())?.checked_add(1)?).ok()?;
    let mut combined = List::with_capacity(bound);
    for byte in &row.spec.words {
        combined.push(*byte).expect("measured words");
    }
    if combined.as_slice().last() != Some(&b'\n') {
        combined.push(b'\n').expect("separator");
    }
    for byte in projection {
        combined.push(*byte).expect("measured projection");
    }
    let combined = combined.as_slice();
    let mut words = List::with_capacity(bound);
    let mut from = 0;
    for at in 0..combined.len() {
        if combined.get(at) != Some(&b'\n') {
            continue;
        }
        let line = combined.get(from..=at).expect("bounded projection line");
        from = at.checked_add(1).expect("bounded line cursor");
        if has_run(combined, line) {
            continue;
        }
        if skein_lib::bytes::find(words.as_slice(), line).is_none() {
            for byte in line {
                words.push(*byte).expect("measured projection");
            }
        }
    }
    Some(words.into_boxed())
}

fn has_run(words: &[u8], line: &[u8]) -> bool {
    let stem = line.get(..line.len().checked_sub(1).expect("nonempty line")).expect("line without newline");
    let suffix = b", forge run ";
    for earlier in words.windows(stem.len().checked_add(suffix.len()).expect("bounded line")) {
        if earlier.starts_with(stem) && earlier.ends_with(suffix) {
            return true;
        }
    }
    false
}

fn projection_needed(domain: &Domain, task: u64) -> bool {
    let Some(projection) = domain.forge.tier_execution_words(task) else { return false };
    let Some(words) = projected_words(domain, task, &projection) else { return false };
    match domain.core.tasks.task(task) {
        Some(row) => row.spec.words != words,
        None => false,
    }
}

// Each verdict line holds one whole escaped ID and its retained change read.
fn report_words(words: &[u8], bound: u32) -> Box<[u8]> {
    let room = skein_lib::usize_of(bound);
    if words.len() <= room {
        return Box::from(words);
    }
    let parsed = report_lines(words);
    let lines = parsed.as_slice();
    let mut kept = List::uncapped();
    let mut lost = List::uncapped();
    let summary = lines.first().expect("verdict summary");
    let mut size = summary.len();
    kept.push(*summary).expect("summary");
    for at in 1..lines.len() {
        let mut possible = lost.clone();
        let after = at.checked_add(1).expect("attribution cursor");
        for line in lines.get(after..).expect("remaining attributions") {
            possible.push(*line).expect("remaining tests");
        }
        let remainder = report_remainder(possible.as_slice());
        let line = lines.get(at).expect("attribution cursor");
        let length = line.len().checked_add(1).expect("attribution separator");
        let needed = size.checked_add(length).expect("bounded verdict");
        let tail = if remainder.is_empty() { 0 } else { remainder.len().checked_add(1).expect("remainder separator") };
        if needed.checked_add(tail).expect("bounded verdict") <= room {
            kept.push(*line).expect("whole test attribution");
            size = needed;
        } else {
            lost.push(*line).expect("omitted attribution");
        }
    }
    let remainder = report_remainder(lost.as_slice());
    if !remainder.is_empty() {
        kept.push(&remainder).expect("explicit remainder");
    }
    fit_report(kept.as_slice(), bound)
}

fn fit_report(lines: &[&[u8]], bound: u32) -> Box<[u8]> {
    let mut size = lines.len().saturating_sub(1);
    for line in lines {
        size = size.checked_add(line.len()).expect("bounded report");
    }
    let mut summary = *lines.first().expect("verdict summary");
    if size > skein_lib::usize_of(bound)
        && let Some(at) = skein_lib::bytes::find(summary, b": red,")
    {
        let end = at.checked_add(b": red".len()).expect("summary cursor");
        summary = summary.get(..end).expect("run identity");
    }
    let mut result = List::uncapped();
    for byte in summary {
        result.push(*byte).expect("summary words");
    }
    for line in lines.get(1..).expect("verdict attributions") {
        result.push(b'\n').expect("verdict separator");
        for byte in *line {
            result.push(*byte).expect("whole verdict line");
        }
    }
    result.into_boxed()
}

fn report_remainder(lines: &[&[u8]]) -> Box<[u8]> {
    if lines.is_empty() {
        return Box::new([]);
    }
    let mut many = false;
    let mut continuation: &[u8] = b"";
    for line in lines {
        let reference = report_reference(line);
        let end = line.len().saturating_sub(reference.len());
        many |= line.get(..end).expect("attribution without reference").ends_with(b": many tests");
        if continuation.is_empty() {
            continuation = reference;
        }
    }
    let count = skein_lib::Decimal::of(skein_lib::len_u64(lines.len()));
    let mut result = List::uncapped();
    let suffix: &[u8] = if !many && lines.len() == 1 { b" more failing test" } else { b" more failing tests" };
    for part in [b"and ".as_slice(), if many { b"many" } else { count.as_bytes() }, suffix, continuation] {
        for byte in part {
            result.push(*byte).expect("remainder words");
        }
    }
    result.into_boxed()
}

fn report_reference(line: &[u8]) -> &[u8] {
    let mut reference: &[u8] = b"";
    for at in 0..line.len() {
        if let Some(number) = line.get(at..).expect("reference cursor").strip_prefix(b"; change show ") {
            let mut decimal = !number.is_empty();
            for byte in number {
                decimal &= byte.is_ascii_digit();
            }
            if decimal {
                reference = line.get(at..).expect("whole reference");
            }
        }
    }
    reference
}

fn report_lines(words: &[u8]) -> List<&[u8]> {
    let mut lines = List::uncapped();
    let mut from = 0;
    for (at, byte) in words.iter().enumerate() {
        if *byte == b'\n' {
            lines.push(words.get(from..at).expect("whole attribution")).expect("verdict lines");
            from = at.checked_add(1).expect("bounded words");
        }
    }
    lines.push(words.get(from..).expect("last attribution")).expect("last verdict line");
    lines
}

#[cfg(test)]
mod report_tests {
    use super::*;

    #[test]
    fn bounded_reports_keep_whole_ids_and_an_explicit_remainder() {
        let words = full_length_words(
            b"Tier slow run 3: red, 2 tests failing, blamed on 1 landing\n012345 owns: \"no-break\"; change show 8\n012345 owns: \"",
            b"\"; change show 8",
        );
        let report = report_words(&words, 256);
        assert_eq!(report.as_ref(), b"Tier slow run 3: red, 2 tests failing, blamed on 1 landing\n012345 owns: \"no-break\"; change show 8\nand 1 more failing test; change show 8");
        assert!(report.len() <= 256);
        assert_eq!(report_words(&words, 512).as_ref(), words.as_ref());
    }

    #[test]
    fn a_many_attribution_is_never_reported_as_one_omitted_test() {
        let words = b"Tier slow run 3: red, many tests failing, blamed on 1 landing\n0123456789012345678901234567890123456789 owns: many tests; change show 8";
        assert_eq!(report_words(words, 120).as_ref(), b"Tier slow run 3: red, many tests failing, blamed on 1 landing\nand many more failing tests; change show 8");
    }
    #[test]
    fn a_shorter_later_id_fits_and_the_remainder_names_the_omitted_landing() {
        let words = full_length_words(
            b"Tier slow run 3: red, 2 tests failing, blamed on 2 landings\naaaa owns: \"",
            b"\"; change show 8\nbbbb owns: \"no-break\"; change show 9",
        );
        let report = report_words(&words, 256);
        assert!(skein_lib::bytes::find(&report, b"bbbb owns: \"no-break\"; change show 9").is_some());
        assert!(report.ends_with(b"and 1 more failing test; change show 8"));
        assert!(skein_lib::bytes::find(&report, &[b'x'; 128]).is_none());
        assert!(report.len() <= 256);
    }
    fn full_length_words(prefix: &[u8], suffix: &[u8]) -> Box<[u8]> {
        let length = prefix.len().checked_add(256).unwrap().checked_add(suffix.len()).unwrap();
        let mut words = skein_lib::Writer::new(length);
        words.put(prefix).unwrap();
        words.put(&[b'x'; 256]).unwrap();
        words.put(suffix).unwrap();
        words.finish()
    }
}
