from pathlib import Path
b=Path('temper/crates');p=b/'temper-engine-domain-forge-landing/src/key.rs';s=p.read_text().replace('    TooLarge,','    TooLarge { bound: Option<KeyBound>, needed: u64 },');pos=s.index('/// Bounds of the filtered');s=s[:pos]+'''/// The configured filtered-tree capacity that refused the next entry.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum KeyBound {
    Entries,
    PathBytes,
}
''' +s[pos:];s=s.replace('Keyed::TooLarge','Keyed::TooLarge { bound: None, needed: 0 }')
s=s.replace('''if !expand(&path, tree, ignored, limits, &mut pending, &mut entries, &mut path_bytes) {
                    return Keyed::TooLarge { bound: None, needed: 0 };
                }''','''if let Err(problem) = expand(&path, tree, ignored, limits, &mut pending, &mut entries, &mut path_bytes) {
                    return problem;
                }''')
start=s.index('fn expand(');r=s[start:];r=r.replace(') -> bool {',') -> Result<(), Keyed> {',1).replace('return false;', 'return Err(Keyed::TooLarge { bound: None, needed: 0 });').replace('''if *entries >= limits.entries || length > usize::from(u16::MAX) {
                    return Err(Keyed::TooLarge { bound: None, needed: 0 });
                }''','''if *entries >= limits.entries {
                    return Err(Keyed::TooLarge { bound: Some(KeyBound::Entries), needed: u64::from(*entries).saturating_add(1) });
                }
                if length > usize::from(u16::MAX) {
                    return Err(Keyed::TooLarge { bound: None, needed: 0 });
                }''').replace('''let Some(total) = path_bytes.checked_add(u32::try_from(path.len()).expect("path fits u16")) else {
                    return Err(Keyed::TooLarge { bound: None, needed: 0 });
                };
                if total > limits.path_bytes {
                    return Err(Keyed::TooLarge { bound: None, needed: 0 });
                }
                *path_bytes = total;''','''let total = u64::from(*path_bytes).saturating_add(skein_lib::len_u64(path.len()));
                if total > u64::from(limits.path_bytes) {
                    return Err(Keyed::TooLarge { bound: Some(KeyBound::PathBytes), needed: total });
                }
                *path_bytes = u32::try_from(total).expect("path bytes below limit");''').replace('    true\n}', '    Ok(())\n}',1);s=s[:start]+r;p.write_text(s)
p=b/'temper-engine-domain-forge-landing/src/lib.rs';s=p.read_text().replace('EntryKind, KeyLimits','EntryKind, KeyBound, KeyLimits');p.write_text(s)
p=b/'temper-engine-domain-forge/src/landing.rs';s=p.read_text().replace('''policy::Keyed::TooLarge => emit(out, Request::Refused { task: p.task }),''','''policy::Keyed::TooLarge { bound: None, .. } => emit(out, Request::Refused { task: p.task }),
                policy::Keyed::TooLarge { bound: Some(bound), needed } => {
                    let (member, configured) = match bound {
                        policy::KeyBound::Entries => ("forge.key_entries", env.limits.key_entries),
                        policy::KeyBound::PathBytes => ("forge.key_path_bytes", env.limits.key_path_bytes),
                    };
                    crate::refusal::limit(out, p.task, repo, member, configured, needed);
                }''');p.write_text(s)
p=b/'temper-engine-domain-forge/src/tiers.rs';s=p.read_text().replace('landing::Keyed::TooLarge;', 'landing::Keyed::TooLarge { bound: None, needed: 0 };').replace('landing::Keyed::TooLarge =>','landing::Keyed::TooLarge { .. } =>');p.write_text(s)
p=b/'temper-engine-domain-forge-landing/src/tests/key.rs';s=p.read_text().replace('Keyed::TooLarge =>','Keyed::TooLarge { .. } =>').replace('Keyed::TooLarge);','Keyed::TooLarge { bound: None, needed: 0 });');s=s.replace('''            assert_eq!(result, Keyed::TooLarge { bound: None, needed: 0 });''','''            let bound = if limits.entries == 1 { KeyBound::Entries } else { KeyBound::PathBytes };
            let needed = if limits.entries == 1 { 2 } else { 3 };
            assert_eq!(result, Keyed::TooLarge { bound: Some(bound), needed });''');p.write_text(s)
p=Path('temper/tests/engine/forge/tests/suite/simulation.rs');s=p.read_text().replace('''assert!(world.seen().iter().any(|request| matches!(request, top::Request::Refused { task: 53 })));''','''assert!(world.seen().iter().any(|request| matches!(request, top::Request::ResourceRefused {
        task: 53, cause: top::RefusalCause::Limit { member: "forge.client.resources", configured, needed }, ..
    } if *configured == u64::from(LIMITS.client.resources) && *needed == u64::from(LIMITS.client.resources) + 1)));''');p.write_text(s)
