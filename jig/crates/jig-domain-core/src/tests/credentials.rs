use super::*;

fn core_with_account() -> (Core, Limits) {
    let (mut core, limits) = core();
    make_task(&mut core, &limits, 1);
    let mut placed = Queue::uncapped();
    fleet::step(
        &mut core.fleet,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.fleet },
        fleet::Event::Hello {
            worker: 0,
            channel: Token::new(1),
            hello: fleet::Hello {
                bounds: fixtures::bounds(),
                stop_bound: Duration::ZERO,
                slots: 1,
                workstreams: Box::new([]),
                hosting: Box::new([]),
            },
        },
        &mut placed,
    );
    fleet::step(
        &mut core.fleet,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.fleet },
        fleet::Event::Logins { channel: Token::new(1), logins: Box::new([]) },
        &mut placed,
    );
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.accounts };
    let mut out = Queue::uncapped();
    accounts::step(&mut core.accounts, &env, accounts::Event::Want, &mut out);
    while out.pop().is_some() {}
    accounts::step(
        &mut core.accounts,
        &env,
        accounts::Event::Component(accounts::FromComponent::Granted {
            form: accounts::Form::Credential(accounts::Scheme::Bearer),
            account: 1,
            generation: 1,
            valid: Duration::from_secs(60),
        }),
        &mut out,
    );
    while out.pop().is_some() {}
    (core, limits)
}

#[test]
fn a_run_waits_while_a_credential_it_needs_is_unusable() {
    let (mut core, limits) = core_with_account();
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.accounts };
    let mut out = Queue::uncapped();
    assert_eq!(core.workspace_credentials.insert(1, Box::new([7])), Ok(None));
    let context = charters::context(1, 1);
    assert_eq!(core.run_admission(&context, Wall::EPOCH, Box::new([])), RunAdmission::Wait(RunWait::Credential));
    core.due.push(context);
    assert!(!core.due_ready());
    accounts::step(
        &mut core.accounts,
        &env,
        accounts::Event::Credential { number: 7, generation: 1, valid: Duration::from_secs(60) },
        &mut out,
    );
    let context = core.take_due().expect("credential frees the waiting task");
    let RunAdmission::Allow(_) = core.run_admission(&context, Wall::EPOCH, Box::new([])) else {
        panic!("credential permits the current allowance")
    };
}

#[test]
fn a_completed_attempts_credential_does_not_block_preparing_the_next_workspace() {
    let (mut core, limits) = core_with_account();
    let mut proof = RunProof::claimed(1, 7, 7, None, fleet::HostKind::Worker);
    proof.credentials = Box::new([7]);
    assert_eq!(core.proofs.insert(1, proof), Ok(None));
    let context = core.tasks.run_context(1).expect("due task");
    assert!(!core.accounts.credential_usable(7));
    assert_eq!(core.run_admission(&context, Wall::EPOCH, Box::new([])), RunAdmission::Wait(RunWait::Credential));
    let document = task_from_row(&core, &limits, 1, core.tasks.task(1).expect("task"), Wall::EPOCH).unwrap();
    assert_eq!(document.run_wait, Some(TaskRunWait::Credential));

    core.proofs.get_mut(&1).expect("current proof").terminal =
        Some(TerminalRecord { task: 1, attempt: 7, cumulative: 0, end: tasks::End::Parked });
    let RunAdmission::Allow(_) = core.run_admission(&context, Wall::EPOCH, Box::new([])) else {
        panic!("the completed attempt does not constrain the next workspace")
    };
    let document = task_from_row(&core, &limits, 1, core.tasks.task(1).expect("task"), Wall::EPOCH).unwrap();
    assert_eq!(document.run_wait, None);

    assert_eq!(core.workspace_credentials.insert(1, Box::new([7])), Ok(None));
    assert_eq!(core.run_admission(&context, Wall::EPOCH, Box::new([])), RunAdmission::Wait(RunWait::Credential));
    let document = task_from_row(&core, &limits, 1, core.tasks.task(1).expect("task"), Wall::EPOCH).unwrap();
    assert_eq!(document.run_wait, Some(TaskRunWait::Credential));
}
