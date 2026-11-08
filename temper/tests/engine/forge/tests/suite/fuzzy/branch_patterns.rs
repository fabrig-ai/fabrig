//! Branch-relative patterns share full resource paths' exact and open matching
//! (temper/docs/design/protocol/configuration.md, 13.2 and 13.3).
use skein_lib::Rng;
use temper_engine_domain_forge::{GateLast, GatePattern, Repository, covers_branch};
use temper_engine_forge_world::{REPO, World};

fn bytes(rng: &mut Rng, length: usize, alphabet: &[u8]) -> Box<[u8]> {
    (0..length)
        .map(|_| {
            alphabet[usize::try_from(rng.below(u64::try_from(alphabet.len()).expect("alphabet length")))
                .expect("byte index")]
        })
        .collect()
}

fn repository(rng: &mut Rng, base: &Repository) -> Repository {
    Repository {
        host: bytes(rng, 12, b"abc012"),
        owner: bytes(rng, 8, b"def345"),
        name: bytes(rng, 16, b"ghi678"),
        ..base.clone()
    }
}

#[test]
fn branch_patterns_match_full_patterns_across_repositories() {
    let mut world = World::new(0);
    world.adopt();
    let Some(temper_engine_domain_forge::Stored::Repository(base)) =
        world.stored().get(&temper_engine_domain_forge::Key::Repository(REPO))
    else {
        panic!("adopted fixture")
    };
    for seed in 0..64 {
        let mut rng = Rng::new(seed);
        let first = repository(&mut rng, base);
        let mut other = repository(&mut rng, base);
        // Distinct names even if the random fields happen to coincide.
        other.name = [other.name.as_ref(), b"other"].concat().into_boxed_slice();
        for parts in 1..=3 {
            let mut branch = Vec::new();
            for part in 0..parts {
                if part != 0 {
                    branch.push(b'/');
                }
                let length = usize::try_from(rng.between(1, 80)).expect("part length");
                branch.extend_from_slice(&bytes(&mut rng, length, b"abc09-"));
            }
            let first_part = branch.split(|byte| *byte == b'/').next().expect("first part");
            let prefix = usize::try_from(rng.between(0, u64::try_from(first_part.len()).expect("part length")))
                .expect("prefix length");
            let random = bytes(&mut rng, 16, b"ab09/");
            for last in [
                GateLast::Exact(branch.clone().into_boxed_slice()),
                GateLast::Exact(first_part.into()),
                GateLast::Open(first_part[..prefix].into()),
                GateLast::Open(Box::new([])),
                GateLast::Exact(random.clone()),
                GateLast::Open(random),
            ] {
                let relative = GatePattern { segments: Box::new([b"branch".as_slice().into()]), last: last.clone() };
                let full = GatePattern {
                    segments: Box::new([
                        b"forge".as_slice().into(),
                        first.host.clone(),
                        first.owner.clone(),
                        first.name.clone(),
                        b"branch".as_slice().into(),
                    ]),
                    last,
                };
                for name in [branch.as_slice(), b"a", b"a/long-branch-part/third", b"a//b", b"/a", b"a/", b""] {
                    let valid = name.split(|byte| *byte == b'/').all(|part| !part.is_empty());
                    let expected = valid
                        && match &relative.last {
                            GateLast::Exact(last) => name == last.as_ref() && !name.contains(&b'/'),
                            GateLast::Open(last) => {
                                name.split(|byte| *byte == b'/').next().expect("first part").starts_with(last)
                            }
                        };
                    assert_eq!(covers_branch(&relative, &first, name), expected, "seed {seed}: {relative:?}, {name:?}");
                    assert_eq!(covers_branch(&full, &first, name), expected, "seed {seed}: full pattern");
                    assert_eq!(covers_branch(&relative, &other, name), expected, "seed {seed}: other repository");
                    assert!(!covers_branch(&full, &other, name), "seed {seed}: full pattern stays in its repository");
                }
            }
        }
    }
}
