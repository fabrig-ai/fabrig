use super::fixtures;
use crate::limits;
use skein_lib::{Duration, Token};

#[test]
fn an_override_sets_the_field_its_dotted_name_names() {
    let mut limits = limits::profile(limits::Profile::Small);
    assert_eq!((limits.forges.log_origins, limits.forge_component.log_origins), (20, 20));
    assert_eq!(limits.forge_component.log_poll, Duration::from_secs(5));
    assert_eq!(limits.forge_component.log_catch_up, Duration::from_secs(86_400));
    assert_eq!(limits.forge_component.log_pages, 10);
    assert_eq!(limits.forge_component.delivery_bytes, 1_048_576);
    assert_eq!(limits.forge.change_policy.questions, 4);
    for profile in [limits::Profile::Small, limits::Profile::Standard] {
        assert_eq!(limits::profile(profile).forge.change_policy.reruns, 1);
    }
    let rows: [(&[u8], u64); 17] = [
        (b"forge.repositories", 3),
        (b"forge.sitting_ms", 3000),
        (b"forge.change_policy.questions", 7),
        (b"forge.change_policy.reruns", 2),
        (b"forge.client.calls", 5),
        (b"forge.client.settle_ms", 2_000),
        (b"checkout.remote_timeout_ms", 7_000),
        (b"root.loads.rows", 9),
        (b"root.loads.read_bytes", 131_072),
        (b"clients.bundle_files", 2),
        (b"clients.bundle_bytes", 61),
        (b"forges.log_origins", 3),
        (b"forge_component.log_origins", 7),
        (b"forge_component.log_poll_ms", 7123),
        (b"forge_component.log_catch_up_ms", 9321),
        (b"forge_component.log_pages", 13),
        (b"forge_component.delivery_bytes", 12_345),
    ];
    for (name, value) in rows {
        assert_eq!(limits::apply(&mut limits, name, value), Ok(()), "{name:?}");
    }
    assert_eq!(limits.forge.repositories, 3);
    assert_eq!(limits.forge.sitting, Duration::from_secs(3));
    assert_eq!(limits.forge.change_policy.questions, 7);
    assert_eq!(limits.forge.change_policy.reruns, 2);
    assert_eq!((limits::find(b"forge.change_policy.reruns").expect("listed re-run bound").value)(&limits), 2);
    assert_eq!(limits.forge.client.calls, 5);
    assert_eq!(limits.forge.client.settle, Duration::from_secs(2));
    assert_eq!(limits.checkout.remote_timeout, Duration::from_millis(7_000));
    assert_eq!(limits.root.loads.rows, 9);
    assert_eq!(limits.root.loads.read_bytes, 131_072);
    assert_eq!(limits.clients, limits::Clients { bundle_files: 2, bundle_bytes: 61 });
    assert_eq!((limits.forges.log_origins, limits.forge_component.log_origins), (3, 7));
    assert_eq!(limits.forge_component.log_poll, Duration::from_nanos(7_123_000_000));
    assert_eq!(limits.forge_component.log_catch_up, Duration::from_nanos(9_321_000_000));
    assert_eq!(limits.forge_component.log_pages, 13);
    assert_eq!(limits.forge_component.delivery_bytes, 12_345);
}

#[test]
fn the_change_tasks_override_sets_its_field_and_refuses_values_outside_its_bound() {
    let mut limits = limits::profile(limits::Profile::Small);
    assert_eq!(limits.forge.change_policy.tasks, 6);
    let name = b"forge.change_policy.tasks";
    for tasks in [8, u32::MAX] {
        assert_eq!(limits::apply(&mut limits, name, u64::from(tasks)), Ok(()));
        assert_eq!(limits.forge.change_policy.tasks, tasks);
    }
    let before = limits;
    for tasks in [0, u64::from(u32::MAX) + 1] {
        assert_eq!(limits::apply(&mut limits, name, tasks), Err(limits::Refused::Range));
        assert_eq!(limits, before);
    }
}

#[test]
fn a_zero_change_tasks_override_refuses_the_start_naming_the_limit() {
    let document =
        fixtures::with(&fixtures::document(), b"application.limits.overrides", br#"{"forge.change_policy.tasks":0}"#);
    let problems = crate::startup::named(&document, &jig_protocol_config::Directories::working()).unwrap_err();
    assert_eq!(
        crate::startup::lines(&problems).as_ref(),
        b"application.limits.overrides.forge.change_policy.tasks: outside its type\n",
    );
}

#[test]
#[expect(clippy::disallowed_macros, reason = "the test formats override documents and their expected diagnostics")]
fn forge_changes_past_the_tier_ceiling_refuses_both_startup_routes() {
    for changes in [1024_u32, 1025] {
        let member = format!(r#"{{"profile":"standard","overrides":{{"forge.changes":{changes}}}}}"#);
        let document = fixtures::with(&fixtures::document(), b"application.limits", member.as_bytes());
        let directories = jig_protocol_config::Directories::working();
        let starting = crate::startup::named(&document, &directories).unwrap_err();
        let reading =
            crate::startup::store_limits(&document, &directories, Some(&fixtures::startup_agent())).unwrap_err();
        for problems in [starting, reading] {
            assert_eq!(
                crate::startup::lines(&problems).as_ref(),
                format!("application.limits.overrides.forge.changes: {changes}, past its bound of 379, the most the store's tier rows hold (2 * forge.changes + 1 within 759)\n").as_bytes(),
            );
        }
    }
}

#[test]
fn standard_changes_at_the_tier_bound_name_the_store_limit_to_raise() {
    let document = fixtures::with(
        &fixtures::document(),
        b"application.limits",
        br#"{"profile":"standard","overrides":{"forge.changes":379}}"#,
    );
    let directories = jig_protocol_config::Directories::working();
    let starting = crate::startup::named(&document, &directories).unwrap_err();
    let reading = crate::startup::store_limits(&document, &directories, Some(&fixtures::startup_agent())).unwrap_err();
    for problems in [starting, reading] {
        assert_eq!(
            crate::startup::lines(&problems).as_ref(),
            b"engine.limits.store: the store's map value, 43648982 bytes, past its bound of 3419252 bytes\n",
        );
    }
}

#[test]
fn an_unknown_name_or_a_value_past_its_type_is_refused() {
    let mut limits = limits::profile(limits::Profile::Small);
    let before = limits;
    let rows: [(&[u8], u64, limits::Refused); 6] = [
        (b"forge.unknown", 1, limits::Refused::Unknown),
        (b"forge.sitting_ms", u64::MAX, limits::Refused::Range),
        (b"Forge.repositories", 1, limits::Refused::Unknown),
        (b"forge.repositories", u64::from(u32::MAX) + 1, limits::Refused::Range),
        (b"root.loads.rows", u64::MAX, limits::Refused::Range),
        (b"checkout.remote_timeout_ms", u64::MAX, limits::Refused::Range),
    ];
    for (name, value, refused) in rows {
        assert_eq!(limits::apply(&mut limits, name, value), Err(refused), "{name:?}");
        assert_eq!(limits, before, "{name:?}");
    }
    for (name, maximum) in [
        (b"forge.change_policy.questions".as_slice(), u64::from(u32::MAX)),
        (b"forges.log_origins", u64::from(u32::MAX)),
        (b"forge_component.log_origins", u64::from(u32::MAX)),
        (b"forge_component.log_poll_ms", u64::MAX / 1_000_000),
        (b"forge_component.log_catch_up_ms", u64::MAX / 1_000_000),
        (b"forge_component.log_pages", u64::from(u32::MAX)),
        (b"forge_component.delivery_bytes", u64::from(u32::MAX)),
    ] {
        assert_eq!(limits::apply(&mut limits, name, maximum), Ok(()));
        let before = limits;
        assert_eq!(limits::apply(&mut limits, name, maximum + 1), Err(limits::Refused::Range));
        assert_eq!(limits, before, "{name:?}");
    }
}

#[test]
fn stock_profiles_hold_a_few_hundred_conflicted_paths() {
    for profile in [limits::Profile::Small, limits::Profile::Standard] {
        let own = limits::profile(profile);
        assert_eq!(own.checkout.conflicts, 512, "{profile:?}");
        assert_eq!(own.git_component.stdout_bytes, 131_072, "{profile:?}");
    }
}

#[test]
fn the_small_profile_passes_each_domains_validity() {
    let settings = fixtures::engine();
    assert!(temper_engine_domain_forge::valid(&settings.limits.forge));
    for profile in [limits::Profile::Small, limits::Profile::Standard] {
        let mut bounds = limits::profile(profile).forge;
        assert_eq!(bounds.sitting, temper_engine_domain_forge::SITTING);
        bounds.sitting = Duration::ZERO;
        assert!(!temper_engine_domain_forge::valid(&bounds));
        bounds.sitting = Duration::from_millis(1);
        assert!(temper_engine_domain_forge::valid(&bounds));
    }
    assert!(temper_worker_domain_checkout::valid(&crate::worker_settings::checkout(settings.limits.checkout)));
    assert_eq!(
        temper_worker_protocol_git::check(
            &crate::worker_settings::git_component(settings.limits.git_component),
            &crate::worker_settings::checkout(settings.limits.checkout)
        ),
        Ok(())
    );
    let jig = jig_protocol_config::profile(jig_protocol_config::Profile::Small);
    let root = limits::engine(&jig, &settings.limits).expect("coherent root profile");
    assert!(temper_engine_domain::valid(&root));
    assert_eq!(root.public_watermark, root.watermark / 2);
    let mut jig = jig;
    jig.clients.public.reads = 8;
    assert_eq!(limits::engine_candidate(&jig, &settings.limits).public_reads, 8);
}

#[test]
fn the_standard_profile_holds_fabrig_scale_work() {
    let small = limits::profile(limits::Profile::Small);
    let standard = limits::profile(limits::Profile::Standard);
    let expected = limits::Limits {
        forge: temper_engine_domain_forge::Limits {
            follows: 8,
            follow_queue: 64,
            brief_bytes: 131_584,
            tasks: 128,
            holds: 128,
            changes: 128,
            named_calls: 128,
            landings: 32,
            client: skein_forge_client::Limits { resources: 128, op_bytes: 262_144, ..small.forge.client },
            change_policy: temper_engine_domain_forge_change::Limits {
                reruns: 1,
                questions: 80,
                tasks: 80,
                repairs: 80,
                resolutions: 80,
                updates: 80,
                ..small.forge.change_policy
            },
            issue_policy: temper_engine_domain_forge_issues::Limits { plan_items: 384, ..small.forge.issue_policy },
            ..small.forge
        },
        root: limits::Root {
            brief: temper_engine_domain::BriefLimits {
                read_bytes: 131_584,
                brief_bytes: 353_072,
                budgets: temper_engine_domain::BriefBudgets {
                    task: 131_584,
                    pull: 131_584,
                    reviews: 131_584,
                    ..small.root.brief.budgets
                },
                ..small.root.brief
            },
            journal: temper_engine_domain::JournalLimits {
                // The closer and session each add 32 bytes; two executor places add 16.
                transcript_bytes: 22_467_776,
                result_bytes: 353_072,
                ..small.root.journal
            },
            loads: temper_engine_domain::loads::Limits {
                transcript_bytes: 70_208,
                bytes: 70_456,
                reply_bytes: 70_456,
                read_bytes: 131_072,
                ..small.root.loads
            },
        },
        ..small
    };
    assert_eq!(standard, expected);
}

#[test]
fn the_standard_brief_and_its_carriers_share_their_bounds() {
    let own = limits::profile(limits::Profile::Standard);
    let jig = jig_protocol_config::profile(jig_protocol_config::Profile::Small);
    let root = limits::engine(&jig, &own).expect("coherent brief carriers");
    assert!(temper_engine_domain::valid(&root));
    assert_eq!(own.root.brief.read_bytes, own.root.brief.budgets.task);
    assert_eq!(own.forge.brief_bytes, own.root.brief.read_bytes);
    assert_eq!(own.root.journal.result_bytes, own.root.brief.brief_bytes);
    let mut short = own;
    short.forge.brief_bytes -= 1;
    assert!(limits::engine(&jig, &short).is_none());
    short = own;
    short.root.journal.result_bytes -= 1;
    assert!(limits::engine(&jig, &short).is_none());
}

#[test]
fn the_standard_profile_passes_each_domains_validity() {
    let own = limits::profile(limits::Profile::Standard);
    assert!(temper_engine_domain_forge::valid(&own.forge));
    assert!(temper_worker_domain_checkout::valid(&crate::worker_settings::checkout(own.checkout)));
    assert_eq!(
        temper_worker_protocol_git::check(
            &crate::worker_settings::git_component(own.git_component),
            &crate::worker_settings::checkout(own.checkout)
        ),
        Ok(())
    );
    for profile in [jig_protocol_config::Profile::Standard, jig_protocol_config::Profile::Small] {
        let jig = jig_protocol_config::profile(profile);
        let root = limits::engine(&jig, &own).expect("coherent root profile");
        assert!(temper_engine_domain::valid(&root));
    }
    let jig = jig_protocol_config::profile(jig_protocol_config::Profile::Standard);
    let root = limits::engine(&jig, &own).unwrap();
    crate::store::admit(&root, &root.forge, &jig.store, None).expect("coherent standard store profile");
    let mut short = own;
    short.forge.named_calls -= 1;
    assert!(limits::engine(&jig, &short).is_none());
    short = own;
    short.forge.client.op_bytes -= 1;
    assert!(limits::engine(&jig, &short).is_none());
    short = own;
    short.root.journal.transcript_bytes -= 1;
    assert!(limits::engine(&jig, &short).is_none());
}

#[test]
fn the_standard_profile_is_read_by_name() {
    use crate::startup::{Problems, What, application};
    use jig_protocol_config::{self as config, Path};
    let jig = fixtures::engine().jig;
    let directories = config::Directories::working();
    for (member, expected) in [
        (br#"{"profile":"standard","overrides":{}}"#.as_slice(), Some(limits::profile(limits::Profile::Standard))),
        (br#"{"profile":"large","overrides":{}}"#, None),
    ] {
        let input = fixtures::with(fixtures::APPLICATION, b"limits", member);
        let document =
            config::document(&input, &config::DocumentLimits { bytes: 1_048_576, tokens: 16_384, depth: 32 }).unwrap();
        let mut problems = Problems::empty();
        let members = application::parse(&document, 0, &jig, &directories, &mut problems);
        if let Some(expected) = expected {
            assert!(problems.is_empty());
            assert_eq!(members.unwrap().limits, expected);
        } else {
            assert!(members.is_none());
            assert_eq!(problems.problems.len(), 1);
            let problem = problems.problems.get(0).unwrap();
            assert_eq!(problem.path, Path::root().member(b"application").member(b"limits").member(b"profile"));
            assert_eq!(problem.what, What::Profile);
        }
    }
    let input = fixtures::with(
        fixtures::APPLICATION,
        b"limits",
        br#"{"profile":"standard","overrides":{"forge.change_policy.tasks":79}}"#,
    );
    let document =
        config::document(&input, &config::DocumentLimits { bytes: 1_048_576, tokens: 16_384, depth: 32 }).unwrap();
    let mut problems = Problems::empty();
    let members = application::parse(&document, 0, &jig, &directories, &mut problems).unwrap();
    assert!(problems.is_empty());
    let mut expected = limits::profile(limits::Profile::Standard);
    expected.forge.change_policy.tasks = 79;
    assert_eq!(members.limits, expected);
}

#[test]
fn the_wire_admits_a_moved_file_and_temper_values_stay_at_their_bound() {
    let settings = fixtures::engine();
    let moved = jig_protocol_accounts::moved::bound(&settings.jig.limits.driver.client.document);
    let value = settings.limits.records.value_bytes;
    assert!(moved > u64::from(value));
    let service = fixtures::started();
    let wire = service.engine.as_ref().unwrap().clients.env.limits.wire.keyed_value_some;
    assert_eq!(u64::from(wire), moved);

    // The effective ceiling remains capped by jig even for a larger moved file.
    let mut larger = settings.jig.limits;
    larger.driver.client.document.record_bytes = u32::MAX;
    assert_eq!(
        limits::client_value_bytes(&larger, &settings.limits),
        settings.jig.limits.clients.wire.keyed_value_some
    );
}

#[test]
fn a_brief_holds_a_whole_task_specification() {
    let settings = fixtures::engine();
    let brief = settings.limits.root.brief;
    let spec = settings.jig.limits.core.tasks.spec_bytes;
    // The contract and requester lines follow the words in the task section.
    assert!(brief.budgets.task >= spec + 512, "task section {} for specs of {spec}", brief.budgets.task);
    assert!(brief.read_bytes >= brief.budgets.task && brief.brief_bytes >= brief.budgets.task);
}

#[test]
fn the_root_limits_take_jigs_children_and_temper_own() {
    let mut settings = fixtures::engine();
    settings.jig.limits.core.authority.segments =
        settings.jig.limits.core.authority.segments.checked_add(1).expect("fixture segment limit can increase");
    settings.jig.limits.core.authority.roles =
        settings.jig.limits.core.authority.roles.checked_sub(1).expect("fixture role limit can decrease");
    settings.jig.limits.core.settings.proposals =
        settings.jig.limits.core.settings.proposals.checked_add(1).expect("fixture proposal limit can increase");
    let root = limits::engine(&settings.jig.limits, &settings.limits).expect("coherent root limits");
    let jig = settings.jig.limits.core;
    assert_eq!(root.journal, settings.limits.root.journal);
    assert_eq!(root.loads, settings.limits.root.loads);
    assert_eq!(root.brief, settings.limits.root.brief);
    assert_eq!(
        root.forge,
        temper_engine_domain_forge::Limits {
            authority_segments: jig.authority.segments,
            roles: jig.authority.roles,
            setting_proposals: jig.settings.proposals,
            setting_history: jig.settings.history,
            ..settings.limits.forge
        }
    );
    assert_eq!(root.tasks, jig.tasks);
    assert_eq!(root.authority, jig.authority);
    assert_eq!(root.people, jig.people);
    assert_eq!(root.fleet, jig.fleet);
    assert_eq!(root.accounts, jig.accounts);
    assert_eq!(root.notes, jig.notes);
    assert_eq!(root.call_records, jig.call_records);
    let mut refused = settings.limits;
    refused.root.journal.transcript_bytes = 1;
    assert_eq!(limits::engine(&settings.jig.limits, &refused), None);
}

#[test]
fn each_git_component_limit_has_its_dotted_override() {
    let names: [(&[u8], u64, [u32; 6]); 6] = [
        (b"git_component.operations", 7, [7, 131_072, 512, 4096, 16, 4080]),
        (b"git_component.stdout_bytes", 262_144, [3, 262_144, 512, 4096, 16, 4080]),
        (b"git_component.stderr_bytes", 1024, [3, 131_072, 1024, 4096, 16, 4080]),
        (b"git_component.read_bytes", 7, [3, 131_072, 512, 7, 16, 4080]),
        (b"git_component.sweep_entries", 7, [3, 131_072, 512, 4096, 7, 4080]),
        (b"git_component.sweep_bytes", 4096, [3, 131_072, 512, 4096, 16, 4096]),
    ];
    for (name, value, expected) in names {
        let mut limits = limits::profile(limits::Profile::Small);
        limits::apply(&mut limits, name, value).unwrap();
        let git = limits.git_component;
        assert_eq!(
            [git.operations, git.stdout_bytes, git.stderr_bytes, git.read_bytes, git.sweep_entries, git.sweep_bytes],
            expected,
            "{name:?}"
        );
        assert_eq!(
            temper_worker_protocol_git::check(
                &crate::worker_settings::git_component(git),
                &crate::worker_settings::checkout(limits.checkout)
            ),
            Ok(()),
            "{name:?}"
        );
    }
}

#[test]
fn the_small_profile_gives_settle_thirty_seconds() {
    assert_eq!(limits::profile(limits::Profile::Small).forge.client.settle, Duration::from_secs(30));
}

#[test]
fn the_hook_limits_have_dotted_overrides() {
    type Row = (&'static [u8], u64, u64, fn(&limits::Limits) -> u64);
    let mut own = limits::profile(limits::Profile::Small);
    let rows: [Row; 5] = [
        (b"forge_component.hooks", 3, 3, |l| u64::from(l.forge_component.hooks)),
        (b"forge_component.hook_head_bytes", 4096, 4096, |l| u64::from(l.forge_component.hook_head_bytes)),
        (b"forge_component.hook_body_bytes", 8192, 8192, |l| u64::from(l.forge_component.hook_body_bytes)),
        (b"forge_component.hook_secret_bytes", 64, 64, |l| u64::from(l.forge_component.hook_secret_bytes)),
        (b"forge_component.hook_deadline_ms", 7000, 7_000_000_000, |l| l.forge_component.hook_deadline.as_nanos()),
    ];
    for (name, input, expected, read) in rows {
        assert_eq!(limits::apply(&mut own, name, input), Ok(()), "{name:?}");
        assert_eq!(read(&own), expected, "{name:?}");
    }
}

#[test]
#[expect(clippy::disallowed_methods, reason = "ordinary tests inspect diagnostic text")]
fn records_and_installations_override_and_check_both_bounds() {
    let jig = fixtures::engine().jig;
    let defaults = limits::profile(limits::Profile::Small);
    assert_eq!(defaults.records.value_bytes, 4096);
    assert_eq!(defaults.forges.installations, 8);
    assert_eq!(defaults.forge_component.installations, 8);
    assert_eq!(defaults.forge_component.key_bytes, u32::try_from(skein_tls::RS256_KEY_BYTES).unwrap());
    assert_eq!(defaults.forge_component.renew_before, Duration::from_secs(600));
    for (name, value, other) in [
        (b"records.value_bytes".as_slice(), 0, b"forge_component.key_bytes".as_slice()),
        (
            b"records.value_bytes",
            u64::from(crate::startup::checks::secret_value_bound(&jig, &defaults).unwrap()) + 1,
            b"engine.limits.store.value",
        ),
        (b"records.value_bytes", u64::from(defaults.forge_component.key_bytes), b"forge_component.key_bytes"),
        (b"forges.installations", 512, b"records.value_bytes"),
        (b"forge_component.key_bytes", u64::from(u32::MAX), b"AppKey"),
        (b"forges.installations", u64::from(u32::MAX), b"Accepted"),
        (b"forges.installations", 0, b"forge_component.installations"),
        (b"forges.installations", 9, b"forge_component.installations"),
        (b"forge_component.installations", 7, b"forges.installations"),
    ] {
        let mut own = defaults;
        limits::apply(&mut own, name, value).unwrap();
        let mut problems = crate::startup::Problems::empty();
        crate::startup::checks::records(&jig, &own, &mut problems);
        let lines = crate::startup::lines(&problems);
        assert!(lines.windows(name.len()).any(|part| part == name), "{name:?}");
        assert!(lines.windows(other.len()).any(|part| part == other), "{other:?}");
    }
    let mut own = defaults;
    for (name, value) in [(b"forge_component.key_bytes".as_slice(), 2000), (b"forge_component.renew_before_ms", 60_000)]
    {
        limits::apply(&mut own, name, value).unwrap();
    }
    assert_eq!(own.forge_component.key_bytes, 2000);
    assert_eq!(own.forge_component.renew_before, Duration::from_secs(60));
    let mut problems = crate::startup::Problems::empty();
    crate::startup::checks::records(&jig, &own, &mut problems);
    assert!(problems.is_empty());
}
#[test]
fn encoded_setup_writes_fit_exactly_and_name_the_short_bound() {
    use crate::startup::{Problems, checks};
    use temper_protocol_records as records;
    let settings = fixtures::engine();
    let mut own = settings.limits;
    let key = skein_lib::bytes::zeroed(usize::try_from(own.forge_component.key_bytes).unwrap());
    assert_eq!(records::encode(records::Item::AppKey, &key).unwrap().len(), 1203); // DER bound plus the two-byte version, tag and four-byte length.
    let mut ids = skein_lib::List::with_capacity(150);
    for id in 1..=150 {
        ids.push(id).unwrap();
    }
    assert_eq!(
        records::encode(records::Item::Accepted, skein_forge_setup::item::accepted_item(ids.as_slice()).expose())
            .unwrap()
            .len(),
        1207
    );
    for (installations, value, expected) in [
        (8, 1203, ""),
        (
            8,
            1202,
            "application.limits.overrides.records.value_bytes: must agree with AppKey (forge_component.key_bytes)\n",
        ),
        (150, 1207, "application.limits.overrides.forge_component.installations: 150 bytes, past its bound of 64\n"),
        (
            150,
            1206,
            concat!(
                "application.limits.overrides.records.value_bytes: must agree with Accepted (forges.installations)\n",
                "application.limits.overrides.forge_component.installations: 150 bytes, past its bound of 64\n"
            ),
        ),
    ] {
        own.forges.installations = installations;
        own.forge_component.installations = installations;
        own.records.value_bytes = value;
        let mut problems = Problems::empty();
        checks::records(&settings.jig, &own, &mut problems);
        assert_eq!(crate::startup::lines(&problems).as_ref(), expected.as_bytes());
    }
}
#[test]
#[expect(clippy::disallowed_macros, clippy::disallowed_methods, reason = "tests format diagnostic text")]
fn log_limits_check_zero_capacity_and_timing_relations() {
    let jig = fixtures::engine().jig;
    let defaults = limits::profile(limits::Profile::Small);
    for name in [
        "forges.log_origins",
        "forge_component.log_origins",
        "forge_component.log_poll_ms",
        "forge_component.log_catch_up_ms",
        "forge_component.log_pages",
        "forge_component.delivery_bytes",
    ] {
        let mut own = defaults;
        limits::apply(&mut own, name.as_bytes(), 0).unwrap();
        let mut problems = crate::startup::Problems::empty();
        crate::startup::checks::records(&jig, &own, &mut problems);
        let mut expected = format!("application.limits.overrides.{name}: must agree with {name} (above 0)\n");
        if name == "forge_component.log_origins" {
            expected +=
                "application.limits.overrides.forges.log_origins: must agree with forge_component.log_origins\n";
        }
        if name == "forge_component.log_catch_up_ms" {
            expected += "application.limits.overrides.forge_component.log_catch_up_ms: must agree with forge_component.log_poll_ms\n";
        }
        assert_eq!(crate::startup::lines(&problems).as_ref(), expected.as_bytes());
    }
    let day = 86_400_000;
    for (origins, capacity, poll, catch_up, expected) in [
        (4, 4, 5000, day, ""),
        (4, 5, 5000, day, ""),
        (20, 20, 5000, day, ""),
        (32, 32, 5000, day, ""),
        (33, 33, 5000, day, "forges.log_origins: must agree with forges.log_origins (at most 32)\n"),
        (2, 2, 5000, day, ""),
        (3, 2, 5000, day, "forges.log_origins: must agree with forge_component.log_origins\n"),
        (4, 4, day, day, ""),
        (4, 4, day, day - 1, "forge_component.log_catch_up_ms: must agree with forge_component.log_poll_ms\n"),
        (
            4,
            4,
            5000,
            day + 1,
            "forge_component.log_catch_up_ms: must agree with forge_component.log_catch_up_ms (at most 86400000 ms / 24 h)\n",
        ),
    ] {
        let mut own = defaults;
        own.forges.log_origins = origins;
        own.forge_component.log_origins = capacity;
        own.forge_component.log_poll = Duration::from_millis(poll);
        own.forge_component.log_catch_up = Duration::from_millis(catch_up);
        let mut problems = crate::startup::Problems::empty();
        crate::startup::checks::records(&jig, &own, &mut problems);
        let expected = [if expected.is_empty() { "" } else { "application.limits.overrides." }, expected].concat();
        assert_eq!(crate::startup::lines(&problems).as_ref(), expected.as_bytes());
    }
}
#[test]
fn sizes_follow_forge_forges() {
    let mut limits = fixtures::engine().limits;
    for forges in [1, 3, 12] {
        limits.forge.forges = forges;
        let inventory = crate::startup::checks::record_writes(&limits);
        assert_eq!(inventory.len(), forges + 5);
        for index in 5..inventory.len() {
            assert_eq!(*inventory.get(index).unwrap(), (Some(267), b"Origin".as_slice(), None));
        }
    }
}
#[test]
#[expect(clippy::disallowed_methods, reason = "ordinary tests inspect diagnostic text")]
fn github_component_checks_key_deadline_and_renewal() {
    for (name, value, expected) in [
        (b"forge_component.key_bytes".as_slice(), 1195, "forge_component.key_bytes"),
        (b"forge_component.deadline_ms", 271_000, "forge_component.deadline_ms"),
        (b"forge_component.renew_before_ms", 19_999, "forge_component.renew_before_ms"),
        (b"forge_component.renew_before_ms", 1_800_001, "forge_component.renew_before_ms"),
        (b"forge_component.installations", 0, "forge_component"),
    ] {
        let mut config = fixtures::configured_engine();
        config.forges[0].kind = crate::settings::Kind::GitHub;
        limits::apply(&mut config.limits, name, value).unwrap();
        if name == b"forge_component.deadline_ms" {
            config.limits.forge.client.lifetime = Duration::from_secs(300);
        }
        let Err(found) = fixtures::checked_endpoints(&config, fixtures::forge_runtime()) else { panic!("{expected}") };
        let found = crate::startup::Problems::jig(found, &jig_protocol_config::Path::root());
        let lines = crate::startup::lines(&found);
        assert!(core::str::from_utf8(&lines).unwrap().contains(expected), "{expected}: {lines:?}");
    }
}

#[test]
fn installation_capacity_fits_the_client_row_at_64_and_refuses_65() {
    // client/forge/records.md: the row carries at most 64 installations.
    const ROW_INSTALLATIONS: u32 = 64;
    let jig = fixtures::engine().jig;
    for (installations, accepted) in [(ROW_INSTALLATIONS, true), (ROW_INSTALLATIONS + 1, false)] {
        let mut own = limits::profile(limits::Profile::Small);
        own.forges.installations = installations;
        own.forge_component.installations = installations;
        let mut problems = crate::startup::Problems::empty();
        crate::startup::checks::records(&jig, &own, &mut problems);
        assert_eq!(problems.is_empty(), accepted);
    }
}

#[test]
fn picker_page_has_its_own_override_and_startup_bounds() {
    let mut settings = fixtures::engine();
    assert_eq!(settings.limits.picker.page, 30);
    for page in [0_u64, 1, 30, 100, 101] {
        let mut own = settings.limits;
        limits::apply(&mut own, b"picker.page", page).unwrap();
        assert_eq!(u64::from(own.picker.page), page);
        let mut found = crate::startup::Problems::empty();
        crate::startup::checks::records(&settings.jig, &own, &mut found);
        let lines = crate::startup::lines(&found);
        assert_eq!(skein_lib::bytes::find(&lines, b"picker.page").is_some(), page == 0 || page > 100);
    }
    assert_eq!(limits::apply(&mut settings.limits, b"picker.page", u64::MAX), Err(limits::Refused::Range));
}

#[test]
fn effective_core_limits_preserve_each_configured_policy_budget() {
    let own = limits::profile(limits::Profile::Small);
    for configured in [16_384, 512, 4096] {
        let mut jig = jig_protocol_config::profile(jig_protocol_config::Profile::Small);
        jig.core.policy_bytes = configured;
        let root = limits::engine(&jig, &own).expect("configured policy budget fits the root");
        assert_eq!(temper_engine_domain::core_limits(&root).policy_bytes, configured);
    }
}

#[test]
fn effective_core_limits_preserve_the_configured_call_answer_budget() {
    let own = limits::profile(limits::Profile::Small);
    let mut jig = jig_protocol_config::profile(jig_protocol_config::Profile::Small);
    // The service fixture's configured bound differs from both profile and journal.
    jig.core.call_answer_bytes = fixtures::engine().jig.limits.core.call_answer_bytes;
    let root = limits::engine(&jig, &own).expect("configured answer budget fits the root");
    assert_eq!(root.call_answer_bytes, jig.core.call_answer_bytes);
    assert_eq!(temper_engine_domain::core_limits(&root).call_answer_bytes, jig.core.call_answer_bytes);
}

#[test]
fn the_worker_takes_its_agents_host_reply_bound() {
    let input = fixtures::owned_worker_alone_document();
    let part = crate::startup::worker_part(&input, &jig_protocol_config::Directories::working()).unwrap();
    let agent = fixtures::owned_worker_agent();
    assert_eq!(agent.limits.bodies.host_reply_text, 4_098);
    assert_eq!(crate::worker_settings::root_limits(&part.worker, &agent, &part.limits).reply_bytes, 4_098);
}

#[test]
fn secrets_schema_fits_the_small_page_ceiling() {
    use temper_protocol_records::v1;
    // The row's u16 version, variant and u32 length precede the largest DER envelope.
    const APPLICATION_BYTES: u64 = 65_546;
    // Secrets keys are two owner bytes followed by a u64 number.
    const KEY_BYTES: u64 = 10;
    let (_, _, profile) = fixtures::store_limits();
    assert_eq!(v1::Stored::encoded_bound(&v1::CEILINGS), Some(APPLICATION_BYTES));
    assert_eq!(profile.store.page.bytes, 3_421_331);
    assert!(APPLICATION_BYTES + KEY_BYTES < u64::from(profile.store.page.bytes));
}

#[test]
fn token_rows_include_schema_framing_in_startup_admission() {
    use crate::startup::{Problems, checks};
    let settings = fixtures::engine();
    let mut own = settings.limits;
    // The worker wire's token ceiling plus seven bytes of schema framing.
    own.forge_component.token_bytes = 4096;
    for (value, expected) in [
        (4103, ""),
        (
            4102,
            "application.limits.overrides.records.value_bytes: must agree with tokens (forge_component.token_bytes and hook_secret_bytes)\n",
        ),
    ] {
        own.records.value_bytes = value;
        let mut problems = Problems::empty();
        checks::records(&settings.jig, &own, &mut problems);
        assert_eq!(crate::startup::lines(&problems).as_ref(), expected.as_bytes());
    }
}

#[test]
fn a_repair_holds_its_task_pull_and_whole_gate_remarks() {
    let limits = limits::profile(limits::Profile::Small);
    let brief = limits.root.brief;
    assert_eq!(brief.budgets.reviews, 4608);
    assert_eq!(brief.brief_bytes, 14_592);
    assert_eq!(limits.root.journal.result_bytes, brief.brief_bytes);
    assert_eq!(brief.budgets.task + brief.budgets.pull + brief.budgets.reviews + 768, brief.brief_bytes);
    assert!(brief.read_bytes >= brief.budgets.reviews);
    assert!(limits.forge.brief_bytes >= brief.read_bytes);
}
#[test]
fn server_connections_carry_the_box_streams_and_four_calls() {
    let mut jig = jig_protocol_config::profile(jig_protocol_config::Profile::Small);
    jig.clients.streams = 8;
    jig.clients.exchanges = 12;
    jig.clients_http.streams = 8;
    jig.clients_http.calls = 12;
    assert_eq!(limits::server(&jig).connections, 12);
}

#[test]
fn the_watch_rooms_follow_the_streams() {
    let settings = fixtures::engine();
    let mut jig = settings.jig.limits;
    for (streams, room) in [(2, 44), (8, 176)] {
        jig.clients.streams = streams;
        let root = limits::engine(&jig, &settings.limits).expect("admitted streams");
        assert_eq!(root.streams, streams);
        let views = temper_engine_domain::core_limits(&root).views;
        assert_eq!(views.watchers, room);
        assert_eq!(views.watchers_per_party, room);
    }
}

#[test]
fn a_room_past_what_the_views_hold_is_refused() {
    // Views reserve two slots per watch; this stream count overflows their slot count.
    let document = engine_overrides(br#""clients.streams":126322568"#);
    let problems = crate::startup::named(&document, &jig_protocol_config::Directories::working()).unwrap_err();
    assert_eq!(crate::startup::lines(&problems).as_ref(),
        b"engine.limits.overrides.clients.streams: must agree with representable watch rooms (clients.streams times 22, with two slots per watch)\n");
    let mut settings = fixtures::engine();
    settings.jig.limits.clients.streams = u32::MAX;
    assert!(limits::engine(&settings.jig.limits, &settings.limits).is_none());
}

#[test]
fn small_holds_two_document_sections_with_their_own_budget_and_override() {
    let mut limits = limits::profile(limits::Profile::Small);
    assert_eq!(limits.root.brief.sections, 10);
    assert_eq!(limits.root.brief.budgets.document, 4096);
    assert_eq!(limits.forge.brief_sections, limits.root.brief.briefs * limits.root.brief.sections * 2);
    assert_eq!(limits::apply(&mut limits, b"root.brief.budgets.document", 2048), Ok(()));
    assert_eq!(limits.root.brief.budgets.document, 2048);
}

#[test]
fn every_row_is_a_name_the_parser_takes() {
    let small = limits::profile(limits::Profile::Small);
    for (index, row) in limits::NAMES.iter().enumerate() {
        let mut changed = small;
        let value = (row.value)(&small);
        let other = if row.name == "setup.state_life_ms" { value - 1 } else { value + 1 };
        assert_eq!(limits::find(row.name.as_bytes()).unwrap().name, row.name);
        assert_eq!(limits::apply(&mut changed, row.name.as_bytes(), other), Ok(()), "{}", row.name);
        assert_eq!((row.value)(&changed), other, "{}", row.name);
        assert_eq!(limits::apply(&mut changed, row.name.as_bytes(), value), Ok(()));
        assert_eq!(changed, small);
        assert!(!row.line.is_empty() && row.line.len() <= 120, "{}", row.name);
        assert!(row.line.ends_with('.') && !row.line.contains(['\n', '\r']));
        assert_eq!(row.line.matches('.').count(), 1);
        for other in &limits::NAMES[index + 1..] {
            assert_ne!(row.name, other.name);
        }
    }
}

#[test]
fn an_unknown_name_is_refused() {
    let small = limits::profile(limits::Profile::Small);
    for name in [b"".as_slice(), b"root.absent", b"setup.absent", b"sign_in.members.extra", b"box.restarts"] {
        let mut changed = small;
        assert!(limits::find(name).is_none());
        assert_eq!(limits::apply(&mut changed, name, 1), Err(limits::Refused::Unknown));
        assert_eq!(changed, small);
    }
}

/// Configured public indexes reach the effective core (jig/docs/design/domain/root.md, 10).
#[test]
fn configured_public_index_bounds_reach_the_core() {
    let mut config = fixtures::engine();
    let default = limits::engine(&config.jig.limits, &config.limits).expect("default engine");
    assert_eq!(temper_engine_domain::core_limits(&default).public, jig_domain_core::PublicIndexLimits::OFF);
    config.jig.limits.core.public = jig_domain_core::PublicIndexLimits { goals: 4, ended: 3, lines: 8, depth: 4 };
    let configured = limits::engine(&config.jig.limits, &config.limits).expect("configured engine");
    assert_eq!(temper_engine_domain::core_limits(&configured).public, config.jig.limits.core.public);
}

// jig/docs/design/domain/engine/briefs.md, 2: recall loads a document's whole stored row.
#[test]
#[expect(clippy::disallowed_macros, reason = "ordinary test builds note text at each configured bound")]
fn the_standard_archive_load_holds_a_whole_note_row() {
    use jig_domain_notes as notes;
    use skein_lib::{List, Queue, Wall};
    use temper_engine_domain::{self as engine, loads};
    let notes = jig_protocol_config::profile(jig_protocol_config::Profile::Standard).core.notes;
    let mut segments = List::<Box<[u8]>>::with_capacity(notes.pattern_bytes);
    for _ in 0..notes.pattern_bytes {
        segments.push(Box::new([])).unwrap();
    }
    let mut references = List::with_capacity(notes.references);
    for task in 1..=notes.references {
        references.push(u64::from(task)).unwrap();
    }
    let row = engine::Record::Core(jig_domain_core::Record::Notes(notes::Record::Entry(notes::Entry {
        name: 1,
        scope: notes::Scope::Resources {
            project: 1,
            connector: 1,
            pattern: notes::Pattern {
                segments: segments.into_boxed(),
                last: notes::Last::Exact(vec![b'p'; notes.pattern_bytes.try_into().unwrap()].into()),
            },
        },
        description: vec![b't'; notes.description_bytes.try_into().unwrap()].into(),
        body: vec![b'x'; notes.body_bytes.try_into().unwrap()].into(),
        references,
        author: notes::Author::Party { party: 1 },
        revision: 1,
        changed: Wall::EPOCH,
    })));
    let page_bytes = engine::record_bytes(&row).unwrap() + u64::try_from(size_of::<engine::Record>()).unwrap();
    let mut domain = loads::Loads::new(&limits::profile(limits::Profile::Standard).root.loads);
    let mut out = Queue::uncapped();
    let owner = loads::begin(
        &mut domain,
        Token::new(1),
        engine::Range::Core(engine::CoreRange::Notes(notes::Range::Entry { name: 1 })),
        None,
        1,
        &mut out,
    )
    .unwrap();
    out.pop().expect("load request");
    loads::loaded(&mut domain, owner, Box::new([row]), None, &mut out);
    let result = out.pop().expect("load terminal");
    let loads::Request::Loaded { rows, cut, next, .. } = result else {
        panic!("whole note page needs {page_bytes} bytes: {result:?}");
    };
    assert_eq!(rows.len(), 1, "whole note page needs {page_bytes} bytes");
    assert!(cut.is_none());
    assert!(next.is_none());
}

#[test]
fn an_override_naming_a_watch_room_is_refused() {
    for (extra, expected) in [
        (br#""views.watchers":17"#.as_slice(), b"engine.limits.overrides.views.watchers: must agree with engine.limits.overrides.clients.streams, which sizes both watch rooms\n".as_slice()),
        (br#""views.watchers_per_party":17"#.as_slice(), b"engine.limits.overrides.views.watchers_per_party: must agree with engine.limits.overrides.clients.streams, which sizes both watch rooms\n".as_slice()),
    ] {
        let document = engine_overrides(extra);
        let problems = crate::startup::named(&document, &jig_protocol_config::Directories::working()).unwrap_err();
        assert_eq!(crate::startup::lines(&problems).as_ref(), expected, "{extra:?}");
    }
}

fn engine_overrides(extra: &[u8]) -> Box<[u8]> {
    let document = fixtures::document();
    let overrides = fixtures::value(&document, b"engine.limits.overrides");
    let prefix = overrides.strip_suffix(b"}").expect("fixture override object");
    let size = prefix.len().saturating_add(extra.len()).saturating_add(b",}".len());
    let mut overrides = skein_lib::Writer::new(size);
    for part in [prefix, b",", extra, b"}"] {
        overrides.put(part).expect("override map fits its measured length");
    }
    fixtures::with(&document, b"engine.limits.overrides", &overrides.finish())
}

#[test]
fn follow_limits_have_their_profile_values_and_dotted_overrides() {
    let small = limits::profile(limits::Profile::Small);
    let standard = limits::profile(limits::Profile::Standard);
    for (name, tiny, normal, unit) in [
        ("forge.follows", 2, 8, limits::Unit::Count),
        ("forge.follow_paths", 8, 8, limits::Unit::Count),
        ("forge.follow_queue", 16, 64, limits::Unit::Count),
        ("forge.follow_merge", 8, 8, limits::Unit::Count),
        ("forge.follow_commits", 8, 8, limits::Unit::Count),
        ("forge.follow_files", 32, 32, limits::Unit::Count),
        ("forge.follow_message_bytes", 512, 512, limits::Unit::Bytes),
        ("forge.follow_skips", 8, 8, limits::Unit::Count),
        ("forge.follow_reason_bytes", 256, 256, limits::Unit::Bytes),
    ] {
        let field = limits::find(name.as_bytes()).expect("follow limit named");
        assert_eq!(field.unit, unit);
        assert_eq!((field.value)(&small), tiny, "{name} in small");
        assert_eq!((field.value)(&standard), normal, "{name} in standard");
        let mut changed = small;
        limits::apply(&mut changed, name.as_bytes(), tiny + 1).unwrap();
        assert_eq!((field.value)(&changed), tiny + 1, "{name} override");
        assert_eq!(limits::apply(&mut changed, name.as_bytes(), u64::from(u32::MAX) + 1), Err(limits::Refused::Range));
    }
    assert_eq!(small.forge.follows, 2);
    assert_eq!(standard.forge.follows, 8);
}
