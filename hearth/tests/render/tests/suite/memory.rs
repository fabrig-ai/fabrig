use hearth_domain_render::{
    evaluate::{self, Bindings, Budget, Evaluator},
    limits,
};
use hearth_render_world::fixtures;
use skein_heap::Meter;

#[test]
fn evaluating_at_the_limits_holds_no_more_than_its_ceiling() {
    const CEILING: u64 = 19_000_000;
    fixtures::memory::each(CEILING, fixtures::at_the_limits(), |expression, roots| {
        let mut evaluator = Evaluator::new(&limits::SHIPPED);
        let mut budget = Budget::new(&limits::SHIPPED);
        let (context, bindings) = (roots.context(), Bindings::with_capacity(fixtures::memory::EMPTY_BINDINGS));
        let evaluated = evaluate::expression(&mut evaluator, expression, &context, &bindings, &mut budget);
        drop(evaluated);
        drop(evaluator);
    });
}

#[test]
fn running_templates_at_the_limits_holds_no_more_than_its_ceiling() {
    const CEILING: u64 = 12_000_000;
    let limits = &fixtures::memory::TEMPLATE_LIMITS;
    fixtures::memory::each(CEILING, fixtures::templates_at_the_limits(), |template, roots| {
        let rendered =
            evaluate::template::run(template, &roots.context(), None, limits, u64::from(limits.rendered_file));
        drop(rendered);
    });
}

#[test]
fn expanding_at_the_limits_holds_no_more_than_its_ceiling() {
    const CEILING: u64 = 1_300_000;
    let limits = fixtures::EXPANSION_LIMITS;
    fixtures::memory::each(CEILING, fixtures::entries_at_the_limits(), |(declaration, count), roots| {
        let expanded =
            hearth_domain_render::entries::expand(declaration, &roots.context(), &limits, &mut { limits.expanded });
        match count {
            Some(count) => {
                let expanded = expanded.expect("the entry-count row fits");
                assert_eq!(expanded.groups.iter().map(|group| group.entries.len()).sum::<usize>(), *count);
            }
            None => assert_eq!(
                expanded.expect_err("expanded bytes reach their limit").kind,
                hearth_domain_render::errors::Kind::Evaluation(evaluate::FaultKind::Limit)
            ),
        }
    });
}

#[test]
fn collecting_errors_at_the_limits_holds_no_more_than_its_ceiling() {
    const CEILING: u64 = 22_000_000;
    use hearth_domain_render::errors::Collector;
    let meter = Meter::new();
    meter.start();
    let mut collector = Collector::new(&limits::SHIPPED);
    let step = meter.end();
    meter.check(step, CEILING, &"empty collector");
    for index in 0..limits::SHIPPED.errors {
        meter.start();
        collector.push(fixtures::error_at_budget(fixtures::ERROR_RECORD_BYTES));
        let step = meter.end();
        meter.check(step, CEILING, &index);
    }
    meter.start();
    collector.push(fixtures::error_at_budget(fixtures::ERROR_FIELDS_BYTES));
    let step = meter.end();
    meter.check(step, CEILING, &"first excluded error");
    let (errors, left_out) = collector.finish();
    assert_eq!(errors.len(), limits::SHIPPED.errors as usize);
    assert_eq!(left_out, 1);
}

#[test]
fn decoding_at_the_limits_holds_no_more_than_its_ceiling() {
    const CEILING: u64 = 20_000_000;
    let repository = fixtures::decoded_templates_repository();
    assert_eq!(
        repository.entries.iter().filter(|entry| entry.path.ends_with(b".j2")).count(),
        fixtures::DECODED_TEMPLATES as usize
    );
    assert_eq!(
        hearth_domain_render::revision::decoded_size(
            &hearth_domain_render::vocabulary::Decoded::Template(fixtures::newline_template()),
            &limits::SHIPPED,
        ),
        fixtures::DECODED_TEMPLATE_BYTES,
    );
    fixtures::checked_decoding(&repository, CEILING, &fixtures::newline_template());
}

#[test]
fn rendering_every_target_holds_one_targets_settings() {
    const CEILING: u64 = 390_000;
    let repository = fixtures::settings::every_target();
    let run = hearth_render_world::parent::checked(
        &repository,
        fixtures::settings::MEMORY_SEED,
        fixtures::settings::LIMITS,
        CEILING,
    );
    assert_eq!(run.outcome, hearth_domain_render::vocabulary::Outcome::Done);
    assert_eq!(run.answers.len(), fixtures::settings::LIMITS.targets as usize);
}

#[test]
fn decoding_comment_only_templates_holds_no_more_than_its_ceiling() {
    const CEILING: u64 = 21_000_000;
    let repository = fixtures::decoded_comments_repository();
    fixtures::checked_decoding(&repository, CEILING, &fixtures::comment_template());
}

#[test]
fn rendering_specs_at_the_limits_holds_no_more_than_its_ceiling() {
    const CEILING: u64 = 390_000;
    let repository = fixtures::specs::repository();
    let run = hearth_render_world::parent::checked(
        &repository,
        fixtures::specs::MEMORY_SEED,
        fixtures::specs::LIMITS,
        CEILING,
    );
    assert_eq!(run.outcome, hearth_domain_render::vocabulary::Outcome::Failed);
    assert!(
        run.answers.iter().any(|answer| matches!(answer, hearth_domain_render::vocabulary::Answer::Spec(_))),
        "the leading specs fit before the next target fails"
    );
}

#[test]
fn a_diff_keeping_the_first_sides_blobs_holds_no_more_than_its_ceiling() {
    const CEILING: u64 = 20_000_000;
    let repository = fixtures::reuse::kept_templates();
    let run = hearth_render_world::parent::checked_diff(
        &repository,
        &repository,
        fixtures::reuse::SEED,
        fixtures::reuse::LIMITS,
        CEILING,
    );
    assert_eq!(run.outcome, hearth_domain_render::vocabulary::Outcome::Done);
    assert_eq!(fixtures::reuse::second_reads(&run), []);
}

#[test]
fn templates_keep_expanded_entries_within_their_ceiling() {
    const CEILING: u64 = 22_000_000;
    let limits = &fixtures::memory::TEMPLATE_LIMITS;
    let (declaration, template) = fixtures::memory::expanded_template();
    let roots = fixtures::wide_roots();
    fixtures::memory::peak(&Meter::new(), CEILING, &"expanded entries beside template output", || {
        let expanded =
            hearth_domain_render::entries::expand(&declaration, &roots.context(), limits, &mut { limits.expanded })
                .expect("expanded fields fit");
        assert_eq!(
            expanded.groups.iter().map(|group| group.entries.len()).sum::<usize>(),
            fixtures::memory::EXPANDED_COPIES
        );
        let text_bytes: usize = expanded
            .groups
            .iter()
            .flat_map(|group| &group.entries)
            .flat_map(|entry| &entry.fields)
            .map(|field| {
                let hearth_domain_render::entries::Fielded::Text(text) = &field.value else {
                    panic!("expanded from field is text")
                };
                text.len()
            })
            .sum();
        assert_eq!(text_bytes, fixtures::memory::EXPANDED_TEXT_BYTES);
        let rendered =
            evaluate::template::run(&template, &roots.context(), None, limits, u64::from(limits.rendered_file))
                .expect("template output fits");
        drop(rendered);
        drop(expanded);
    });
}
