use super::*;

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
