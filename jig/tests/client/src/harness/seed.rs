//! Front-neutral starting facts (jig/docs/design/client/domain/testing.md, 4–5).
use crate::joint::engine::Driver;
use skein_lib::{Time, Wall};

/// Facts installed before admitting any clients. Each front owns their translation.
#[derive(Clone, Debug)]
pub enum Seed {
    /// A signed-out owner with a project that permits starting conversations.
    ChatProject { party: u64, project: u32 },
    /// Project facts shared by person stories.
    Project(crate::stories::shared::b_seed::Facts),
    /// An existing conversation belonging to the project's owner.
    Conversation { party: u64, project: u32, words: Box<[u8]> },
}

/// Install facts through the joint core's ordinary start and person inputs.
/// The core fixture supplies project 1 and its first owner, party 1.
pub fn seed_core(driver: &mut Driver, now: Time, wall: Wall, seed: Seed) {
    use jig_fake_parties::{Act, Ask, Party, Role};
    if let Seed::Project(facts) = seed {
        crate::stories::shared::b_seed::core(driver, now, wall, facts);
        return;
    }
    let (Seed::ChatProject { party, project } | Seed::Conversation { party, project, .. }) = &seed else {
        unreachable!("project facts handled above")
    };
    assert_eq!((*party, *project), (1, 1), "the configured core fixture's project and owner");
    match seed {
        Seed::Project(_) => unreachable!("project facts handled above"),
        Seed::ChatProject { .. } => {}
        Seed::Conversation { project, words, .. } => {
            let peers = driver.world.peers.as_ref().expect("ordinary core peers");
            let number = u64::try_from(peers.parties.len()).expect("seed party count fits");
            let mut key = [0; 16];
            key[..8].copy_from_slice(b"seedchat");
            key[8..].copy_from_slice(&number.to_be_bytes());
            driver.scripted_party(
                Party::new(
                    project,
                    Role::Owner,
                    1_000_000 + number * 100,
                    Box::new([
                        Act::SignIn { provider: 0, subject: 7_u64.to_be_bytes().into() },
                        Act::Request { key, ask: Ask::Chat { words } },
                    ]),
                ),
                now,
                wall,
            );
        }
    }
}
