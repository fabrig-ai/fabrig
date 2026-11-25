use skein_io::{
    self as io,
    kernel::{Done, Fd},
};
use skein_lib::{Env, Queue, Wall, stream};
use temper_worker_domain::tooling::{Event, Exit};
use temper_worker_service::{
    Worker, WorkerRuntime,
    testing::{JOB, event, jobs_deadline, jobs_empty, jobs_fire, launch_args},
};

mod fixtures {
    use temper_worker_service::{settings, testing::fixtures};

    pub use fixtures::{pinned_agent, worker_files};

    pub fn worker_settings() -> settings::Worker {
        fixtures::worker_settings(jig_protocol_config_fixtures::small())
    }

    pub fn worker_agent() -> settings::Agent {
        fixtures::worker_agent(smith_agent_world::LIMITS.run)
    }

    pub use fixtures::worker_limits;
}

#[test]
#[cfg(unix)]
#[expect(
    clippy::disallowed_types,
    clippy::disallowed_macros,
    reason = "the real-loop test collects output and formats its expected environment as ordinary Rust"
)]
#[expect(
    clippy::wildcard_enum_match_arm,
    reason = "the real-loop observer records only spawn and stdout; every event still reaches the worker"
)]
fn a_real_job_has_only_its_environment_and_releases_its_group() {
    use skein_shell::{Clock, Config, Kernel, Wait};
    use skein_world::{Deadline, Expired};
    use std::os::fd::AsRawFd;
    use std::os::unix::ffi::OsStrExt;
    let scratch = skein_scratch::Scratch::new("worker-job");
    let directory = scratch.path().join("workspaces/7");
    std::fs::create_dir_all(directory.join("home")).unwrap();
    std::os::unix::fs::symlink("/usr/bin/env", directory.join("home/tool")).unwrap();
    let root_fd = std::fs::File::open(scratch.path()).unwrap();
    let settings = fixtures::worker_settings();
    let mut worker = Worker::new(
        &settings,
        &fixtures::worker_agent(),
        &fixtures::worker_limits(),
        &fixtures::worker_files(),
        WorkerRuntime {
            root: Fd::new(root_fd.as_raw_fd()),
            path: scratch.path().as_os_str().as_bytes().into(),
            agent: fixtures::pinned_agent(settings.jig.agent.as_ref().expect("checked worker agent")),
        },
        7,
    )
    .unwrap();
    let clock = Clock::new();
    let mut env = Env {
        now: clock.now().now,
        wall: Wall::EPOCH,
        limits: settings.jig.limits.as_ref().expect("checked worker limits").io,
    };
    let operations = io::operations(&env.limits).unwrap();
    let mut kernel = Kernel::open(Config { operations }).unwrap();
    let mut io = io::Io::new(&env.limits);
    let mut completions = Queue::uncapped();
    let mut submissions = Queue::with_capacity(operations);
    let mut told = Queue::uncapped();
    let (owner, spawn) = launch_args(
        &mut worker,
        Box::new([b"workspaces/7/home/tool".as_slice().into()]),
        env.now.saturating_add(skein_lib::Duration::from_secs(2)),
    );
    worker.io().push(io::Request::Spawn { owner, spawn });
    let deadline = Deadline::arm("the worker's environment child", skein_lib::Duration::from_secs(2));
    let mut starved = None;
    let mut group = None;
    let mut stdout = None;
    let mut variables = Vec::new();
    loop {
        env.now = clock.now().now;
        if starved.is_none() && deadline.passed() {
            match deadline.expired() {
                Expired::Starved(expired) => starved = Some(expired),
                Expired::Failed(figures) => panic!("the environment child settles: {figures}"),
            }
        }
        kernel.reap(&mut completions);
        for complete in &completions {
            if let Ok(Done::Spawned { pidfd, .. }) = complete.result {
                let pid = skein_shell::keeper::child_id(pidfd).unwrap();
                assert!(skein_shell::keeper::group_live(pid).unwrap(), "the child leads its own group");
                group = Some(pid);
            }
        }
        io::stage::up(&mut io, &env, &mut completions, &mut told, &mut submissions, env.now);
        while let Some(event) = told.pop() {
            match &event {
                io::Event::Spawned { pipes, .. } => stdout = pipes.first().copied(),
                io::Event::Stream { owner, up: stream::Up::Bytes(bytes) } if Some(*owner) == stdout => {
                    variables.extend_from_slice(bytes);
                }
                _ => {}
            }
            worker.up(env.now, event);
        }
        jobs_fire(&mut worker, env.now, false);
        io::stage::down(&mut io, &env, worker.io(), &mut submissions);
        io.reclaim();
        if jobs_empty(&worker) && io.is_empty() && kernel.in_flight() == 0 && submissions.is_empty() {
            break;
        }
        let until = jobs_deadline(&worker).unwrap_or(deadline.until()).min(deadline.until());
        let busy = io.is_ready() || !worker.io().is_empty() || !completions.is_empty();
        kernel.submit(&mut submissions, if busy { Wait::No } else { Wait::Until(until) });
    }
    assert_eq!(event(&mut worker), Event::Spawned { owner: JOB });
    assert_eq!(event(&mut worker), Event::Exited { owner: JOB, exit: Exit::Code(0), tail: Box::new([]) });
    assert_eq!(event(&mut worker), Event::Gone { owner: JOB });
    let directory = directory.to_str().unwrap();
    assert_eq!(
        String::from_utf8(variables).unwrap(),
        format!("PATH=/usr/bin:/bin\nHOME={directory}\nLANG=C.UTF-8\nTEMPER_JOB={directory}\n")
    );
    assert!(
        !skein_shell::keeper::group_live(group.expect("a child was spawned")).unwrap(),
        "Gone follows the group's release"
    );
    if let Some(starved) = starved {
        starved.pass();
    }
}
