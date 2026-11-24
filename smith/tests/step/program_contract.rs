//! Offline controls for the live Claude story's offered-tools check.

#[test]
fn the_offered_tools_check_names_each_departure() {
    use smith_end_to_end_kit::program_contract::{OFFERED, offered_tools_fault};
    let offered: Vec<_> = OFFERED.iter().map(|name| name.as_bytes()).chain([b"remove".as_slice()]).collect();
    assert_eq!(offered_tools_fault(&offered), None, "the offered tools may include remove");
    for name in OFFERED {
        let missing: Vec<_> = offered.iter().copied().filter(|tool| *tool != name.as_bytes()).collect();
        assert_eq!(
            offered_tools_fault(&missing),
            Some(format!("the first prompt offers no {name}")),
            "a missing {name} is named"
        );
    }
    for (name, expected) in [
        ("finish", "the first prompt offers finish under a result contract"),
        ("sub_agent", "the first prompt offers sub_agent"),
    ] {
        let mut tools = offered.clone();
        tools.push(name.as_bytes());
        assert_eq!(offered_tools_fault(&tools).as_deref(), Some(expected), "an offered {name} is named");
    }
}
