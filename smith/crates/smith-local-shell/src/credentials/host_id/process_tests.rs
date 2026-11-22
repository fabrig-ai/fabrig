use skein_lib::Time;
use skein_world::Referee;
use smith_agent_process_world::testing::host_id::Judge;

#[test]
fn the_judge_keeps_an_act_finding_until_watch() {
    let mut judge = Judge::default();
    judge.act(Time::ZERO, &mut []);
    assert!(!judge.passed(), "an act finding prevents a pass");
    assert_eq!(judge.watch(Time::ZERO, &[]), Err("binary process".into()));
    assert!(!judge.passed(), "the finding remains held");
    assert_eq!(judge.watch(Time::ZERO, &[]), Err("binary process".into()));
}
