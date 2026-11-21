//! The live Claude story's check of what a program run's first prompt offers
//! (smith/docs/design/protocol/programs.md, 5.3 and 5.5).

/// What a program run granted inspect, modify and shell offers in its first prompt, under smith's
/// names: the six built-ins and the permission prompt.
pub const OFFERED: [&str; 7] = ["Read", "Glob", "Grep", "Write", "Edit", "Bash", "permission_prompt"];

/// The first way `tools` departs from what a result-contract run on those families is offered, in
/// fixed words naming the tool: a name of `OFFERED` missing, `finish` offered, or `sub_agent`
/// offered. `None` when it departs in none.
#[must_use]
pub fn offered_tools_fault(tools: &[&[u8]]) -> Option<String> {
    for name in OFFERED {
        if !tools.contains(&name.as_bytes()) {
            return Some(format!("the first prompt offers no {name}"));
        }
    }
    if tools.contains(&b"finish".as_slice()) {
        return Some("the first prompt offers finish under a result contract".into());
    }
    if tools.contains(&b"sub_agent".as_slice()) {
        return Some("the first prompt offers sub_agent".into());
    }
    None
}
