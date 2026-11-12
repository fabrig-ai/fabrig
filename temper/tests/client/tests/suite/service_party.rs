//! An external ctl directory signs in as a service (client/ctl/README.md, 7).
mod machine;
use machine::Machine;

#[test]
fn an_owner_makes_a_service_and_a_second_ctl_directory_acts_as_it() {
    let mut world = Machine::new();
    world.call("owner", &[b"sign-in", b"link"]);
    let made = world.call(
        "owner",
        &[b"party", b"make", b"--project", b"1", b"--role", b"2", b"--out", b"service", b"scout", b"--key", b"scout"],
    );
    let party = made.trim().strip_prefix("made: party ").expect("made party").parse::<u64>().expect("party number");
    assert_ne!(party, 1, "the service is not the owner");
    world.private_file("service");
    let signed = world.call("agent", &[b"sign-in", b"--service", b"service"]);
    let me = world.call("agent", &[b"me"]);
    for text in [&signed, &me] {
        assert!(text.starts_with(&format!("party {party} · ")), "service party: {text}");
        assert!(text.lines().next().expect("party line").ends_with(" · scout"), "service name: {text}");
        assert!(
            text.lines().any(|line| line.starts_with("  1 · ") && line.contains(" · member")),
            "project role: {text}"
        );
    }
    world.private_file("agent/token");
    world.private_file("agent/address");
}
