//! Front-neutral starting facts (jig/docs/design/client/domain/testing.md, 4–5).
use crate::joint::engine::Driver;
use skein_lib::{Time, Wall};

/// Facts installed before admitting any clients. Each front owns their translation.
#[derive(Clone, Debug)]
pub enum Seed {
    /// A signed-out owner with a project that permits starting conversations.
    ChatProject { party: u64, project: u32 },
    /// Project and deployment facts for the configured first owner.
    Profile,
    /// The next conversation completes with this retained report.
    ChatResult { report: Box<[u8]> },
    /// A pending worker proposal for a tracked goal, visible to its owner.
    PendingGoal { words: Box<[u8]>, budget: u64 },
    /// Authenticated project roles and their independently seeded work.
    Roles { story: RoleStory },
    /// Project facts shared by person stories.
    Project(crate::stories::shared::b_seed::Facts),
    /// An existing conversation belonging to a configured project member.
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
    if let Seed::ChatResult { report } = seed {
        driver.world.peers.as_mut().expect("ordinary core peers").workers =
            vec![jig_fake_workers::Worker::new(1, 1, vec![Box::new([jig_fake_workers::Script::Finish { report }])])];
        return;
    }
    if let Seed::PendingGoal { words, budget } = seed {
        driver.world.peers.as_mut().expect("ordinary core peers").workers = vec![jig_fake_workers::Worker::new(
            1,
            1,
            vec![
                Box::new([
                    jig_fake_workers::Script::Call(jig_fake_workers::Call::ProposeGoal { words, budget, priority: 7 }),
                    jig_fake_workers::Script::Park,
                ]),
                Box::new([jig_fake_workers::Script::Finish { report: b"one accepted goal".as_slice().into() }]),
            ],
        )];
        seed_core(driver, now, wall, Seed::Conversation { party: 1, project: 1, words: b"Hello".as_slice().into() });
        return;
    }
    let (Seed::ChatProject { party, project } | Seed::Conversation { party, project, .. }) = &seed else { return };
    assert_eq!(*project, 1, "the configured core project");
    assert!([1, 3].contains(party), "the configured owner or member");
    match seed {
        Seed::Project(_)
        | Seed::ChatProject { .. }
        | Seed::Profile
        | Seed::ChatResult { .. }
        | Seed::PendingGoal { .. }
        | Seed::Roles { .. } => {}
        Seed::Conversation { party, project, words } => {
            let peers = driver.world.peers.as_ref().expect("ordinary core peers");
            let number = u64::try_from(peers.parties.len()).expect("seed party count fits");
            let mut key = [0; 16];
            key[..8].copy_from_slice(b"seedchat");
            key[8..].copy_from_slice(&number.to_be_bytes());
            driver.scripted_party(
                Party::new(
                    project,
                    if party == 3 { Role::Member } else { Role::Owner },
                    1_000_000 + number * 100,
                    Box::new([
                        Act::SignIn { provider: 0, subject: (party + 6).to_be_bytes().into() },
                        Act::Request { key, ask: Ask::Chat { words } },
                    ]),
                ),
                now,
                wall,
            );
        }
    }
}

/// Starting work shared by people with project roles.
#[derive(Clone, Copy, Debug)]
pub enum RoleStory {
    /// A member's goal awaiting a holder's decision.
    Proposal,
    /// A role-addressed person task choosing between reports.
    Choice,
    /// A member whose goal exceeds their allotment.
    Goal,
    /// An observer without a creation offer.
    Observer,
}
