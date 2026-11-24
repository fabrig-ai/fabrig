//! The builder's render output through cut tool pipes (hearth/docs/design/testing/README.md, 2.5).
use hearth::{Call, Form};
use hearth_builder_world::{fixtures, protocol};

fn render(files: &[fixtures::File]) -> protocol::Run {
    protocol::run(&fixtures::Scenario { files, ..fixtures::thin() }, fixtures::SIMULATION_SEED)
}

fn render_at(files: &[fixtures::File], revision: &[u8], targets: &[&[u8]]) -> protocol::Run {
    let scenario = fixtures::Scenario {
        files,
        call: Call::Render {
            resolve: false,
            repository: fixtures::GIT_DIR.into(),
            revision: revision.into(),
            targets: targets.iter().map(|target| Box::<[u8]>::from(*target)).collect(),
        },
        ..fixtures::thin()
    };
    protocol::run(&scenario, fixtures::SIMULATION_SEED)
}

#[test]
fn json_lines_give_the_version_first_and_the_outcome_last() {
    let scenario = fixtures::Scenario { form: Form::JsonLines, ..fixtures::thin() };
    let rendered = protocol::run(&scenario, fixtures::SIMULATION_SEED);
    let stdout = std::str::from_utf8(&rendered.stdout).expect("JSON text");
    let lines: Vec<_> = stdout.lines().collect();
    assert_eq!(lines.first().copied(), Some("{\"record\":\"version\",\"client\":1}"), "version first");
    assert_eq!(
        lines.last().copied(),
        Some("{\"record\":\"outcome\",\"outcome\":\"done\",\"left_out\":0}"),
        "terminal last"
    );
    assert_eq!(
        lines.len(),
        fixtures::THIN_RECORDS,
        "version, spec, boot, packages, software, config, data, secrets, live, exports, meta and outcome"
    );
}

#[test]
fn an_unknown_target_exits_64_after_the_listing() {
    let rendered = render_at(fixtures::THIN, b"HEAD", &[b"no-such-target"]);
    assert_eq!(rendered.code, Some(64), "unknown target is Usage");
    assert_eq!(rendered.stdout, b"", "no unknown spec");
}

#[test]
fn a_revision_git_cannot_resolve_exits_64() {
    let rendered = render_at(fixtures::THIN, b"nope", &[]);
    assert_eq!(rendered.code, Some(64), "unresolved revision is Usage");
}

#[test]
fn hearth_render_gives_each_target_the_tasks_its_presence_layers_leave() {
    let rendered = render(&fixtures::thin_with(fixtures::LAYERS));
    assert_eq!(
        fixtures::lines_starting(&rendered.stdout, &["target ", "  /"]),
        "target       x1-prod   base x1   environment prod\n  /etc/motd                file 0644 root:root 775324fcea7f\n  /usr/local/bin/motd      file 0755 root:root f487088ec8b1\ntarget       x2-prod   base x2   environment prod\n"
    );
    assert_eq!(rendered.code, Some(0));
}

#[test]
fn a_cluster_s_members_render_as_targets_with_their_own_layers() {
    let rendered = render(&fixtures::cluster());
    assert_eq!(
        fixtures::lines_starting(&rendered.stdout, &["target ", "placement "]),
        concat!(
            "target       stoa-1-lab   base stoa   member 1   environment lab\n",
            "placement    vm efi incus\n",
            "target       stoa-2-lab   base stoa   member 2   environment lab\n",
            "placement    vm bios pool\n",
        )
    );
    assert_eq!(rendered.code, Some(0));
}

#[test]
fn a_template_prints_the_default_its_setting_was_filled_with() {
    let rendered = render(&fixtures::thin_with(fixtures::GREET));
    assert_eq!(
        fixtures::lines_starting(&rendered.stdout, &["  /etc/greeting"]),
        "  /etc/greeting            file 0644 root:root 5891b5b522d5\n"
    );
    assert_eq!(rendered.code, Some(0));
}

#[test]
fn a_guard_the_settings_satisfy_leaves_the_thin_spec_as_it_was() {
    let rendered = render(&fixtures::thin_with(&[fixtures::GUARDED, fixtures::GREETING]));
    assert_eq!(String::from_utf8_lossy(&rendered.stdout), fixtures::THIN_TEXT);
}

#[test]
fn distinct_for_each_keys_render_with_no_error() {
    let rendered = render(&fixtures::thin_with(&[fixtures::TWICE, fixtures::DISTINCT_TEAMS]));
    assert_eq!(String::from_utf8_lossy(&rendered.stdout), fixtures::DISTINCT_TEAMS_TEXT);
    assert_eq!(rendered.code, Some(0));
}

#[test]
fn a_hosts_template_lists_the_containers_placed_on_it() {
    let rendered =
        render_at(&fixtures::thin_with(fixtures::CONTAINED), b"HEAD", &[b"chat-prod", b"east-lab", b"east-prod"]);
    assert_eq!(
        fixtures::lines_starting(&rendered.stdout, &["  /etc/containers"]),
        concat!(
            "  /etc/containers          file 0644 root:root 01ba4719c80b\n",
            "  /etc/containers          file 0644 root:root 81d31332747e\n",
            "  /etc/containers          file 0644 root:root 2e41c4778818\n",
        ),
        "hearth/docs/design/protocol/format/targets.md, 1.5: chat none, lab mail, prod chat and mail"
    );
    assert_eq!(rendered.code, Some(0));
}

#[test]
fn a_template_reads_the_debian_root_composed_over_three_layers() {
    let rendered = render(&fixtures::thin_with(fixtures::DEBIAN_LAYERS));
    assert_eq!(
        fixtures::lines_starting(&rendered.stdout, &["  /etc/debian "]),
        "  /etc/debian              file 0644 root:root f745f2a642a2\n"
    );
}

#[test]
fn hearth_render_lists_secrets_passwords_and_identity_by_name_and_place() {
    let rendered = render(&fixtures::thin_with(core::slice::from_ref(&fixtures::SECRETS)));
    assert_eq!(
        fixtures::lines_starting(&rendered.stdout, &["  secret ", "  password ", "  identity "]),
        concat!(
            "  secret     mail/dkim/example.org.key /etc/dkimkeys/example.org.key 0640 root:opendkim\n",
            "  secret     mail/tls/mx1.key /etc/ssl/private/mx1.key 0600 root:root\n",
            "  password   alice users/alice.passwd\n",
            "  identity   ssh-ed25519 ssh-host-key ed25519 /etc/ssh/ssh_host_ed25519_key /etc/ssh/ssh_host_ed25519_key.pub 0600 root:root\n"
        )
    );
}

#[test]
fn a_target_prints_its_packages_part_from_debian_and_its_tasks() {
    let rendered = render(&fixtures::packages_files(true));
    assert_eq!(
        fixtures::lines_starting(
            &rendered.stdout,
            &["packages ", "  sources", "  request", "  variant", "  microcode", "  apt"]
        ),
        concat!(
            "packages     part bb94f2003485   resolution 059882763340 needed\n",
            "  sources    forky amd64 main non-free-firmware 20261001T030405Z\n",
            "  request    curl firmware-linux tzdata\n",
            "  variant    apt\n",
            "  microcode  generic\n",
            "  apt        install_recommends false\n"
        )
    );
}

#[test]
fn preseeds_and_modules_reach_the_packages_part() {
    let rendered = render(&fixtures::thin_with(fixtures::PRESEEDED));
    assert_eq!(
        fixtures::lines_starting(
            &rendered.stdout,
            &["packages ", "  sources", "  request", "  variant", "  preseed", "  initramfs", "  microcode", "  apt"]
        ),
        concat!(
            "packages     part 078c2a1460c5   resolution 3b244f24d10d needed\n",
            "  sources    forky amd64 main 20261001T030405Z\n",
            "  request    curl tzdata\n",
            "  variant    apt\n",
            "  preseed    tzdata/Areas select Europe\n",
            "  initramfs  btrfs nvme\n",
            "  microcode  generic\n",
            "  apt        install_recommends false\n"
        )
    );
}

#[test]
fn hearth_render_lists_each_path_with_its_consumers() {
    let rendered = render(&fixtures::thin_with(&[fixtures::LIVE]));
    assert_eq!(
        fixtures::lines_starting(&rendered.stdout, &["  path "]),
        concat!(
            "  path       /etc/mailname motd postfix.service reload\n",
            "  path       /etc/postfix/ motd postfix.service reload\n",
            "  path       /opt/ripgrep/ motd - none\n"
        )
    );
}
