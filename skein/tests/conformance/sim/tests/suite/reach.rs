//! Reach scenarios draw nothing: calm seeds only (testing/conformance.md, 1).

use skein_conformance::reach;
use skein_conformance_sim::{CALM, each_seed};
use skein_sim::Config;

#[test]
fn granted_and_refused() {
    each_seed(Config::calm(), CALM, |world| reach::granted_and_refused(world, b"reach_probe"));
}

#[test]
fn no_privilege_gained() {
    each_seed(Config::calm(), CALM, |world| reach::no_privilege_gained(world, b"reach_probe"));
}

#[test]
fn signal_to_parent() {
    each_seed(Config::calm(), CALM, |world| reach::signal_to_parent(world, b"reach_probe"));
}

#[test]
fn missing_abi() {
    each_seed(Config::calm(), CALM, |world| reach::missing_abi(world, b"reach_probe"));
}

#[test]
fn missing_rule() {
    each_seed(Config::calm(), CALM, |world| reach::missing_rule(world, b"reach_probe"));
}

#[test]
fn child_narrows() {
    each_seed(Config::calm(), CALM, |world| reach::child_narrows(world, b"reach_probe"));
}

#[test]
fn descriptors() {
    each_seed(Config::calm(), CALM, |world| reach::descriptors(world, b"reach_probe"));
}

#[test]
fn landlock_absent() {
    each_seed(Config::calm(), CALM, |world| reach::landlock_absent(world, b"reach_probe"));
}

#[test]
fn landlock_disabled() {
    each_seed(Config::calm(), CALM, |world| reach::landlock_disabled(world, b"reach_probe"));
}

#[test]
fn landlock_older_abi() {
    each_seed(Config::calm(), CALM, |world| reach::landlock_older_abi(world, b"reach_probe"));
}
