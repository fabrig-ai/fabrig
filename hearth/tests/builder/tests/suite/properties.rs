use hearth_builder_world::fixtures;
use hearth_test_render::properties as inputs;
#[test]
fn a_generated_repository_renders_the_worlds_digests_through_real_git() {
    for case in inputs::PROCESS_CASES {
        let generated = inputs::generated(case);
        let digests = fixtures::properties::digests(&generated, case);
        assert_eq!(fixtures::properties::printed(&generated), digests, "case {case}");
    }
}

#[test]
fn a_production_edit_leaves_the_lab_specs_through_real_git() {
    for case in inputs::PROCESS_CASES {
        let generated = inputs::generated(case);
        let before = inputs::environment(&fixtures::properties::printed(&generated), inputs::LAB);
        for edit in inputs::production_edits() {
            let edited = inputs::edited(&generated, &edit, case);
            assert_eq!(
                inputs::environment(&fixtures::properties::printed(&edited), inputs::LAB),
                before,
                "case {case}: {edit:?}"
            );
        }
    }
}

/// One test per batch, so that the batches run apart.
macro_rules! cut_pipe_batches {
    ($($name:ident = $batch:literal,)*) => {
        $(#[test]
        fn $name() {
            fixtures::properties::cut_pipe_digests($batch);
        })*
    };
}

cut_pipe_batches! {
    generated_repositories_have_the_render_worlds_digests_through_cut_pipes = 0,
    generated_repositories_have_the_render_worlds_digests_through_cut_pipes_1 = 1,
    generated_repositories_have_the_render_worlds_digests_through_cut_pipes_2 = 2,
    generated_repositories_have_the_render_worlds_digests_through_cut_pipes_3 = 3,
    generated_repositories_have_the_render_worlds_digests_through_cut_pipes_4 = 4,
    generated_repositories_have_the_render_worlds_digests_through_cut_pipes_5 = 5,
}

#[test]
fn generated_repositories_have_the_render_worlds_digests_through_the_simulated_client() {
    for case in fixtures::budgets::CASES {
        let generated = inputs::generated(case);
        let files = fixtures::files_of(&generated.leaves);
        let outcome = hearth_builder_world::simulated::run(
            &fixtures::properties::scenario(&files),
            case,
            skein_sim::Config::calm(),
        );
        let bytes = outcome.machine.contents(fixtures::OUTPUT).expect("client output");
        assert_eq!(
            fixtures::digests_of(std::str::from_utf8(&bytes).expect("JSON lines")),
            fixtures::properties::digests(&generated, case),
            "case {case}"
        );
    }
}
