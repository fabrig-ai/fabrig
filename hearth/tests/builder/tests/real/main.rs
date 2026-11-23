use hearth::Form;
use hearth_builder_world::fixtures::{self, Repository};
use skein_io::kernel::Exit;

#[test]
#[expect(clippy::disallowed_methods, reason = "hearth's round: reads PATH for the real git repository fixture")]
fn a_written_repository_is_the_one_git_makes() {
    use fixtures::File;
    let files = fixtures::thin_with(&[
        File { path: "executable", body: b"#!/bin/sh\nexit 0\n", mode: 0o755 },
        File::link("link", b"a.b"),
        File { path: "a.b", body: b"before directory\n", mode: 0o644 },
        File { path: "a/c", body: b"last in directory\n", mode: 0o644 },
    ]);
    let path = std::env::var_os("PATH").expect("git is required on PATH");
    let real = Repository::make_on(&path, "git-written", &files);
    let written = Repository::make("kit-written", &files);
    assert_eq!(written.git(&["rev-parse", "HEAD"]), real.git(&["rev-parse", "HEAD"]));
    drop(written.git(&["fsck", "--strict"]));
    let changed = [File { path: "a.b", body: b"changed\n", mode: 0o644 }];
    drop(real.next(&changed, &["a/c"]));
    drop(real.git(&["add", "-A"]));
    drop(real.git(&["commit", "--allow-empty", "--amend", "-q", "-m", "next"]));
    drop(written.next(&changed, &["a/c"]));
    assert_eq!(written.git(&["rev-parse", "HEAD"]), real.git(&["rev-parse", "HEAD"]));
    drop(written.git(&["fsck", "--strict"]));
}

#[test]
fn hearth_render_prints_the_thin_repositorys_spec_through_real_git() {
    let repository = Repository::make("thin", fixtures::THIN);
    let rendered = fixtures::render(&repository, &[], Form::Text);
    assert_eq!(
        String::from_utf8_lossy(&rendered.stdout),
        fixtures::THIN_TEXT,
        "hearth/docs/design/domain/render/README.md 6.2's lines"
    );
    assert_eq!(rendered.exit, Exit::Code(0), "hearth/docs/design/client/README.md 3.3: done");
}
#[test]
fn a_directory_that_is_no_repository_exits_75() {
    let repository = Repository::empty("no-repository");
    let rendered = fixtures::render(&repository, &[], Form::Text);
    assert_eq!(fixtures::lines_starting(&rendered.stdout, &["git "]), "git rev-parse exited 128\n");
    assert_eq!(rendered.exit, Exit::Code(75), "git failed operationally");
}
#[test]
fn a_scratch_repository_without_git_on_the_path_fails_by_name_not_by_skip() {
    let scratch = skein_scratch::Scratch::new("without-git");
    let error = Repository::try_make_on(scratch.path().as_os_str(), "no-git", fixtures::THIN)
        .expect_err("missing git refuses the scratch repository");
    let cause = error.strip_prefix("git is required on the supplied PATH: ").expect("the refusal names git");
    assert!(!cause.is_empty(), "the refusal includes the spawn error: {error}");
}

const CACHED: &[(&str, &[u8])] = &[
    ("775324fcea7f2cdcbf11df25550e362669caceb6e69c8370374d2d436bb63123", b"Welcome to x1.\n"),
    ("f487088ec8b18ff569dfede6f12e5fa91983f9f6dd2f3e0b6cee4349a231e745", b"#!/bin/sh\nexec cat /etc/motd\n"),
];
#[test]
fn static_files_enter_the_cache_by_sha256_and_no_temporary_stays() {
    use std::{fs, os::unix::fs::PermissionsExt};
    let repository = Repository::make("cache", fixtures::THIN);
    hearth::umask(0o077);
    let rendered = fixtures::render(&repository, &[], Form::Text);
    assert_eq!(rendered.exit, Exit::Code(0));
    let cache = repository.scratch().path().join("store/cache");
    assert_eq!(fs::metadata(&cache).expect("cache").permissions().mode() & 0o7777, 0o700);
    let mut names: Vec<_> =
        fs::read_dir(&cache).expect("cache listing").map(|entry| entry.expect("cache entry").file_name()).collect();
    names.sort();
    assert_eq!(
        names,
        [
            "42ecbc10b6791c87aa0ca833e522946a40a7eece2297a745c5c10cea07baea96",
            "42ecbc10b6791c87aa0ca833e522946a40a7eece2297a745c5c10cea07baea96.record",
            "775324fcea7f2cdcbf11df25550e362669caceb6e69c8370374d2d436bb63123",
            "775324fcea7f2cdcbf11df25550e362669caceb6e69c8370374d2d436bb63123.record",
            "f487088ec8b18ff569dfede6f12e5fa91983f9f6dd2f3e0b6cee4349a231e745",
            "f487088ec8b18ff569dfede6f12e5fa91983f9f6dd2f3e0b6cee4349a231e745.record"
        ]
    );
    for &(name, body) in CACHED {
        assert_eq!(fs::read(cache.join(name)).expect("cached bytes"), body, "{name}");
        assert_eq!(fs::metadata(cache.join(name)).expect("cache file").permissions().mode() & 0o7777, 0o644, "{name}");
    }
}

#[test]
fn a_broken_document_prints_its_error_after_the_specs_that_rendered() {
    let repository = Repository::make("broken-lab", &fixtures::thin_with(fixtures::BROKEN_LAB));
    assert_eq!(
        String::from_utf8_lossy(&fixtures::render(&repository, &[], Form::Text).stdout),
        fixtures::BROKEN_LAB_TEXT
    );
}
#[test]
fn a_broken_document_exits_with_configuration_failure() {
    let repository = Repository::make("broken-lab-exit", &fixtures::thin_with(fixtures::BROKEN_LAB));
    assert_eq!(fixtures::render(&repository, &[], Form::Text).exit, Exit::Code(2));
}
#[test]
fn a_broken_task_json_prints_its_error_with_its_task() {
    let repository = Repository::make("broken-task", &fixtures::broken_task());
    assert_eq!(
        String::from_utf8_lossy(&fixtures::render(&repository, &[], Form::Text).stdout),
        "tasks/motd/task.json:1:10: motd: duplicate key \"a\" (tasks/motd/task.json:1:2)\n"
    );
}
#[test]
fn a_broken_task_json_exits_with_configuration_failure() {
    let repository = Repository::make("broken-task-exit", &fixtures::broken_task());
    assert_eq!(fixtures::render(&repository, &[], Form::Text).exit, Exit::Code(2));
}
#[test]
fn a_store_inside_a_work_tree_diffs_as_outside_any_repository() {
    let rendered = fixtures::diff_beneath_work_tree("diff-config", false);
    assert_eq!(
        fixtures::lines_starting(&rendered.stdout, &["               @@"]),
        "               @@ -1,5 +1,5 @@\n               @@ -9,7 +9,7 @@ line 8\n"
    );
}
#[test]
fn a_work_trees_gitattributes_do_not_make_a_file_binary() {
    let rendered = fixtures::diff_beneath_work_tree("diff-attributes", true);
    assert_eq!(
        fixtures::lines_starting(&rendered.stdout, &["               @@"]),
        "               @@ -1,5 +1,5 @@\n               @@ -9,7 +9,7 @@ line 8\n"
    );
}
#[test]
fn json_counts_the_hunk_lines_left_out_past_200() {
    let repository = Repository::make("diff-left-out", fixtures::MOTD);
    let from = repository.next(&[fixtures::File { body: b"old\n".repeat(300).leak(), ..fixtures::MOTD_KIND }], &[]);
    let to = repository.next(&[fixtures::File { body: b"new\n".repeat(300).leak(), ..fixtures::MOTD_KIND }], &[]);
    let asked = fixtures::Asked { form: Form::JsonLines, targets: &[b"x1-prod"], ..fixtures::Asked::new(&from, &to) };
    let rendered = fixtures::diff(&repository, &asked);
    let stdout = std::str::from_utf8(&rendered.stdout).expect("JSON");
    assert_eq!(stdout.matches("\"kind\":\"removed\"").count(), 199);
    assert!(stdout.contains("\"hunks\":{\"binary\":false,\"left_out\":401,\"lines\":["));
}
#[test]
fn a_binary_file_answers_binary_with_no_hunk_line() {
    let repository = Repository::make("diff-binary", fixtures::MOTD);
    let from = repository.next(&[fixtures::File { body: b"old\0\n", ..fixtures::MOTD_KIND }], &[]);
    let to = repository.next(&[fixtures::File { body: b"new\0\n", ..fixtures::MOTD_KIND }], &[]);
    let asked = fixtures::Asked { form: Form::JsonLines, targets: &[b"x1-prod"], ..fixtures::Asked::new(&from, &to) };
    let rendered = fixtures::diff(&repository, &asked);
    let stdout = std::str::from_utf8(&rendered.stdout).expect("JSON");
    assert!(stdout.contains("\"hunks\":{\"binary\":true,\"left_out\":0,\"lines\":[]}"));
}

#[test]
fn a_task_json_out_of_shape_prints_its_fault_and_fails_its_targets() {
    let repository = Repository::make("miskeyed", &fixtures::thin_with(core::slice::from_ref(&fixtures::MISKEYED)));
    assert_eq!(
        String::from_utf8_lossy(&fixtures::render(&repository, &[], Form::Text).stdout),
        "tasks/motd/task.json:1:2: motd: no such key in a declaration\n"
    );
}
#[test]
fn a_task_json_out_of_shape_exits_with_configuration_failure() {
    let repository =
        Repository::make("miskeyed-exit", &fixtures::thin_with(core::slice::from_ref(&fixtures::MISKEYED)));
    assert_eq!(fixtures::render(&repository, &[], Form::Text).exit, Exit::Code(2));
}

#[test]
fn a_read_of_a_setting_its_task_lacks_prints_and_fails_its_targets() {
    let repository = Repository::make("misread", &fixtures::thin_with(fixtures::MISREAD));
    assert_eq!(
        String::from_utf8_lossy(&fixtures::render(&repository, &[], Form::Text).stdout),
        "tasks/motd/task.json:1:34: motd: reads core.domian, which core does not declare; core declares domain\n"
    );
}
#[test]
fn a_read_of_a_setting_its_task_lacks_exits_with_configuration_failure() {
    let repository = Repository::make("misread-exit", &fixtures::thin_with(fixtures::MISREAD));
    assert_eq!(fixtures::render(&repository, &[], Form::Text).exit, Exit::Code(2));
}

#[test]
fn hearth_render_prints_256_errors_then_the_count() {
    let repository = fixtures::broken_envs(300);
    let rendered = fixtures::render(&repository, &[], Form::Text);
    let stdout = fixtures::lines_starting(&rendered.stdout, &["envs/", "44 "]);
    let mut lines = stdout.lines();
    assert_eq!(lines.next(), Some("envs/e000.json:1:14: duplicate key \"motd\" (envs/e000.json:1:2)"));
    assert_eq!(lines.next_back(), Some("44 more errors"));
    assert_eq!(lines.next_back(), Some("envs/e255.json:1:14: duplicate key \"motd\" (envs/e255.json:1:2)"));
    assert_eq!(lines.count(), 254);
}
#[test]
fn hearth_render_with_300_broken_documents_exits_with_configuration_failure() {
    let repository = fixtures::broken_envs(300);
    assert_eq!(fixtures::render(&repository, &[], Form::Text).exit, Exit::Code(2));
}
#[test]
fn hearth_render_json_gives_left_out_44() {
    let repository = fixtures::broken_envs(300);
    let rendered = fixtures::render(&repository, &[], Form::JsonLines);
    assert_eq!(
        String::from_utf8_lossy(&rendered.stdout).lines().last(),
        Some("{\"record\":\"outcome\",\"outcome\":\"failed\",\"left_out\":44}")
    );
}

#[test]
fn a_default_without_its_type_prints_and_fails_its_targets() {
    let repository = Repository::make("mistyped", &fixtures::thin_with(&[fixtures::MISTYPED]));
    assert_eq!(
        String::from_utf8_lossy(&fixtures::render(&repository, &[], Form::Text).stdout),
        "tasks/motd/task.json:1:54: motd: not an int\n"
    );
}
#[test]
fn a_default_without_its_type_exits_with_configuration_failure() {
    let repository = Repository::make("mistyped-exit", &fixtures::thin_with(&[fixtures::MISTYPED]));
    assert_eq!(fixtures::render(&repository, &[], Form::Text).exit, Exit::Code(2));
}
#[test]
fn a_tree_git_cannot_read_prints_git_s_tail_and_exits_75() {
    let repository = Repository::make("broken-tree", fixtures::THIN);
    repository.remove_object(&repository.git(&["rev-parse", "HEAD^{tree}"]));
    let rendered = fixtures::render(&repository, &[], Form::Text);
    assert_eq!(String::from_utf8_lossy(&rendered.stdout), "git ls-tree exited 128\n  fatal: not a tree object\n");
    assert_eq!(rendered.exit, Exit::Code(75));
}
#[test]
fn a_stray_file_in_a_task_prints_a_repository_error() {
    let repository = Repository::make("stray", &fixtures::thin_with(core::slice::from_ref(&fixtures::STRAY)));
    let rendered = fixtures::render(&repository, &[], Form::Text);
    assert_eq!(
        String::from_utf8_lossy(&rendered.stdout),
        "tasks/motd/tre/etc/motd: motd: a path protocol/format/README.md 2.1 does not allow\n"
    );
    assert_eq!(rendered.exit, Exit::Code(2));
}

#[test]
fn a_template_that_does_not_compile_prints_its_decode_error_at_its_line() {
    let repository =
        Repository::make("broken-template", &fixtures::thin_with(core::slice::from_ref(&fixtures::BROKEN_TEMPLATE)));
    assert_eq!(
        String::from_utf8_lossy(&fixtures::render(&repository, &[], Form::Text).stdout),
        "tasks/motd/tree/etc/issue.j2:2:1: motd: a tag with no end\n"
    );
}
#[test]
fn a_template_that_does_not_compile_leaves_its_target_out() {
    let repository = Repository::make(
        "broken-template-exit",
        &fixtures::thin_with(core::slice::from_ref(&fixtures::BROKEN_TEMPLATE)),
    );
    assert_eq!(fixtures::render(&repository, &[], Form::Text).exit, Exit::Code(2));
}

#[test]
fn the_diffs_errors_follow_its_changes_and_precede_the_outcome() {
    let repository = Repository::make("diff-error-order", fixtures::MOTD);
    let from = repository.head();
    let to = repository.next(&[fixtures::MOTD_KIND, fixtures::BAD_WEST_LAB], &[]);
    let asked = fixtures::Asked { form: Form::JsonLines, ..fixtures::Asked::new(&from, &to) };
    let rendered = fixtures::diff(&repository, &asked);
    let stdout = std::str::from_utf8(&rendered.stdout).expect("JSON text");
    let mut lines = stdout.lines();
    assert_eq!(lines.next_back().and_then(|line| line.split(',').next()), Some("{\"record\":\"outcome\""));
    assert_eq!(
        lines.next_back(),
        Some(
            "{\"record\":\"error\",\"side\":\"to\",\"kind\":\"decode\",\"decode\":\"unexpected\",\"class\":\"configuration\",\"at\":{\"column\":2,\"line\":1,\"path\":\"hosts/west/west-lab.json\"}}"
        )
    );
    assert_eq!(lines.filter(|line| line.starts_with("{\"record\":\"change\"")).count(), 4);
}

#[test]
fn an_account_s_faults_print_at_their_lines_in_the_base() {
    let repository = Repository::make("users-schema", &fixtures::thin_with(fixtures::USERS));
    assert_eq!(
        String::from_utf8_lossy(&fixtures::render(&repository, &[], Form::Text).stdout),
        concat!(
            "hosts/x1/x1.json:2:16: x1-prod: users: not an int\n",
            "hosts/x1/x1.json:3:22: x1-prod: users: undeclared field \"shel\"; the object declares groups, password, shell, uid\n",
            "hosts/x1/x1.json:4:7: x1-prod: users: missing required field uid\n",
        )
    );
}
#[test]
fn an_account_s_faults_exit_with_configuration_failure() {
    let repository = Repository::make("users-schema-exit", &fixtures::thin_with(fixtures::USERS));
    assert_eq!(fixtures::render(&repository, &[], Form::Text).exit, Exit::Code(2));
}

#[test]
fn a_container_on_another_environment_s_target_prints_its_error() {
    let repository = Repository::make("containers", fixtures::CONTAINERS);
    let rendered = fixtures::render(&repository, &[], Form::Text);
    assert_eq!(
        fixtures::lines_starting(&rendered.stdout, &["target ", "hosts/"]),
        concat!(
            "target       east-lab   base east   environment lab\n",
            "target       east-prod   base east   environment prod\n",
            "hosts/chat/chat-prod.json:1:43: chat-prod: placement's on names east-lab, a target of lab\n",
        )
    );
}
#[test]
fn a_container_on_another_environment_s_target_exits_two() {
    let repository = Repository::make("containers-exit", fixtures::CONTAINERS);
    assert_eq!(fixtures::render(&repository, &[], Form::Text).exit, Exit::Code(2));
}

#[test]
fn hearth_render_lists_a_rendered_template_by_its_sha256() {
    let repository = Repository::make("templated", &fixtures::thin_with(fixtures::TEMPLATED));
    let rendered = fixtures::render(&repository, &[], Form::Text);
    assert_eq!(
        fixtures::lines_starting(&rendered.stdout, &["  /etc/hostname", "  /etc/issue"]),
        "  /etc/hostname            file 0644 root:root 21c50adeb849\n  /etc/issue               file 0644 root:root e75f92b4b841\n"
    );
}
#[test]
fn a_rendered_template_leaves_the_call_successful() {
    let repository = Repository::make("templated-exit", &fixtures::thin_with(fixtures::TEMPLATED));
    assert_eq!(fixtures::render(&repository, &[], Form::Text).exit, Exit::Code(0));
}
#[test]
fn a_missing_key_in_a_template_prints_its_evaluation_error() {
    let repository = Repository::make("template-key", &fixtures::thin_with(&[fixtures::MISSING_TEMPLATE_KEY]));
    assert_eq!(
        fixtures::render(&repository, &[], Form::Text).stdout,
        b"tasks/motd/tree/etc/issue.j2:1:11: x1-prod: motd: a missing key\n"
    );
}
#[test]
fn a_missing_key_in_a_template_fails_the_call() {
    let repository = Repository::make("template-key-exit", &fixtures::thin_with(&[fixtures::MISSING_TEMPLATE_KEY]));
    assert_eq!(fixtures::render(&repository, &[], Form::Text).exit, Exit::Code(2));
}

#[test]
fn a_missing_file_and_target_print_where_the_base_sets_them() {
    let repository = Repository::make("references-schema", &fixtures::thin_with(fixtures::REFERENCES));
    assert_eq!(
        String::from_utf8_lossy(&fixtures::render(&repository, &[], Form::Text).stdout),
        concat!(
            "hosts/x1/x1.json:1:28: x1-prod: net: no file lab-ca/lab.crt at this revision\n",
            "hosts/x1/x1.json:1:54: x1-prod: net: no target x9-prod in prod\n",
        )
    );
}
#[test]
fn missing_file_and_target_references_exit_with_configuration_failure() {
    let repository = Repository::make("references-schema-exit", &fixtures::thin_with(fixtures::REFERENCES));
    assert_eq!(fixtures::render(&repository, &[], Form::Text).exit, Exit::Code(2));
}

#[test]
fn rendered_files_enter_the_cache_by_sha256() {
    let repository = Repository::make("template-cache", &fixtures::thin_with(fixtures::TEMPLATED));
    drop(fixtures::render(&repository, &[], Form::Text));
    let cache = repository.scratch().path().join("store/cache");
    for (row, &(name, bytes)) in fixtures::RENDERED_FILES.iter().enumerate() {
        assert_eq!(std::fs::read(cache.join(name)).expect("cached rendering"), bytes, "row {row}");
    }
}
#[test]
fn rendered_cache_files_have_mode_0644() {
    use std::os::unix::fs::PermissionsExt;
    let repository = Repository::make("template-cache-modes", &fixtures::thin_with(fixtures::TEMPLATED));
    drop(fixtures::render(&repository, &[], Form::Text));
    let cache = repository.scratch().path().join("store/cache");
    for (row, &(name, _)) in fixtures::RENDERED_FILES.iter().enumerate() {
        assert_eq!(
            std::fs::metadata(cache.join(name)).expect("cache file").permissions().mode() & 0o7777,
            0o644,
            "row {row}"
        );
    }
}
#[test]
fn rendered_cache_files_leave_no_temporary() {
    let repository = Repository::make("template-cache-temporary", &fixtures::thin_with(fixtures::TEMPLATED));
    drop(fixtures::render(&repository, &[], Form::Text));
    let cache = repository.scratch().path().join("store/cache");
    assert_eq!(
        fixtures::names(&cache).into_iter().filter(|name| name.as_bytes().ends_with(b".tmp")).collect::<Vec<_>>(),
        Vec::<String>::new()
    );
}

#[test]
fn a_target_past_its_rendered_bytes_prints_a_limit_error() {
    let repository = Repository::make("template-target-limit", &fixtures::thin_with(fixtures::TEMPLATED));
    let rendered = fixtures::render_with(&repository, &fixtures::target_output_limit());
    assert_eq!(rendered.stdout, b"tasks/motd/tree/etc/issue.j2:1:39: x1-prod: motd: a limit exceeded\n");
}
#[test]
fn a_target_past_its_rendered_bytes_fails_the_call() {
    let repository = Repository::make("template-target-limit-exit", &fixtures::thin_with(fixtures::TEMPLATED));
    assert_eq!(fixtures::render_with(&repository, &fixtures::target_output_limit()).exit, Exit::Code(2));
}

#[test]
fn a_false_guard_prints_its_message_at_its_key_and_leaves_its_target_out() {
    let repository = Repository::make("guard", &fixtures::thin_with(core::slice::from_ref(&fixtures::GUARDED)));
    assert_eq!(
        fixtures::render(&repository, &[], Form::Text).stdout,
        b"tasks/motd/task.json:3:5: x1-prod: motd: requires greeting: set a greeting of your own\n"
    );
}
#[test]
fn a_false_guard_exits_with_configuration_failure() {
    let repository = Repository::make("guard-exit", &fixtures::thin_with(core::slice::from_ref(&fixtures::GUARDED)));
    assert_eq!(fixtures::render(&repository, &[], Form::Text).exit, Exit::Code(2));
}
#[test]
fn a_target_with_a_schema_fault_and_a_false_guard_prints_the_schema_fault_only() {
    let repository =
        Repository::make("guard-schema", &fixtures::thin_with(&[fixtures::GUARDED, fixtures::WRONG_GREETING]));
    assert_eq!(
        fixtures::render(&repository, &[], Form::Text).stdout,
        b"hosts/x1/x1.json:1:21: x1-prod: motd: not a string\n"
    );
}

#[test]
fn a_for_each_making_a_key_twice_prints_a_collision_at_its_entry() {
    let repository = Repository::make("twice", &fixtures::thin_with(core::slice::from_ref(&fixtures::TWICE)));
    assert_eq!(
        fixtures::render(&repository, &[], Form::Text).stdout,
        b"tasks/motd/task.json:3:5: x1-prod: motd: the key \"ops\" is made twice (tasks/motd/task.json:3:5)\n"
    );
}
#[test]
fn a_for_each_making_a_key_twice_exits_with_configuration_failure() {
    let repository = Repository::make("twice-exit", &fixtures::thin_with(core::slice::from_ref(&fixtures::TWICE)));
    assert_eq!(fixtures::render(&repository, &[], Form::Text).exit, Exit::Code(2));
}

#[test]
fn an_undefined_value_beside_text_fails_while_a_lone_tag_leaves_the_field_absent() {
    let repository = Repository::make("unset-field", &fixtures::thin_with(core::slice::from_ref(&fixtures::UNSET)));
    assert_eq!(
        fixtures::render(&repository, &[], Form::Text).stdout,
        b"tasks/motd/task.json:4:21: x1-prod: motd: an undefined value used\n"
    );
}
#[test]
fn an_undefined_value_beside_text_exits_with_configuration_failure() {
    let repository =
        Repository::make("unset-field-exit", &fixtures::thin_with(core::slice::from_ref(&fixtures::UNSET)));
    assert_eq!(fixtures::render(&repository, &[], Form::Text).exit, Exit::Code(2));
}
#[test]
fn a_boolean_in_a_string_field_prints_an_evaluation_error_at_the_field() {
    let repository =
        Repository::make("boolean-field", &fixtures::thin_with(core::slice::from_ref(&fixtures::BOOLEAN_FIELD)));
    assert_eq!(
        fixtures::render(&repository, &[], Form::Text).stdout,
        b"tasks/motd/task.json:3:21: x1-prod: motd: a value of the wrong type\n"
    );
}

#[test]
fn a_typo_in_a_settings_reference_prints_the_declared_names() {
    let repository = Repository::make("typo-reference", &fixtures::thin_with(core::slice::from_ref(&fixtures::TYPO)));
    assert_eq!(
        fixtures::render(&repository, &[], Form::Text).stdout,
        b"tasks/motd/task.json:3:21: motd: undeclared setting \"greting\"; motd declares greeting\n"
    );
}
#[test]
fn a_typo_in_a_settings_reference_exits_with_configuration_failure() {
    let repository =
        Repository::make("typo-reference-exit", &fixtures::thin_with(core::slice::from_ref(&fixtures::TYPO)));
    assert_eq!(fixtures::render(&repository, &[], Form::Text).exit, Exit::Code(2));
}
#[test]
fn a_typo_in_a_branch_no_run_takes_prints_at_its_line() {
    let repository = Repository::make("typo-branch", &fixtures::thin_with(fixtures::TYPO_BRANCH));
    assert_eq!(
        fixtures::render(&repository, &[], Form::Text).stdout,
        b"tasks/motd/tree/etc/issue.j2:1:27: motd: undeclared setting \"greting\"; motd declares greeting\n"
    );
}
#[test]
fn a_typo_in_an_untaken_branch_exits_with_configuration_failure() {
    let repository = Repository::make("typo-branch-exit", &fixtures::thin_with(fixtures::TYPO_BRANCH));
    assert_eq!(fixtures::render(&repository, &[], Form::Text).exit, Exit::Code(2));
}

#[test]
fn a_debian_the_schema_refuses_prints_the_layer_that_set_it() {
    let repository =
        Repository::make("debian-refused", &fixtures::thin_with(core::slice::from_ref(&fixtures::REFUSED_DEBIAN)));
    let rendered = fixtures::render(&repository, &[], Form::Text);
    assert_eq!(
        String::from_utf8_lossy(&rendered.stdout),
        "hosts/x1/x1.json:1:43: x1-prod: debian.components must keep main on\n"
    );
}
#[test]
fn a_refused_debian_exits_with_configuration_failure() {
    let repository =
        Repository::make("debian-refused-exit", &fixtures::thin_with(core::slice::from_ref(&fixtures::REFUSED_DEBIAN)));
    assert_eq!(fixtures::render(&repository, &[], Form::Text).exit, Exit::Code(2));
}

#[test]
fn a_secret_without_a_path_prints_its_fault_at_the_entry() {
    let repository =
        Repository::make("secret-no-path", &fixtures::thin_with(core::slice::from_ref(&fixtures::SECRET_NO_PATH)));
    assert_eq!(
        fixtures::render(&repository, &[], Form::Text).stdout,
        b"tasks/motd/task.json:1:14: x1-prod: motd: missing required field path\n"
    );
}
#[test]
fn a_secret_without_a_path_exits_with_configuration_failure() {
    let repository =
        Repository::make("secret-no-path-exit", &fixtures::thin_with(core::slice::from_ref(&fixtures::SECRET_NO_PATH)));
    assert_eq!(fixtures::render(&repository, &[], Form::Text).exit, Exit::Code(2));
}

mod files;
#[test]
fn a_component_debian_does_not_enable_prints_the_package_and_the_component() {
    let repository = fixtures::packages_repository("component-off", false);
    let rendered = fixtures::render(&repository, &[], Form::Text);
    assert_eq!(rendered.stdout, b"tasks/base/task.json:1:60: x1-prod: base: firmware-linux needs component non-free-firmware, which debian.components does not enable\n");
}
#[test]
fn a_component_debian_does_not_enable_exits_with_configuration_failure() {
    let repository = fixtures::packages_repository("component-off-exit", false);
    assert_eq!(fixtures::render(&repository, &[], Form::Text).exit, Exit::Code(2));
}

mod boot;

mod accounts;
mod exports;
mod package_changes;

mod diff;

mod additions;
mod data;
mod scripts;

mod diff_resolution;
mod software;

mod collisions;
mod ids;

mod part_changes;

mod reserved;
