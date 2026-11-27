//! A saved direct window crosses endpoint identity (smith/docs/design/faults.md, 5).
use skein_fake_checkout::Checkout;
use skein_fake_llm_domain::api::{Line, Script};
use skein_lib::Time;
use skein_world::{
    faults::{FaultLedger, Struck},
    streams::{Stream, Streams},
    sweep::{Case, Sweep, cases},
};
use smith_agent_world::{Job, Settings, World, fixture, scripted_charter};
use smith_domain::{
    Transcript, run,
    session::{llm, record},
};

const ROUNDS: u32 = 10_000;
// Seeds 0 and 1 draw a wider and a narrower read window on the resumed endpoint, respectively.
const WINDOW_SEEDS: [u64; 2] = [0, 1];

fn world(settings: &Settings, history: Option<Transcript>, read_bytes: u64) -> World {
    let scripts = Box::new([Script {
        cue: b"@report".as_slice().into(),
        turns: Box::new([
            fixture::calls(
                Box::new([Line::Call {
                    name: b"read".as_slice().into(),
                    arguments: format!(r#"{{"path":".","window":{read_bytes}}}"#).into_bytes().into(),
                }]),
                1,
            ),
            fixture::calls(
                Box::new([Line::Call {
                    name: b"finish".as_slice().into(),
                    arguments: br#"{"report":"done","source":"peer"}"#.as_slice().into(),
                }]),
                1,
            ),
            fixture::says(Box::new([Line::Text { text: b"Keep the saved work.".as_slice().into() }]), 1),
        ]),
    }]);
    let charter = scripted_charter(settings);
    World::with_workspace_scripts_charter(*settings, history, None, Checkout::new(), scripts, charter)
}

#[test]
fn resumed_windows_carry_over() -> Result<(), Case<String>> {
    let mut sweep = Sweep::new("resumed direct windows");
    sweep.configure(smith_faults::WINDOW_ELSEWHERE);
    cases(skein_world::this_test!("resumed_windows_carry_over"), WINDOW_SEEDS, |seed| {
        eprintln!("resumed direct windows seed {seed}");
        let streams = Streams::new(seed);
        let mut workload = streams.rng(Stream::Workload, "runs");
        let mut faults = streams.rng(Stream::Faults, "llm");
        let endpoint = u32::try_from(faults.below(8)).expect("small endpoint number");
        let settings =
            Settings { endpoint, job: Job::Reporting, ..Settings::calm(streams.seed_of(Stream::Scheduling, "world")) };
        let mut ledger = FaultLedger::new();
        let mut first = world(&settings, None, 32 + workload.below(32));
        first.record_fault_runs("run 1", "a");
        first.run(ROUNDS);
        first.judge_faults(&mut ledger)?;
        assert!(matches!(first.answer(), run::Answer::Accepted { .. }), "{:?}", first.answer());
        let opening = first.turns().first().expect("saved first turn");
        let saved = Transcript {
            version: opening.version,
            endpoint: opening.endpoint,
            dialect: opening.dialect,
            turns: first.turns().into(),
        };
        let mut resumed =
            world(&Settings { endpoint: endpoint + 1, resume: true, ..settings }, Some(saved), 32 + workload.below(32));
        resumed.record_fault_runs("run 2", "a");
        ledger.inject(
            smith_faults::WINDOW_ELSEWHERE,
            Time::ZERO,
            Struck { account: Some("a".into()), ..Struck::default() },
        );
        resumed.run(ROUNDS);
        resumed.judge_faults(&mut ledger)?;
        let carried = resumed.turns().first().expect("carried window");
        assert!(matches!(carried.kind, record::Kind::Window(_)), "old window compacted first");
        assert_eq!(carried.endpoint, Some(llm::Endpoint(endpoint + 1)));
        assert_eq!(
            resumed.prompts().first().expect("summary request").choice,
            skein_fake_llm_domain::api::ToolChoice::None
        );
        assert!(matches!(resumed.answer(), run::Answer::Accepted { .. }), "{:?}", resumed.answer());
        assert_eq!(ledger.entries()[0].count(skein_world::faults::Counter::SessionRetries), 0);
        eprintln!("seed {seed} fell {:?}", ledger.fell());
        sweep.ran(&ledger);
        Ok(())
    })?;
    sweep.check_fell().expect("saved window moved");
    Ok(())
}
