//! The large render: what "the defaults fit a modest machine" means (hearth/docs/design/domain/render/README.md, 5.4).
use super::{File, Scenario};
use skein_io::kernel::Done;
use skein_sim::{Entry, Event, Summary};
use std::collections::{BTreeMap, BTreeSet};

pub const ENTRIES: usize = 4_096;
pub const DOCUMENTS: usize = 512;
pub const TARGETS: usize = 64;

// Fleet, environment and hearth, then each host's base and production documents.
const SHARED_DOCUMENTS: usize = 3 + 2 * TARGETS;
// Each service has a declaration and at least two configuration templates.
const TASKS: usize = (DOCUMENTS - SHARED_DOCUMENTS) / 3;
const COMMON_TASKS: usize = 2;
// Shared assets range from small configuration tables to larger service resources.
const STATIC_SIZES: [usize; 4] = [1_024, 2_048, 4_096, 8_192];

#[must_use]
pub fn scenario() -> Scenario<'static> {
    let mut files = super::THIN.to_vec();
    files.retain(|file| !file.path.starts_with("hosts/") && !file.path.starts_with("tasks/motd/"));
    files.iter_mut().find(|file| file.path == "envs/prod.json").expect("production environment").body =
        br#"{"_note":"Production defaults."}"#;
    for host in 0..TARGETS {
        let tasks = [
            0,
            1,
            COMMON_TASKS + 2 * host % (TASKS - COMMON_TASKS),
            COMMON_TASKS + (2 * host + 1) % (TASKS - COMMON_TASKS),
        ]
        .map(|task| format!(r#""service{task:03}":{{"message":"Service {task:03} is configured."}}"#))
        .join(",");
        files.push(file(
            format!("hosts/node{host:03}/node{host:03}.json"),
            format!(r#"{{"_note":"node{host:03}",{tasks}}}"#).into_bytes(),
        ));
        files.push(file(
            format!("hosts/node{host:03}/node{host:03}-prod.json"),
            format!(
                r#"{{"_note":"node{host:03} production","placement":{{"kind":"metal","firmware":"efi","hardware":"7th-gen"}}}}"#
            )
            .into_bytes(),
        ));
    }
    for task in 0..TASKS {
        files.push(file(
            format!("tasks/service{task:03}/task.json"),
            format!(
                r#"{{"doc":"Service {task:03}.","settings":{{"message":{{"type":"string","doc":"Status message."}}}}}}"#
            )
            .into_bytes(),
        ));
    }
    for template in 0..DOCUMENTS - SHARED_DOCUMENTS - TASKS {
        let task = template % TASKS;
        files.push(file(
            format!("tasks/service{task:03}/tree/etc/service{task:03}/status{template:03}.conf.j2"),
            format!("# Service {task:03}, configuration {template:03}\nmessage={{{{ settings.message }}}}\n")
                .into_bytes(),
        ));
    }
    let assets = STATIC_SIZES.map(|size| -> &'static [u8] { vec![b'x'; size].leak() });
    for entry in 0..ENTRIES - DOCUMENTS {
        let task = COMMON_TASKS + entry % (TASKS - COMMON_TASKS);
        files.push(File {
            path: format!("tasks/service{task:03}/tree/usr/local/share/service{task:03}/asset{entry:04}.dat").leak(),
            body: assets[entry % assets.len()],
            mode: 0o644,
        });
    }
    Scenario { files: files.leak(), ..super::thin() }
}

fn file(path: String, body: Vec<u8>) -> File {
    File { path: path.leak(), body: body.leak(), mode: 0o644 }
}

/// Count listing entries and decoded inputs served by hosted git.
/// Git emits one response per entry or blob; short writes finish that same response.
#[must_use]
pub fn served(trace: &[Entry]) -> (usize, usize) {
    let parents: BTreeSet<_> = trace
        .iter()
        .filter_map(|entry| {
            matches!(entry.event, Event::Submit { kind: Summary::Spawn { .. }, .. }).then_some(entry.pid)
        })
        .collect();
    let batches: BTreeSet<_> = trace
        .iter()
        .filter_map(|entry| {
            (!parents.contains(&entry.pid)
                && matches!(entry.event, Event::Submit { kind: Summary::PipeRead { .. }, .. }))
            .then_some(entry.pid)
        })
        .collect();
    let mut listings = BTreeMap::new();
    let mut documents = 0;
    for entry in trace {
        if !parents.contains(&entry.pid)
            && let Event::Complete {
                kind: Summary::PipeWrite { len, from, .. }, result: Ok(Done::Count(count)), ..
            } = entry.event
            && usize::try_from(from + count).expect("response length") == len
        {
            if batches.contains(&entry.pid) {
                // Every document response fits below the smallest static asset.
                if len < STATIC_SIZES[0] {
                    documents += 1;
                }
            } else {
                *listings.entry(entry.pid).or_insert(0) += 1;
            }
        }
    }
    // rev-parse has one response; ls-tree has one per revision entry.
    (listings.values().filter(|count| **count > 1).sum(), documents)
}
