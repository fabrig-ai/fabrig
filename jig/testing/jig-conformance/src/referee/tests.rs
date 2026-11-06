use super::*;

/// Projectless authority is checked independently (jig/docs/design/domain/authority.md, 6.1).
#[test]
fn a_projectless_task_has_only_stripped_deployment_authority() {
    const BUDGET: u64 = 100;
    let deployment = Scope {
        budget: BUDGET,
        tools: 1,
        grants: vec![Grant {
            connector: 1,
            kind: 1,
            pattern: Pattern { base: vec![], terminal: b"owned".to_vec(), open: false },
        }],
        notes: 1,
        note_resources: vec![(1, Pattern { base: vec![], terminal: b"owned".to_vec(), open: false })],
        descendants: 1,
        depth: 1,
        ..Scope::default()
    };
    let policy = Policy {
        deployment: deployment.clone(),
        projects: BTreeMap::new(),
        people: BTreeMap::from([((0, 1), deployment.clone())]),
        projections: BTreeMap::new(),
        effect_accepters: BTreeSet::new(),
        requirements: vec![],
        implications: BTreeSet::new(),
        kinds: BTreeMap::new(),
        owned: vec![],
        participating: BTreeSet::new(),
        mechanics: BTreeSet::new(),
        delivery_bound: 20,
        uncertainty_bound: 30,
        story_steps: 10,
    };
    let task = Task {
        number: 1,
        project: 0,
        source: Source::Deployment,
        scope: Scope { budget: BUDGET, ..Scope::default() },
        executor: (0, 1),
        tracked: false,
        budget: BUDGET,
        spent: 0,
        spent_below: 0,
        reserved: 0,
        run_reserved: 0,
        run_spent: 0,
        attempt: 0,
        period: None,
        phase: Phase::Active,
        settings: SettingsRevision::default(),
        spec_revision: 0,
        hold_revision: 0,
        placed: None,
        placed_reason: false,
        dependencies: vec![],
        delegates: vec![],
        pools: vec![],
        inbox: vec![],
        proposal: None,
    };
    let observe = |task| {
        Referee::new(policy.clone()).observe(
            0,
            Observed::Durable {
                number: 1,
                snapshot: Snapshot { tasks: BTreeMap::from([(1, task)]), ..Snapshot::default() },
            },
        )
    };
    observe(task.clone()).expect("stripped deployment task needs no project policy");
    let mut tool = task.clone();
    tool.scope.tools = 1;
    let mut grant = task.clone();
    grant.scope.grants = deployment.grants;
    let mut note = task.clone();
    note.scope.notes = 1;
    let mut resource = task.clone();
    resource.scope.note_resources = deployment.note_resources;
    let mut descendants = task.clone();
    descendants.scope.descendants = 1;
    let mut depth = task.clone();
    depth.scope.depth = 1;
    let mut spend = task.clone();
    spend.scope.budget = BUDGET + 1;
    let mut person = task.clone();
    person.source = Source::Person(1);
    let mut unknown = task;
    unknown.project = 99;
    for (task, reason) in [
        (tool, "a tool"),
        (grant, "a grant"),
        (note, "a note"),
        (resource, "a note resource"),
        (descendants, "descendant delegation"),
        (depth, "depth delegation"),
        (spend, "spend past deployment"),
        (person, "a person source"),
        (unknown, "an unknown project"),
    ] {
        assert_eq!(observe(task).expect_err(reason).promise, Promise::Authority, "{reason}");
    }
}

#[test]
fn procedure_and_projection_keys_are_judged_as_distinct_effects() {
    let resource = Name { connector: 1, path: vec![b"owned".to_vec()] };
    let pattern = Pattern { base: vec![], terminal: b"owned".to_vec(), open: false };
    let grant = Grant { connector: 1, kind: 5, pattern: pattern.clone() };
    let scope = Scope { grants: vec![grant.clone()], executors: BTreeSet::from([(0, 1)]), ..Scope::default() };
    let mut referee = Referee::new(Policy {
        projections: BTreeMap::from([(1, vec![grant])]),
        deployment: scope.clone(),
        projects: BTreeMap::from([(1, scope.clone())]),
        people: BTreeMap::new(),
        effect_accepters: BTreeSet::new(),
        requirements: vec![],
        implications: BTreeSet::new(),
        kinds: BTreeMap::from([((1, 5), Kind { recovery: Recovery::Keyed, price: 0 })]),
        owned: vec![(1, pattern)],
        participating: BTreeSet::new(),
        mechanics: BTreeSet::new(),
        delivery_bound: 20,
        uncertainty_bound: 30,
        story_steps: 10,
    });
    let task = Task {
        number: 1,
        project: 1,
        source: Source::Deployment,
        scope,
        executor: (0, 1),
        tracked: true,
        budget: 0,
        spent: 0,
        spent_below: 0,
        reserved: 0,
        run_reserved: 0,
        run_spent: 0,
        attempt: 0,
        period: None,
        phase: Phase::Active,
        settings: SettingsRevision::default(),
        spec_revision: 0,
        hold_revision: 0,
        placed: None,
        placed_reason: false,
        dependencies: vec![],
        delegates: vec![],
        pools: vec![],
        inbox: vec![],
        proposal: None,
    };
    let keys = [EffectOrigin::Procedure, EffectOrigin::Projection].map(|origin| EffectKey {
        deployment: [31; 16],
        task: 1,
        purpose: 9,
        origin,
    });
    let decisions = keys
        .iter()
        .map(|key| Decision {
            projection: key.origin == EffectOrigin::Projection,
            connector: 1,
            key: *key,
            kind: 5,
            resources: vec![resource.clone()],
            condition: None,
            target: 13,
            judged_state: 13,
            accepted_by: None,
        })
        .collect();
    referee
        .observe(
            0,
            Observed::Durable {
                number: 1,
                snapshot: Snapshot { tasks: BTreeMap::from([(1, task)]), decisions, ..Snapshot::default() },
            },
        )
        .expect("both named decisions are authorized");
    for key in keys {
        let effect = EffectObserved {
            kind: 5,
            key,
            resources: vec![resource.clone()],
            condition: None,
            target: 13,
            before: vec![ValueObserved { state: None, owner: None }],
            applied: true,
            copy: Copy::First,
        };
        referee
            .observe(0, Observed::Effect { connector: 1, effect: effect.clone() })
            .expect("each named origin may apply once");
        assert_eq!(
            referee
                .observe(0, Observed::Effect { connector: 1, effect })
                .expect_err("each origin retains its own once-only verdict")
                .promise,
            Promise::Once
        );
    }
}
