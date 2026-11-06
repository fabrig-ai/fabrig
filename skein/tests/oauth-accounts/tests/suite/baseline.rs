//! What each story does, as digests, while D048's owners merge. Deleted by
//! the plan's `oauth-close` (skein/docs/foundation/testing-strategy.md, 10).
use skein_oauth_accounts_world::{accounts, private, private_cuts, shared, sign_in, sign_out, world};
use skein_world::{Cut, Memory};
use std::fmt::Write;

// Fixed seeds name the runs; faulted seeds are the focused and fuzzy suite's cases.
const SEED: u64 = 97;
const PROTOCOL_SEEDS: [u64; 2] = [67, 97];

/// The first eight bytes of a SHA-256, in hex.
fn short(text: &str) -> String {
    let mut hex = String::new();
    for byte in skein_lib::sha256::digest(text.as_bytes()).iter().take(8) {
        write!(hex, "{byte:02x}").expect("writing to a String");
    }
    hex
}

/// One row: the story, how it ran, what the sim saw, what the owner saw.
fn row(name: &str, trace: &str, facts: &str) -> String {
    format!("{name} {} {}\n", short(trace), short(facts))
}

fn private_facts(facts: &impl std::fmt::Debug, machine: &private::FileSystem) -> String {
    format!(
        "{facts:?} reads={} writes={} syncs={} renames={} faulted={}",
        machine.reads, machine.writes, machine.syncs, machine.renames, machine.faulted
    )
}

// The protocol flag is the script boundary in sign_in/protocol.rs. The last
// column records only faulted runs already exercised by the world's tests.
const SIGN_IN: &[(sign_in::Story, bool, &[u64])] = &[
    (sign_in::Story::Restart, true, &[22]),
    (sign_in::Story::Return(sign_in::Return::Back), true, &[7, 30]),
    (sign_in::Story::Return(sign_in::Return::Before), true, &[]),
    (sign_in::Story::Return(sign_in::Return::After), true, &[10]),
    (sign_in::Story::Return(sign_in::Return::Replace), true, &[8, 31]),
    (sign_in::Story::Return(sign_in::Return::Long), true, &[]),
    (sign_in::Story::Return(sign_in::Return::Space), true, &[]),
    (sign_in::Story::Return(sign_in::Return::Scheme), true, &[]),
    (sign_in::Story::Return(sign_in::Return::Empty), true, &[]),
    (sign_in::Story::Return(sign_in::Return::Control), true, &[]),
    (sign_in::Story::Return(sign_in::Return::NonAscii), true, &[]),
    (sign_in::Story::Return(sign_in::Return::Bound), true, &[]),
    (sign_in::Story::Return(sign_in::Return::WrongPath), true, &[9]),
    (sign_in::Story::OpenIdAgain, true, &[]),
    (sign_in::Story::IssuedClientId, true, &[]),
    (sign_in::Story::AnotherSubject, true, &[]),
    (sign_in::Story::RequiredScopes, true, &[]),
    (sign_in::Story::Paste, true, &[11]),
    (sign_in::Story::PasteThenListener, true, &[12]),
    (sign_in::Story::ListenerThenPaste, true, &[13]),
    (sign_in::Story::PasteTwice, false, &[14]),
    (sign_in::Story::PasteWrong, false, &[15]),
    (sign_in::Story::StateRefusals, false, &[16]),
    (sign_in::Story::ListenerWrongState, false, &[17]),
    (sign_in::Story::PasteExpired, false, &[18]),
    (sign_in::Story::PasteCancelled, false, &[19]),
    (sign_in::Story::PasteClose, false, &[20]),
    (sign_in::Story::Public, true, &[21]),
    (sign_in::Story::VaryingTaken, true, &[89]),
    (sign_in::Story::TwoPorts, true, &[]),
    (sign_in::Story::Confidential, true, &[6, 29]),
    (sign_in::Story::WrongPath, true, &[0, 23]),
    (sign_in::Story::WrongHost, true, &[1, 24]),
    (sign_in::Story::LongHead, true, &[2, 25]),
    (sign_in::Story::CloseWaiting, true, &[3, 26]),
    (sign_in::Story::CloseConfidential, true, &[]),
    (sign_in::Story::Cancel, true, &[4, 27]),
    (sign_in::Story::Abort, true, &[5, 28]),
    (sign_in::Story::NotKept, true, &[57]),
    (sign_in::Story::Timeout, true, &[]),
];

#[test]
#[ignore = "D048 baseline, deleted by oauth-close"]
fn sign_in_run() {
    let mut lines = String::new();
    for &(story, _, faulted_seeds) in SIGN_IN {
        for (seed, faulted) in std::iter::once((SEED, false)).chain(faulted_seeds.iter().map(|&seed| (seed, true))) {
            let outcome = sign_in::run(seed, story, faulted, false, Memory::Unchecked);
            lines += &row(
                &format!("{story:?}:{seed}:faulted={faulted}"),
                &skein_sim::render(seed, &outcome.trace),
                &format!("{:?}", sign_in::facts(&outcome.procs)),
            );
        }
    }
    assert_eq!(lines, SIGN_IN_RUN, "a story's digest moved; every row is printed for a paste");
}

#[test]
#[ignore = "D048 baseline, deleted by oauth-close"]
fn sign_in_protocol() {
    let mut lines = String::new();
    for &(story, _, _) in SIGN_IN.iter().filter(|(_, protocol, _)| *protocol) {
        for seed in PROTOCOL_SEEDS {
            let (facts, trace) = sign_in::protocol(seed, story);
            lines += &row(&format!("{story:?}:{seed}"), &trace.join("\n"), &format!("{facts:?}"));
        }
    }
    assert_eq!(lines, SIGN_IN_PROTOCOL, "a story's digest moved; every row is printed for a paste");
}

#[test]
#[ignore = "D048 baseline, deleted by oauth-close"]
fn sign_in_run_private() {
    use private::Story;
    let mut lines = String::new();
    for story in [
        Story::Missing,
        Story::Unreadable,
        Story::RootMode,
        Story::FileMode,
        Story::FileOwner,
        Story::FileLink,
        Story::FileLinks,
        Story::LoadFailed,
        Story::LoadStalled,
        Story::TooLarge,
    ] {
        let mut outcome = sign_in::run_private(SEED, story);
        lines += &row(
            &format!("{story:?}:{SEED}"),
            &skein_sim::render(SEED, &outcome.trace),
            &private_facts(&sign_in::facts(&outcome.procs), &outcome.machine),
        );
        outcome.machine.finish();
    }
    assert_eq!(lines, SIGN_IN_RUN_PRIVATE, "a story's digest moved; every row is printed for a paste");
}

#[test]
#[ignore = "D048 baseline, deleted by oauth-close"]
fn sign_in_run_private_return() {
    let mut outcome = sign_in::run_private_return(SEED);
    let lines = row(
        &format!("Return(Back):{SEED}"),
        &skein_sim::render(SEED, &outcome.trace),
        &private_facts(&sign_in::facts(&outcome.procs), &outcome.machine),
    );
    outcome.machine.finish();
    assert_eq!(lines, SIGN_IN_RUN_PRIVATE_RETURN, "a story's digest moved; every row is printed for a paste");
}

const WORLD: &[(world::Story, &[u64])] = &[
    (world::Story::Lead, &PROTOCOL_SEEDS),
    (world::Story::Rotate, &PROTOCOL_SEEDS),
    (world::Story::Omit, &PROTOCOL_SEEDS),
    (world::Story::NotKept, &PROTOCOL_SEEDS),
    (world::Story::Rejected, &PROTOCOL_SEEDS),
    (world::Story::Repeated, &PROTOCOL_SEEDS),
    (world::Story::Released, &PROTOCOL_SEEDS),
    (world::Story::CloseConnecting, &PROTOCOL_SEEDS),
    (world::Story::CloseSending, &PROTOCOL_SEEDS),
    (world::Story::CloseKeeping, &PROTOCOL_SEEDS),
    (world::Story::Abort, &[83, 97]),
];

#[test]
#[ignore = "D048 baseline, deleted by oauth-close"]
fn world_run() {
    let mut lines = String::new();
    for &(story, _) in WORLD.iter().chain(&[(world::Story::Retry, &[][..])]) {
        for faulted in [false, true] {
            let outcome = world::run(SEED, story, faulted, Memory::Unchecked);
            lines += &row(
                &format!("{story:?}:{SEED}:faulted={faulted}"),
                &skein_sim::render(SEED, &outcome.trace),
                &format!("{:?}", private::facts(&outcome.procs)),
            );
        }
    }
    assert_eq!(lines, WORLD_RUN, "a story's digest moved; every row is printed for a paste");
}

#[test]
#[ignore = "D048 baseline, deleted by oauth-close"]
fn world_protocol() {
    let mut lines = String::new();
    for &(story, seeds) in WORLD.iter().chain(&[(world::Story::HandedIn, PROTOCOL_SEEDS.as_slice())]) {
        for &seed in seeds {
            let (facts, trace) = world::protocol(seed, story);
            lines += &row(&format!("{story:?}:{seed}"), &trace.join("\n"), &format!("{facts:?}"));
        }
    }
    assert_eq!(lines, WORLD_PROTOCOL, "a story's digest moved; every row is printed for a paste");
}

#[test]
#[ignore = "D048 baseline, deleted by oauth-close"]
fn world_tls_close() {
    let mut lines = String::new();
    for room in [false, true] {
        lines += &row(&format!("room={room}:{SEED}"), "", &format!("{:?}", world::tls_close(SEED, room)));
    }
    assert_eq!(lines, WORLD_TLS_CLOSE, "a story's digest moved; every row is printed for a paste");
}

#[test]
#[ignore = "D048 baseline, deleted by oauth-close"]
fn private_run() {
    use private::Story;
    let mut lines = String::new();
    for story in [
        Story::SignInAgain,
        Story::NoIssuedId,
        Story::Load,
        Story::LoadFailed,
        Story::LoadStalled,
        Story::Fixed,
        Story::Conflict,
        Story::TooLarge,
        Story::CloseLoading,
        Story::AbortLoading,
        Story::Refresh,
        Story::RootMode,
        Story::FileMode,
        Story::FileOwner,
        Story::FileLink,
        Story::FileLinks,
        Story::Missing,
        Story::Unreadable,
        Story::KeepFailed,
        Story::KeepStalled,
        Story::CloseKeeping,
        Story::AbortKeeping,
    ] {
        let mut outcome = private::run(SEED, story);
        lines += &row(
            &format!("{story:?}:{SEED}"),
            &skein_sim::render(SEED, &outcome.trace),
            &private_facts(&private::facts(&outcome.procs), &outcome.machine),
        );
        outcome.machine.finish();
    }
    assert_eq!(lines, PRIVATE_RUN, "a story's digest moved; every row is printed for a paste");
}

#[test]
#[ignore = "D048 baseline, deleted by oauth-close"]
fn private_cuts_world() {
    let mut uncut = private_cuts::world(SEED).run();
    let before = uncut
        .trace
        .iter()
        .filter(|entry| entry.pid.raw() == 1 && matches!(entry.event, skein_sim::Event::Submit { .. }))
        .position(|entry| {
            matches!(entry.event, skein_sim::Event::Submit { kind: skein_sim::Summary::Rename { .. }, .. })
        })
        .expect("private rename submission");
    let mut lines = row(
        &format!("uncut:{SEED}"),
        &skein_sim::render(SEED, &uncut.trace),
        &private_facts(&private_cuts::recovered(&uncut.procs), &uncut.machine),
    );
    uncut.machine.finish();
    for cut in [Cut::Kill, Cut::PowerLoss] {
        let mut world = private_cuts::world(SEED);
        world.cut(1, u32::try_from(before).expect("bounded submissions"), cut);
        let mut outcome = world.run();
        lines += &row(
            &format!("{cut:?}:before={before}:{SEED}"),
            &skein_sim::render(SEED, &outcome.trace),
            &private_facts(&private_cuts::recovered(&outcome.procs), &outcome.machine),
        );
        outcome.machine.finish();
    }
    assert_eq!(lines, PRIVATE_CUTS_WORLD, "a story's digest moved; every row is printed for a paste");
}

const ACCOUNTS: &[accounts::Story] = &[
    accounts::Story::Idle,
    accounts::Story::Loading,
    accounts::Story::SigningIn,
    accounts::Story::Refreshing,
    accounts::Story::Keeping,
    accounts::Story::Holding,
    accounts::Story::Wanted,
    accounts::Story::WantedFailure,
];

#[test]
#[ignore = "D048 baseline, deleted by oauth-close"]
fn accounts_run() {
    let mut lines = String::new();
    for (story, seed) in ACCOUNTS.iter().map(|&story| (story, SEED)).chain([(accounts::Story::Keeping, 131)]) {
        let mut outcome = accounts::run(seed, story, Memory::Unchecked);
        lines += &row(
            &format!("{story:?}:{seed}"),
            &skein_sim::render(seed, &outcome.trace),
            &private_facts(&accounts::facts(&outcome.procs), &outcome.machine),
        );
        outcome.machine.finish();
    }
    assert_eq!(lines, ACCOUNTS_RUN, "a story's digest moved; every row is printed for a paste");
}

#[test]
#[ignore = "D048 baseline, deleted by oauth-close"]
fn accounts_protocol() {
    let mut lines = String::new();
    for (story, seed) in ACCOUNTS.iter().map(|&story| (story, SEED)).chain([(accounts::Story::Keeping, 131)]) {
        let (facts, trace) = accounts::protocol(seed, story);
        lines += &row(&format!("{story:?}:{seed}"), &trace.join("\n"), &format!("{facts:?}"));
    }
    assert_eq!(lines, ACCOUNTS_PROTOCOL, "a story's digest moved; every row is printed for a paste");
}

const SHARED: &[shared::Story] = &[
    shared::Story::Busy,
    shared::Story::Slow,
    shared::Story::Newer,
    shared::Story::NoRecord,
    shared::Story::Subject,
    shared::Story::Cancelled,
    shared::Story::Removed,
    shared::Story::Aborted,
];

#[test]
#[ignore = "D048 baseline, deleted by oauth-close"]
fn shared_run() {
    let mut lines = String::new();
    for (story, seed) in SHARED.iter().map(|&story| (story, SEED)) {
        let mut outcome = shared::run(seed, story, Memory::Unchecked);
        lines += &row(
            &format!("{story:?}:{seed}"),
            &skein_sim::render(seed, &outcome.trace),
            &private_facts(&shared::facts(&outcome.procs), &outcome.machine),
        );
        outcome.machine.finish();
    }
    assert_eq!(lines, SHARED_RUN, "a story's digest moved; every row is printed for a paste");
}

#[test]
#[ignore = "D048 baseline, deleted by oauth-close"]
fn shared_protocol() {
    let mut lines = String::new();
    for (story, seed) in SHARED.iter().map(|&story| (story, SEED)) {
        let (facts, trace) = shared::protocol(seed, story);
        lines += &row(&format!("{story:?}:{seed}"), &trace.join("\n"), &format!("{facts:?}"));
    }
    assert_eq!(lines, SHARED_PROTOCOL, "a story's digest moved; every row is printed for a paste");
}

const SIGN_OUT: &[sign_out::Story] = &[
    sign_out::Story::Confirmed,
    sign_out::Story::Unconfirmed,
    sign_out::Story::NoRevocation,
    sign_out::Story::Close,
    sign_out::Story::Abort,
];

#[test]
#[ignore = "D048 baseline, deleted by oauth-close"]
fn sign_out_run() {
    let mut lines = String::new();
    for (story, seed) in SIGN_OUT.iter().map(|&story| (story, SEED)) {
        for private in [false, true] {
            let mut outcome = sign_out::run(seed, story, private, Memory::Unchecked);
            lines += &row(
                &format!("{story:?}:{seed}:private={private}"),
                &skein_sim::render(seed, &outcome.trace),
                &private_facts(&sign_out::facts(&outcome.procs), &outcome.machine),
            );
            outcome.machine.finish();
        }
    }
    assert_eq!(lines, SIGN_OUT_RUN, "a story's digest moved; every row is printed for a paste");
}

#[test]
#[ignore = "D048 baseline, deleted by oauth-close"]
fn sign_out_protocol() {
    let mut lines = String::new();
    for (story, seed) in SIGN_OUT.iter().map(|&story| (story, SEED)) {
        let (facts, trace) = sign_out::protocol(seed, story);
        lines += &row(&format!("{story:?}:{seed}"), &trace.join("\n"), &format!("{facts:?}"));
    }
    assert_eq!(lines, SIGN_OUT_PROTOCOL, "a story's digest moved; every row is printed for a paste");
}

const SIGN_IN_RUN: &str = "\
Restart:97:faulted=false 46bf26d025d3fcd2 e48d4351a7e702fd
Restart:22:faulted=true c296ad30f16c793a e48d4351a7e702fd
Return(Back):97:faulted=false 74ae286bd1c657e2 965a0304a9e4e884
Return(Back):7:faulted=true 5a71aaa223151c77 965a0304a9e4e884
Return(Back):30:faulted=true 2653b9a23297a0e4 965a0304a9e4e884
Return(Before):97:faulted=false 45d32ee9fe02b7f6 965a0304a9e4e884
Return(After):97:faulted=false 5cdd7b2533560d52 f98592d5edf1cbcb
Return(After):10:faulted=true 606494863f5311f1 f98592d5edf1cbcb
Return(Replace):97:faulted=false 95d584862832898b 965a0304a9e4e884
Return(Replace):8:faulted=true dbf5c38b57714971 965a0304a9e4e884
Return(Replace):31:faulted=true f0defec34237427b 965a0304a9e4e884
Return(Long):97:faulted=false 45d32ee9fe02b7f6 965a0304a9e4e884
Return(Space):97:faulted=false 45d32ee9fe02b7f6 965a0304a9e4e884
Return(Scheme):97:faulted=false 45d32ee9fe02b7f6 965a0304a9e4e884
Return(Empty):97:faulted=false 45d32ee9fe02b7f6 965a0304a9e4e884
Return(Control):97:faulted=false 45d32ee9fe02b7f6 965a0304a9e4e884
Return(NonAscii):97:faulted=false 45d32ee9fe02b7f6 965a0304a9e4e884
Return(Bound):97:faulted=false 869c351ad6edb79b 965a0304a9e4e884
Return(WrongPath):97:faulted=false 4f0d29c957daa260 965a0304a9e4e884
Return(WrongPath):9:faulted=true 789e274869833ec7 965a0304a9e4e884
OpenIdAgain:97:faulted=false a6a3919b905bbc25 f98592d5edf1cbcb
IssuedClientId:97:faulted=false 4ebe9c3f39f2f1e6 965a0304a9e4e884
AnotherSubject:97:faulted=false 67ab8a4c30d3fe33 cc1a9d972c3ff0ec
RequiredScopes:97:faulted=false 6d95ee8c7950baae 62b8ca5d0b52c3de
Paste:97:faulted=false 87a0433391cff6bb a0f25dc38e12ccaf
Paste:11:faulted=true 330c9c0c75b19bd5 a0f25dc38e12ccaf
PasteThenListener:97:faulted=false 5873e0b0dd9f3fe9 a0f25dc38e12ccaf
PasteThenListener:12:faulted=true 7dca1e5b01cdea26 a0f25dc38e12ccaf
ListenerThenPaste:97:faulted=false 45d32ee9fe02b7f6 6b69a8a85e3f7092
ListenerThenPaste:13:faulted=true 9119c045ca265a8f 6b69a8a85e3f7092
PasteTwice:97:faulted=false 87a0433391cff6bb 2773288523f55b55
PasteTwice:14:faulted=true b6fdfaef4eb219ac 2773288523f55b55
PasteWrong:97:faulted=false 87a0433391cff6bb 019d52118af8c04b
PasteWrong:15:faulted=true ae14b02164ce9be8 019d52118af8c04b
StateRefusals:97:faulted=false 637f72222b0fdb90 05b0840c797ee573
StateRefusals:16:faulted=true f3bdf2b3340ba57e 05b0840c797ee573
ListenerWrongState:97:faulted=false 53c8298b7f8c2c57 965a0304a9e4e884
ListenerWrongState:17:faulted=true 6b196497492d863a 965a0304a9e4e884
PasteExpired:97:faulted=false 5eee04bbcdb3fd9a f2657c48b84f6340
PasteExpired:18:faulted=true 2a10e9f7f8b6a64d f2657c48b84f6340
PasteCancelled:97:faulted=false 7c66db4edf57d4ae 7ec35e6219b896fd
PasteCancelled:19:faulted=true b7d15d0b68abf500 7ec35e6219b896fd
PasteClose:97:faulted=false 87a0433391cff6bb e21bf8aecaf8d6e9
PasteClose:20:faulted=true 277f0621bcea9ef8 e21bf8aecaf8d6e9
Public:97:faulted=false 9cc8fae4d15d47a7 2c8494258bf0e73d
Public:21:faulted=true 7b856bbaa79cb9f7 2c8494258bf0e73d
VaryingTaken:97:faulted=false 78deed10f9e63d53 965a0304a9e4e884
VaryingTaken:89:faulted=true ae7aa7b6a63d91a1 965a0304a9e4e884
TwoPorts:97:faulted=false df1d4b2feaa7bc0e 74150df52ae3de24
Confidential:97:faulted=false dfd7953f3ff315aa d1c290609824313e
Confidential:6:faulted=true 182aeed67c2ec0dc d1c290609824313e
Confidential:29:faulted=true 99f13a7f30c67bb3 d1c290609824313e
WrongPath:97:faulted=false cb2024ca02fa8b21 965a0304a9e4e884
WrongPath:0:faulted=true a05a583d7d815279 965a0304a9e4e884
WrongPath:23:faulted=true fb6f135a77d5b7d8 965a0304a9e4e884
WrongHost:97:faulted=false b8b5d95f91e4dd92 965a0304a9e4e884
WrongHost:1:faulted=true 7841e53aa81f4890 965a0304a9e4e884
WrongHost:24:faulted=true edab48f7e99ce2fa 965a0304a9e4e884
LongHead:97:faulted=false 29564208b2cf935d 965a0304a9e4e884
LongHead:2:faulted=true b67ee3098a853ac8 965a0304a9e4e884
LongHead:25:faulted=true 07f2eb6dbd76ebb4 965a0304a9e4e884
CloseWaiting:97:faulted=false 45d32ee9fe02b7f6 8bb40a9d38c55028
CloseWaiting:3:faulted=true d641bf3dc8b089f3 8bb40a9d38c55028
CloseWaiting:26:faulted=true e03bb704732c9ba8 8bb40a9d38c55028
CloseConfidential:97:faulted=false dfd7953f3ff315aa e21bf8aecaf8d6e9
Cancel:97:faulted=false 6aca05497c328a7f f82893412927c529
Cancel:4:faulted=true 15156ca160e6f589 f82893412927c529
Cancel:27:faulted=true e7e1307d1d3d2bcd f82893412927c529
Abort:97:faulted=false 146188beca1acafb f82893412927c529
Abort:5:faulted=true fa68e5f33cd71c27 f82893412927c529
Abort:28:faulted=true bebfd861c94fe44d f82893412927c529
NotKept:97:faulted=false 45d32ee9fe02b7f6 17a930a961f81605
NotKept:57:faulted=true 20d74f3ec7b48731 17a930a961f81605
Timeout:97:faulted=false 39c2c37d6443df41 25665b76c13dff6c
";
const SIGN_IN_PROTOCOL: &str = "\
Restart:67 a18174c3bb9773eb e48d4351a7e702fd
Restart:97 7b67f585e39c545d e48d4351a7e702fd
Return(Back):67 c66641c48f04e121 965a0304a9e4e884
Return(Back):97 de60ba0b6e4abd96 965a0304a9e4e884
Return(Before):67 fac4c0c9dd5ace5b 965a0304a9e4e884
Return(Before):97 0909e4dcea97e89c 965a0304a9e4e884
Return(After):67 82fcda03be0011d3 f98592d5edf1cbcb
Return(After):97 a82e1e96fb98c75a f98592d5edf1cbcb
Return(Replace):67 be0a2330bdae94e6 965a0304a9e4e884
Return(Replace):97 a99ab73626bfd9f1 965a0304a9e4e884
Return(Long):67 fac4c0c9dd5ace5b 965a0304a9e4e884
Return(Long):97 0909e4dcea97e89c 965a0304a9e4e884
Return(Space):67 fac4c0c9dd5ace5b 965a0304a9e4e884
Return(Space):97 0909e4dcea97e89c 965a0304a9e4e884
Return(Scheme):67 fac4c0c9dd5ace5b 965a0304a9e4e884
Return(Scheme):97 0909e4dcea97e89c 965a0304a9e4e884
Return(Empty):67 fac4c0c9dd5ace5b 965a0304a9e4e884
Return(Empty):97 0909e4dcea97e89c 965a0304a9e4e884
Return(Control):67 fac4c0c9dd5ace5b 965a0304a9e4e884
Return(Control):97 0909e4dcea97e89c 965a0304a9e4e884
Return(NonAscii):67 fac4c0c9dd5ace5b 965a0304a9e4e884
Return(NonAscii):97 0909e4dcea97e89c 965a0304a9e4e884
Return(Bound):67 706d14165c941bb3 965a0304a9e4e884
Return(Bound):97 70d4ec0ec3ae7521 965a0304a9e4e884
Return(WrongPath):67 a9bcdb58aa1e3fa5 965a0304a9e4e884
Return(WrongPath):97 8ae50adc29cfda89 965a0304a9e4e884
OpenIdAgain:67 a9fcf84840b026e3 f98592d5edf1cbcb
OpenIdAgain:97 720b74390b545944 f98592d5edf1cbcb
IssuedClientId:67 b12ff5b6f8287116 965a0304a9e4e884
IssuedClientId:97 ab37eb84427f755b 965a0304a9e4e884
AnotherSubject:67 89b647cff037a805 cc1a9d972c3ff0ec
AnotherSubject:97 b1474d1cb7ab3c61 cc1a9d972c3ff0ec
RequiredScopes:67 c98d815cb0e49949 62b8ca5d0b52c3de
RequiredScopes:97 a23058bb6c58acb2 62b8ca5d0b52c3de
Paste:67 6c3c7b7663579caf a0f25dc38e12ccaf
Paste:97 93ec0dbd9ee96731 a0f25dc38e12ccaf
PasteThenListener:67 dc03b7550b4624ff a0f25dc38e12ccaf
PasteThenListener:97 1b57858ea4480079 a0f25dc38e12ccaf
ListenerThenPaste:67 b12cf5c7297c5f50 6b69a8a85e3f7092
ListenerThenPaste:97 f57ef7c85439fc3f 6b69a8a85e3f7092
Public:67 158a2b8789e66070 2c8494258bf0e73d
Public:97 41026ed5fb56ce1d 2c8494258bf0e73d
VaryingTaken:67 dc55711b6b69ae84 965a0304a9e4e884
VaryingTaken:97 3054c7e357139ca1 965a0304a9e4e884
TwoPorts:67 325d5095211e2fba 735ec75df1dd31ad
TwoPorts:97 94c404bb70dad162 735ec75df1dd31ad
Confidential:67 34c5b2a4a16b9849 d1c290609824313e
Confidential:97 913d74726cc5d3a2 d1c290609824313e
WrongPath:67 b361c05900be0c0b 965a0304a9e4e884
WrongPath:97 33c0734ba9634472 965a0304a9e4e884
WrongHost:67 648541a0061aef33 965a0304a9e4e884
WrongHost:97 10928c6f841c99d0 965a0304a9e4e884
LongHead:67 25a2b0718344eb44 965a0304a9e4e884
LongHead:97 c4dab99d89f130c9 965a0304a9e4e884
CloseWaiting:67 fac4c0c9dd5ace5b 8bb40a9d38c55028
CloseWaiting:97 0909e4dcea97e89c 8bb40a9d38c55028
CloseConfidential:67 34c5b2a4a16b9849 e21bf8aecaf8d6e9
CloseConfidential:97 913d74726cc5d3a2 e21bf8aecaf8d6e9
Cancel:67 e3b0c44298fc1c14 f82893412927c529
Cancel:97 e3b0c44298fc1c14 f82893412927c529
Abort:67 e3b0c44298fc1c14 f82893412927c529
Abort:97 e3b0c44298fc1c14 f82893412927c529
NotKept:67 fac4c0c9dd5ace5b 17a930a961f81605
NotKept:97 0909e4dcea97e89c 17a930a961f81605
Timeout:67 e3b0c44298fc1c14 25665b76c13dff6c
Timeout:97 e3b0c44298fc1c14 25665b76c13dff6c
";
const SIGN_IN_RUN_PRIVATE: &str = "\
Missing:97 5f6f7da19658e315 8598ed2734f0b335
Unreadable:97 09f8b142e7c9a88e 17e99837c42afc55
RootMode:97 f001ac74a26fb228 b88917bfca36bbcd
FileMode:97 fdeaa9d5b7cd9d41 b1a3ecaca0d69a9d
FileOwner:97 0758d97f891e4e88 74c4caf68709fb0f
FileLink:97 1ef2406125756276 be8fcad09fb27dac
FileLinks:97 ad2c3edb5d387872 1f4116912e9e6ab9
LoadFailed:97 b77dee12ad26006d 7516d57544211113
LoadStalled:97 53eab43f56cbf8e7 b9787517fef6447e
TooLarge:97 912804adc922f2ef ecb5a62d9549ea29
";
const SIGN_IN_RUN_PRIVATE_RETURN: &str = "\
Return(Back):97 7e51ad6e54e9aae7 4054afd6409fc199
";
const WORLD_RUN: &str = "\
Lead:97:faulted=false 7f7b3273dd93b7f3 cfafa3a4b4608a1f
Lead:97:faulted=true b7fda9e4b1e82f13 cfafa3a4b4608a1f
Rotate:97:faulted=false fcc0b8c11b29bdd1 e61355a927df0c02
Rotate:97:faulted=true 96b6d685e8387af3 e61355a927df0c02
Omit:97:faulted=false 49f59e14b25a89ce e61355a927df0c02
Omit:97:faulted=true dc347a55b5dcb250 e61355a927df0c02
NotKept:97:faulted=false fcc0b8c11b29bdd1 2e64b37b8d5879c0
NotKept:97:faulted=true 96b6d685e8387af3 2e64b37b8d5879c0
Rejected:97:faulted=false fcc0b8c11b29bdd1 cfafa3a4b4608a1f
Rejected:97:faulted=true 96b6d685e8387af3 cfafa3a4b4608a1f
Repeated:97:faulted=false cd966daa60139382 4bba17d49ab7922f
Repeated:97:faulted=true cd966daa60139382 4bba17d49ab7922f
Released:97:faulted=false 4c0b93fb07f4e6b5 1623b9a97b7d3897
Released:97:faulted=true 4c0b93fb07f4e6b5 1623b9a97b7d3897
CloseConnecting:97:faulted=false fcc0b8c11b29bdd1 e61355a927df0c02
CloseConnecting:97:faulted=true 96b6d685e8387af3 e61355a927df0c02
CloseSending:97:faulted=false fcc0b8c11b29bdd1 e61355a927df0c02
CloseSending:97:faulted=true 96b6d685e8387af3 e61355a927df0c02
CloseKeeping:97:faulted=false fcc0b8c11b29bdd1 e61355a927df0c02
CloseKeeping:97:faulted=true 96b6d685e8387af3 e61355a927df0c02
Abort:97:faulted=false f0caaacddbf08a21 dbf88011a5e8c246
Abort:97:faulted=true f0caaacddbf08a21 dbf88011a5e8c246
Retry:97:faulted=false e94187f8590eef83 e61355a927df0c02
Retry:97:faulted=true f50e2236a4e401f9 e61355a927df0c02
";
const WORLD_PROTOCOL: &str = "\
Lead:67 1039366d0599e439 cfafa3a4b4608a1f
Lead:97 75d31413e9313c3d cfafa3a4b4608a1f
Rotate:67 bd01034bf168461d e61355a927df0c02
Rotate:97 7e4a123a588e3036 e61355a927df0c02
Omit:67 cab8d30871dfccb8 e61355a927df0c02
Omit:97 55dbc8ad13314078 e61355a927df0c02
NotKept:67 bd01034bf168461d 2e64b37b8d5879c0
NotKept:97 7e4a123a588e3036 2e64b37b8d5879c0
Rejected:67 bd01034bf168461d cfafa3a4b4608a1f
Rejected:97 7e4a123a588e3036 cfafa3a4b4608a1f
Repeated:67 e3b0c44298fc1c14 4bba17d49ab7922f
Repeated:97 e3b0c44298fc1c14 4bba17d49ab7922f
Released:67 e3b0c44298fc1c14 1623b9a97b7d3897
Released:97 e3b0c44298fc1c14 1623b9a97b7d3897
CloseConnecting:67 bd01034bf168461d e61355a927df0c02
CloseConnecting:97 7e4a123a588e3036 e61355a927df0c02
CloseSending:67 bd01034bf168461d e61355a927df0c02
CloseSending:97 7e4a123a588e3036 e61355a927df0c02
CloseKeeping:67 bd01034bf168461d e61355a927df0c02
CloseKeeping:97 7e4a123a588e3036 e61355a927df0c02
Abort:83 bd08394250e541ca dbf88011a5e8c246
Abort:97 bd08394250e541ca dbf88011a5e8c246
HandedIn:67 e3b0c44298fc1c14 52e7144a7931db49
HandedIn:97 e3b0c44298fc1c14 52e7144a7931db49
";
const WORLD_TLS_CLOSE: &str = "\
room=false:97 e3b0c44298fc1c14 8cbedaf4037335ce
room=true:97 e3b0c44298fc1c14 6d8a2864896e9967
";
const PRIVATE_RUN: &str = "\
SignInAgain:97 5f9e3bb7f6e5801b b6819f586d1e34ea
NoIssuedId:97 2a66ab6983268599 d0153e6b20314e97
Load:97 dc30d4e7b855c3ad d738ab241912d9ad
LoadFailed:97 992f6db09a4341a4 b28aa8f4cb8941b3
LoadStalled:97 5c721cea8fb761c0 a14d60afa107a561
Fixed:97 e2299a7b8ef1d00b b28aa8f4cb8941b3
Conflict:97 8109850c6300e520 22596848e031ded5
TooLarge:97 912804adc922f2ef ecb5a62d9549ea29
CloseLoading:97 dc30d4e7b855c3ad d738ab241912d9ad
AbortLoading:97 a0110786d6114443 db5676d22d5c03e3
Refresh:97 7d42fcd0a92f20f5 ca607a3e63e051b0
RootMode:97 f001ac74a26fb228 b88917bfca36bbcd
FileMode:97 fdeaa9d5b7cd9d41 b1a3ecaca0d69a9d
FileOwner:97 0758d97f891e4e88 74c4caf68709fb0f
FileLink:97 1ef2406125756276 be8fcad09fb27dac
FileLinks:97 ad2c3edb5d387872 1f4116912e9e6ab9
Missing:97 4d66188b1be91c67 4f65b5a8d255650b
Unreadable:97 80d8661491ac7452 a6ad36ab593e52df
KeepFailed:97 d15db5cfba35ef3b 3f2214d0768f46f6
KeepStalled:97 88b322b93fe78ad5 287542eba025afed
CloseKeeping:97 7d42fcd0a92f20f5 ca607a3e63e051b0
AbortKeeping:97 561fd45ee10618f0 6105031ad07b2e5e
";
const PRIVATE_CUTS_WORLD: &str = "\
uncut:97 f3211b75b904c7d8 ae8b974c1c65d94f
Kill:before=31:97 d12170e42c42b823 b05e69d0302c2a4a
PowerLoss:before=31:97 40d82bb055da5320 b05e69d0302c2a4a
";
const ACCOUNTS_RUN: &str = "\
Idle:97 878a849bc9671d45 db046a5d2992691b
Loading:97 de78f7bead40d29b 193c68a80dbc0d61
SigningIn:97 8cdd5bffa66e8d7d 4a6087e414ea5177
Refreshing:97 95757302effaa6b2 c85dec7bd5486035
Keeping:97 95757302effaa6b2 5d1d476fba346f80
Holding:97 878a849bc9671d45 13541cdc7e38c3b5
Wanted:97 15a657b48d62ba3e 2cd14b6ec2a178cf
WantedFailure:97 524f17a4a184622a 4089f0e7ecdac45b
Keeping:131 17008a6d5aca4b77 5d1d476fba346f80
";
const ACCOUNTS_PROTOCOL: &str = "\
Idle:97 e3b0c44298fc1c14 b8651fb5c39f8ba9
Loading:97 e3b0c44298fc1c14 e544e8a5ea18f68b
SigningIn:97 e3b0c44298fc1c14 1af23a73e386e5bd
Refreshing:97 7e4a123a588e3036 38aba6d4fe17208d
Keeping:97 7e4a123a588e3036 bcd07217428db810
Holding:97 e3b0c44298fc1c14 17184dbc610ebae0
Wanted:97 75d31413e9313c3d e23338e9c49d39f5
WantedFailure:97 b9c79f05bdb337ff 1811c4f1a51ba369
Keeping:131 7c27d43f31b48ce5 bcd07217428db810
";
const SHARED_RUN: &str = "\
Busy:97 cd966daa60139382 4df2d8471efd2890
Slow:97 e1e73ca484df2071 4df2d8471efd2890
Newer:97 cd966daa60139382 a62f9110dae0df82
NoRecord:97 cd966daa60139382 70f025e897d1048d
Subject:97 cd966daa60139382 decb160abf36a388
Cancelled:97 cd966daa60139382 5a159f6970ea0229
Removed:97 cd966daa60139382 35d5704e0461eebd
Aborted:97 cd966daa60139382 5a159f6970ea0229
";
const SHARED_PROTOCOL: &str = "\
Busy:97 e3b0c44298fc1c14 35cc635260ae5e63
Slow:97 e3b0c44298fc1c14 35cc635260ae5e63
Newer:97 e3b0c44298fc1c14 3e0982785ceb2f6f
NoRecord:97 e3b0c44298fc1c14 1b4af4da5935fe5e
Subject:97 e3b0c44298fc1c14 bac2783086d28b05
Cancelled:97 e3b0c44298fc1c14 182f1a82b3a0e61f
Removed:97 e3b0c44298fc1c14 9f2f37fb7327a1ee
Aborted:97 e3b0c44298fc1c14 182f1a82b3a0e61f
";
const SIGN_OUT_RUN: &str = "\
Confirmed:97:private=false da8df1601e57a765 621c70123baca50f
Confirmed:97:private=true 676ef94d7a971434 04fef79d4e7db774
Unconfirmed:97:private=false d0d97160060c03cb 8c536240de176afd
Unconfirmed:97:private=true d3362025f3d93c22 670b4f2d1df256ed
NoRevocation:97:private=false 878a849bc9671d45 8c536240de176afd
NoRevocation:97:private=true ba15b4a53ebed32b 670b4f2d1df256ed
Close:97:private=false da8df1601e57a765 621c70123baca50f
Close:97:private=true 676ef94d7a971434 04fef79d4e7db774
Abort:97:private=false 878a849bc9671d45 61c810d9481b7997
Abort:97:private=true 878a849bc9671d45 61c810d9481b7997
";
const SIGN_OUT_PROTOCOL: &str = "\
Confirmed:97 c161cef2bb509e73 c906cc0e9973310c
Unconfirmed:97 9b982ebcb0c7fb88 849b65c54d6168c2
NoRevocation:97 e3b0c44298fc1c14 849b65c54d6168c2
Close:97 c161cef2bb509e73 c906cc0e9973310c
Abort:97 e3b0c44298fc1c14 dbf88011a5e8c246
";
