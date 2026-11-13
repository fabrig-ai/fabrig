//! Agent executable dispatch (temper/docs/design/playground.md, 3).
use super::{Process, agent, agent_roots, programs, with};
use crate::{stories, story};
use skein_io::kernel::{Fd, Spawn};
use skein_lib::Wall;
use skein_shell::Host;
use skein_sim_world::Inherited;

#[test]
fn agent_spawns_select_the_run_or_list_process() {
    let parameters = story::parameters(&story::world(stories::A.start(7, Wall::EPOCH)));
    with(parameters, || {
        let inherited = Inherited {
            pipes: vec![(0, Fd::new(10)), (1, Fd::new(11)), (2, Fd::new(12))],
            signal: Fd::new(13),
            roots: vec![],
            appends: vec![],
        };
        for (words, list) in [
            (vec!["internal", "agent", "/agent.json"], false),
            (vec!["internal", "agent-list", "/agent.json"], true),
            (vec!["--list", "internal", "agent-list", "/agent.json"], true),
        ] {
            let spawn = Spawn {
                program: crate::documents::PINNED_AGENT_PROGRAM.into(),
                args: words.iter().map(|word| Box::from(word.as_bytes())).collect(),
                env: Box::new([]),
                root: Fd::new(9),
                dir: b".".as_slice().into(),
                pipes: Box::new([]),
            };
            let process = agent::<()>(&spawn, &inherited);
            assert_eq!(matches!(process, Process::List(_)), list, "{words:?}");
            assert_eq!(agent_roots::<()>(&spawn).is_empty(), list, "a list takes no workspace: {words:?}");
            let registered = programs::<()>();
            let (program, _) = registered.iter().find(|(program, _)| program.program == spawn.program).unwrap();
            assert!(program.operations >= process.operations(), "registered operations cover {words:?}");
        }
    });
}
