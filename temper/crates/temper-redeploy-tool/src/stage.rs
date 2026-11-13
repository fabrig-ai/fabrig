//! Stage only missing binaries on each host (temper/docs/design/redeploy.md, 3).
use crate::{
    description::Description,
    hosts::{Hosts, Part, words},
    record::Journal,
};

#[must_use]
pub fn valid_commit(commit: &str) -> bool {
    commit.len() == 12 && commit.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
/// The paths and binary names a part owns, in deployment order.
#[must_use]
pub fn binaries(description: &Description, part: Part) -> (&str, &str, [&'static str; 2]) {
    match part {
        Part::Engine => (&description.engine.binaries, &description.engine.builds, ["temper-engine", "temper"]),
        Part::Worker(index) => {
            let worker = &description.workers[index];
            (&worker.binaries, &worker.builds, ["temper-worker", "temper-agent"])
        }
    }
}
#[must_use]
pub fn part_name(description: &Description, part: Part) -> &str {
    match part {
        Part::Engine => "engine",
        Part::Worker(index) => &description.workers[index].name,
    }
}
#[must_use]
pub fn version(text: &str, name: &str, commit: &str) -> bool {
    let fields: Vec<_> = text.split_whitespace().collect();
    fields.len() == 3 && fields[0] == name && !fields[1].is_empty() && fields[2] == format!("({commit})")
}
pub fn stage(
    description: &Description,
    commit: &str,
    journal: &mut Journal,
    hosts: &mut dyn Hosts,
) -> Result<(), String> {
    if !valid_commit(commit) {
        return Err("COMMIT must be twelve lowercase hexadecimal digits".into());
    }
    for part in std::iter::once(Part::Engine).chain((0..description.workers.len()).map(Part::Worker)) {
        let (_, builds, names) = binaries(description, part);
        let directory = format!("{builds}/{commit}");
        let mut missing = Vec::new();
        for name in names {
            let binary = format!("{directory}/{name}");
            let answer = hosts.run(part, &words(&[&binary, "--version"]));
            if answer.code == Some(0) && version(&answer.stdout, name, commit) {
                note(journal, hosts, &format!("{} {binary} staged", part_name(description, part)))?;
            } else {
                missing.push(name);
            }
        }
        if missing.is_empty() {
            continue;
        }
        let fetched = hosts.run(part, &fetch(&description.source, commit, &directory, &missing));
        if fetched.code != Some(0) {
            let paths = missing.iter().map(|name| format!("{directory}/{name}")).collect::<Vec<_>>().join(", ");
            return Err(format!("{} {paths}: fetch failed: {}", part_name(description, part), fetched.stderr.trim()));
        }
        for name in missing {
            let binary = format!("{directory}/{name}");
            let answer = hosts.run(part, &words(&[&binary, "--version"]));
            if answer.code != Some(0) || !version(&answer.stdout, name, commit) {
                return Err(format!(
                    "{} {binary}: expected {name} VERSION ({commit}), got {:?}: {}",
                    part_name(description, part),
                    answer.stdout.trim(),
                    answer.stderr.trim()
                ));
            }
            note(journal, hosts, &format!("{} {binary} fetched and staged", part_name(description, part)))?;
        }
    }
    Ok(())
}
fn note(journal: &mut Journal, hosts: &mut dyn Hosts, text: &str) -> Result<(), String> {
    for line in text.lines() {
        journal.line(hosts.now(), &format!("temper-redeploy stage: {line}"))?;
    }
    Ok(())
}
fn fetch(source: &str, commit: &str, directory: &str, missing: &[&str]) -> String {
    let source = source.trim_end_matches('/');
    let mut lines = vec![
        words(&["mkdir", "-p", &format!("{directory}/Linux-x86_64")]),
        words(&["cd", directory]),
        words(&["curl", "-fLO", &format!("{source}/{commit}/SHA256SUMS")]),
    ];
    for name in missing {
        lines.push(words(&[
            "curl",
            "-fL",
            &format!("{source}/{commit}/Linux-x86_64/{name}"),
            "-o",
            &format!("Linux-x86_64/{name}"),
        ]));
    }
    lines.push(words(&["sha256sum", "-c", "--ignore-missing", "SHA256SUMS"]));
    for name in missing {
        lines.push(format!(
            "{} > {}",
            words(&["gunzip", "-c", &format!("Linux-x86_64/{name}")]),
            crate::hosts::quote(name)
        ));
        lines.push(words(&["chmod", "0755", name]));
    }
    lines.join(" && ")
}
