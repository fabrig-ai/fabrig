//! Deployment-started tier run tasks (temper/docs/design/domain/after-landing-runs.md, 2, temper/docs/design/domain/after-landing-storage.md, 1).
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
    let Some(seed) = domain.engine.core_mut().procedure_seed(project) else { return false };
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
    let released = match domain.engine.core().tasks.task(task) {
        Some(row) => row.due_reason == tasks::Why::Released,
        None => false,
    };
    if !released && projection_needed(domain, task) {
        tier_decision(domain, env, task, forge::TierDecision::Wait { until: Some(env.wall) }, None);
        return;
    }
    let Some(entry) = crate::fresh(&mut domain.engine.core_mut().counters, Family::ConnectorRow) else { return };
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
    let Some((connector, code, step)) = domain.engine.core().tasks.procedure_due(task) else { return };
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
                &domain.engine.core().tasks.task(task).expect("due tier task").contract
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
    let context = domain.engine.core().tasks.delegation(task)?;
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
    let row = domain.engine.core().tasks.task(task)?;
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
    match domain.engine.core().tasks.task(task) {
        Some(row) => row.spec.words != words,
        None => false,
    }
}

// Each verdict line holds one whole escaped ID and its retained change read.
fn report_words(words: &[u8], bound: u32) -> Box<[u8]> {
    let mut offset = 0_usize;
    while let Some(rest) = words.get(offset..) {
        let length = skein_lib::bytes::find(rest, b"\n").unwrap_or(rest.len());
        let line = rest.get(..length).expect("report line");
        if line.ends_with(b" red filings wait for room in the project's findings.")
            || line.ends_with(b" red filings were dropped for want of room.")
        {
            let end = offset.saturating_sub(1);
            let tail = words.get(end..).expect("report tail");
            let reserve = u32::try_from(tail.len()).expect("bounded filing counts");
            let head = forge::tier_report_words(words.get(..end).expect("report head"), bound.saturating_sub(reserve));
            let mut result = List::uncapped();
            for part in [head.as_ref(), tail] {
                for byte in part {
                    result.push(*byte).expect("report words");
                }
            }
            return result.into_boxed();
        }
        if length == rest.len() {
            break;
        }
        offset = offset.saturating_add(length).saturating_add(1);
    }
    forge::tier_report_words(words, bound)
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
    #[expect(clippy::disallowed_macros, reason = "literal report expectations compose test text")]
    fn many_names_take_only_the_room_after_attributions_and_remainder() {
        let summary = "Tier main run 3: red, many tests failing, blamed on 2 landings";
        let attribution = "012345 owns: many tests; change show 8";
        let omitted = "x".repeat(256);
        let words = format!(
            "{summary}\n{attribution}\n{omitted} suspected of: many tests; change show 9\nIts report names: \"suite::t00\", \"suite::t01\", \"suite::t02\" and 21 more."
        );
        let remainder = "and many more failing tests; change show 9";
        let expected = format!(
            "{summary}\n{attribution}\nIts report names: \"suite::t00\", \"suite::t01\" and 22 more.\n{remainder}"
        );
        let report = report_words(words.as_bytes(), u32::try_from(expected.len()).unwrap());
        assert_eq!(report.as_ref(), expected.as_bytes());
        let no_ids = format!("{summary}\n{attribution}\nIts report names: and 24 more.\n{remainder}");
        assert_eq!(report_words(words.as_bytes(), u32::try_from(no_ids.len()).unwrap()).as_ref(), no_ids.as_bytes());
        let dropped = format!("{summary}\n{attribution}\n{remainder}");
        assert_eq!(
            report_words(words.as_bytes(), u32::try_from(no_ids.len() - 1).unwrap()).as_ref(),
            dropped.as_bytes()
        );
    }

    #[test]
    #[expect(clippy::disallowed_macros, reason = "literal report expectations compose test text")]
    fn many_names_keep_escaped_quotes_whole_and_the_cut_remainder() {
        let prefix = "Tier main run 3: red, many tests failing, blamed on 1 landing\n012345 owns: many tests; change show 8\nIts report names: ";
        let words =
            format!("{prefix}\"a\\\"b\\\\c\\nd\", \"suite::t01\" and more, its report cut at the read's bound.");
        let expected = format!("{prefix}\"a\\\"b\\\\c\\nd\" and 1 more, its report cut at the read's bound.");
        assert_eq!(
            report_words(words.as_bytes(), u32::try_from(expected.len()).unwrap()).as_ref(),
            expected.as_bytes()
        );
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
    #[test]
    fn bounded_reports_reserve_waiting_and_unkept_counts_after_the_remainder() {
        let tail =
            b"\n8 red filings wait for room in the project's findings.\n12 red filings were dropped for want of room.";
        let words = full_length_words(b"Tier slow run 3: red, 2 tests failing, blamed on 2 landings\naaaa owns: \"", b"\"; change show 8\nbbbb owns: \"no-break\"; change show 9\n8 red filings wait for room in the project's findings.\n12 red filings were dropped for want of room.");
        let report = report_words(&words, 256);
        assert!(report.ends_with(tail), "{report:?}");
        assert!(skein_lib::bytes::find(&report, b"and 1 more failing test; change show 8").is_some(), "{report:?}");
        assert!(skein_lib::bytes::find(&report, b"bbbb owns: \"no-break\"; change show 9").is_some(), "{report:?}");
        assert!(report.len() <= 256, "{report:?}");
        assert_eq!(report_words(&words, 1024), words);
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
