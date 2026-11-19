//! Cases share a process, each named with its replay seed before it runs.
#[derive(Default)]
pub struct Bundle {}
impl Bundle {
    pub fn case(&mut self, name: &str, seed: u64, run: fn()) {
        eprintln!("{name} (seed {seed})");
        run();
    }
}

#[macro_export]
macro_rules! case {
    ($bundle:expr, $seed:expr, $run:path) => {
        $bundle.case(stringify!($run), $seed, $run);
    };
}
pub use crate::case;
