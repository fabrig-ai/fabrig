//! The seed seam and its failure lines (`skein/docs/design/testing/sweeps.md`, 4.2–4.4).

use core::{cell::Cell, fmt};
use std::{io::Write, sync::Once};

use crate::{
    streams::{fnv, mix},
    test_env::{Mode, mode},
};

/// The calling test and, when it runs several sweeps, this sweep's part.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Test {
    package: &'static str,
    module: &'static str,
    name: &'static str,
    part: Option<&'static str>,
}

impl Test {
    #[must_use]
    pub const fn new(
        package: &'static str,
        module: &'static str,
        name: &'static str,
        part: Option<&'static str>,
    ) -> Self {
        Self { package, module, name, part }
    }
}

impl fmt::Display for Test {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(out, "{}::{}::{}", self.package, self.module, self.name)?;
        if let Some(part) = self.part {
            write!(out, "/{part}")?;
        }
        Ok(())
    }
}

/// The calling test's package, module and name, optionally with a sweep part.
#[macro_export]
macro_rules! this_test {
    ($name:literal) => {
        $crate::sweep::Test::new(
            ::core::env!("CARGO_PKG_NAME"),
            ::core::module_path!(),
            $name,
            ::core::option::Option::None,
        )
    };
    ($name:literal, $part:literal) => {
        $crate::sweep::Test::new(
            ::core::env!("CARGO_PKG_NAME"),
            ::core::module_path!(),
            $name,
            ::core::option::Option::Some($part),
        )
    };
}

/// The first failing case of a sweep and the finding its check returned.
#[derive(Debug)]
pub struct Case<E> {
    pub test: Test,
    pub seed: u64,
    pub finding: E,
}

impl<E: fmt::Display> fmt::Display for Case<E> {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        let location = Location { test: self.test, seed: self.seed };
        write!(out, "{location}: {}", self.finding)
    }
}

/// Judge the sweep's cases in the process's mode, stopping at the first finding.
pub fn cases<S, E, F>(test: Test, seeds: S, check: F) -> Result<(), Case<E>>
where
    S: IntoIterator<Item = u64>,
    E: fmt::Display,
    F: FnMut(u64) -> Result<(), E>,
{
    cases_in(mode(), test, seeds, check, stderr, |ran| skein_checks::count("seed", ran))
}

/// Whether a sweep runs its gate seeds, for configuration and checks over all seeds.
#[must_use]
pub fn gate() -> bool {
    mode().gate()
}

/// Report the digest of what the calling thread's case drew, only under replay.
pub fn input(digest: [u8; 32]) {
    input_in(mode, digest, stderr);
}

pub(crate) fn input_in(mode: impl FnOnce() -> Mode, digest: [u8; 32], report: impl FnOnce(fmt::Arguments<'_>)) {
    if let Ok(Some(Location { test, seed })) = ACTIVE.try_with(Cell::get)
        && matches!(mode(), Mode::Replay(_))
    {
        report(format_args!("fuzzy input: {test} seed {seed:016x} {}", InputDigest(digest)));
    }
}

struct InputDigest([u8; 32]);

impl fmt::Display for InputDigest {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.0 {
            write!(out, "{byte:02x}")?;
        }
        Ok(())
    }
}

pub(crate) fn cases_in<S, E, F, R, C>(
    mode: Mode,
    test: Test,
    seeds: S,
    check: F,
    mut report: R,
    count: C,
) -> Result<(), Case<E>>
where
    S: IntoIterator<Item = u64>,
    E: fmt::Display,
    F: FnMut(u64) -> Result<(), E>,
    R: FnMut(fmt::Arguments<'_>),
    C: FnOnce(u64),
{
    let (result, ran) = match mode {
        Mode::Gate => run(test, seeds, check),
        Mode::Campaign(seed) => {
            let base = mix(seed ^ fnv(&test.to_string()));
            let fresh = seeds.into_iter().zip(0_u64..).map(|(_, k)| mix(base ^ k));
            run(test, fresh, check)
        }
        Mode::Replay(seed) => run(test, core::iter::once(seed), check),
    };
    if let Err(case) = &result {
        report(format_args!("{case}"));
    }
    if !mode.gate() {
        report(format_args!("fuzzy cases: {test} {ran}"));
    }
    if ran > 0 {
        count(ran);
    }
    result
}

fn run<S, E, F>(test: Test, seeds: S, mut check: F) -> (Result<(), Case<E>>, u64)
where
    S: IntoIterator<Item = u64>,
    F: FnMut(u64) -> Result<(), E>,
{
    let mut ran = 0_u64;
    for seed in seeds {
        install_hook();
        let active = Active::enter(Location { test, seed });
        let result = check(seed);
        drop(active);
        ran = ran.checked_add(1).expect("a sweep's case count fits u64");
        if let Err(finding) = result {
            return (Err(Case { test, seed, finding }), ran);
        }
    }
    (Ok(()), ran)
}

#[derive(Clone, Copy)]
struct Location {
    test: Test,
    seed: u64,
}

impl fmt::Display for Location {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(out, "fuzzy case: {} seed {:016x}", self.test, self.seed)
    }
}

std::thread_local! {
    static ACTIVE: Cell<Option<Location>> = const { Cell::new(None) };
}

// The location belongs to the check, and is restored even if it unwinds.
struct Active(Option<Location>);

impl Active {
    fn enter(location: Location) -> Self {
        Self(ACTIVE.replace(Some(location)))
    }
}

impl Drop for Active {
    fn drop(&mut self) {
        ACTIVE.set(self.0);
    }
}

fn install_hook() {
    static HOOK: Once = Once::new();
    HOOK.call_once(|| {
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| dispatch_hook(stderr, || previous(info))));
    });
}

pub(crate) fn dispatch_hook<R, H>(report: R, previous: H)
where
    R: FnOnce(fmt::Arguments<'_>),
    H: FnOnce(),
{
    if let Ok(Some(location)) = ACTIVE.try_with(Cell::get) {
        report(format_args!("{location}"));
    }
    previous();
}

pub(super) fn stderr(line: fmt::Arguments<'_>) {
    // A failed stderr write must not stop the hook it chains to.
    let _reported = writeln!(std::io::stderr().lock(), "{line}");
}
