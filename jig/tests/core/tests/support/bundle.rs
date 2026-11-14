//! Cases share a process, each named with its replay seed before it runs.
#[derive(Default)]
pub struct Bundle {}
impl Bundle {
    #[expect(clippy::unused_self, reason = "case keeps the bundle interface shared by the scenario families")]
    pub fn case(&mut self, name: &str, seed: u64, run: fn()) {
        eprintln!("{name} (seed {seed})");
        run();
    }
}

macro_rules! case {
    ($bundle:expr, $seed:expr, $run:path) => {
        $bundle.case(stringify!($run), $seed, $run);
    };
}
pub(crate) use case;
