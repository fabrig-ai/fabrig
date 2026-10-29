use skein_forge_setup_world::memory::{self, Kinds};

#[test]
fn forgejo_plaintext_host_stays_under_its_peak_ceiling() {
    // Bounds the plaintext Forgejo host with both run/connection slots occupied.
    const FORGEJO_PLAINTEXT_PEAK_CEILING: u64 = 65_000;
    let peak = check(Kinds::Forgejo, false, 0, FORGEJO_PLAINTEXT_PEAK_CEILING);
    assert!(peak <= FORGEJO_PLAINTEXT_PEAK_CEILING);
}
#[test]
fn forgejo_tls_host_stays_under_its_peak_ceiling() {
    // Bounds the TLS Forgejo host with both run/connection slots occupied.
    const FORGEJO_TLS_PEAK_CEILING: u64 = 130_000;
    let peak = check(Kinds::Forgejo, true, 0, FORGEJO_TLS_PEAK_CEILING);
    assert!(peak <= FORGEJO_TLS_PEAK_CEILING);
}
#[test]
fn github_plaintext_host_stays_under_its_peak_ceiling() {
    // Bounds the plaintext GitHub host with full run, connection and state slots.
    const GITHUB_PLAINTEXT_PEAK_CEILING: u64 = 78_000;
    let peak = check(Kinds::GitHub, false, 8, GITHUB_PLAINTEXT_PEAK_CEILING);
    assert!(peak <= GITHUB_PLAINTEXT_PEAK_CEILING);
}
#[test]
fn github_tls_host_stays_under_its_peak_ceiling() {
    // Bounds the TLS GitHub host with full run, connection and state slots.
    const GITHUB_TLS_PEAK_CEILING: u64 = 140_000;
    let peak = check(Kinds::GitHub, true, 8, GITHUB_TLS_PEAK_CEILING);
    assert!(peak <= GITHUB_TLS_PEAK_CEILING);
}
#[test]
fn mixed_host_stays_under_its_peak_ceiling() {
    // Bounds the TLS mixed-kind host with full run, connection and state slots.
    const MIXED_PEAK_CEILING: u64 = 140_000;
    let peak = check(Kinds::Both, true, 8, MIXED_PEAK_CEILING);
    assert!(peak <= MIXED_PEAK_CEILING);
}
fn check(kinds: Kinds, tls: bool, states: u32, ceiling: u64) -> u64 {
    let full = memory::full_heap(kinds, tls, ceiling);
    assert_eq!((full.runs, full.connections, full.states), (2, 2, states));
    assert!(full.peak > 0);
    println!(
        "{kinds:?}, tls={tls}: peak={}, ceiling={ceiling}, runs={}, connections={}, states={}",
        full.peak, full.runs, full.connections, full.states
    );
    full.peak
}
