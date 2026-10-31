//! Cases share a process while keeping independent failures and replay seeds.
#[derive(Default)]
pub struct Bundle {
    failed: Vec<String>,
}
impl Bundle {
    pub fn case(&mut self, name: &str, seed: u64, run: fn()) {
        if let Err(panic) = std::panic::catch_unwind(run) {
            let why = panic
                .downcast_ref::<String>()
                .map(String::as_str)
                .or_else(|| panic.downcast_ref::<&str>().copied())
                .unwrap_or("non-text panic");
            self.failed.push(format!("{name} (seed {seed}): {why}"));
        }
    }
    pub fn finish(self) -> Result<(), String> {
        if self.failed.is_empty() { Ok(()) } else { Err(self.failed.join("\n")) }
    }
}

#[test]
fn a_bundle_reports_both_failing_cases_and_their_seeds() {
    fn first() {
        panic!("first failure");
    }
    fn second() {
        panic!("second failure");
    }
    let mut bundle = Bundle::default();
    bundle.case("first", 17, first);
    bundle.case("second", 29, second);
    assert_eq!(bundle.finish(), Err("first (seed 17): first failure\nsecond (seed 29): second failure".into()));
}

macro_rules! case {
    ($bundle:expr, $seed:expr, $run:path) => {
        $bundle.case(stringify!($run), $seed, $run);
    };
}
pub(crate) use case;
