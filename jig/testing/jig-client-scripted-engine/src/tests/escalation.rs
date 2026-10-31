use super::*;

#[test]
fn escalation_script_preserves_supplied_identity_scope_and_outer_offer_without_synthetic_terminal() {
    let mut setup = script();
    let mut task = task();
    task.context = None;
    task.acts = list(vec![Offered { effect: None, act: ActKind::Stop, admission: Admission::Allowed }]);
    let value = ReadValue::Escalation(Box::new(Escalation {
        task: Box::new(task),
        acts: list(vec![Offered { effect: None, act: ActKind::Release, admission: Admission::Allowed }]),
    }));
    setup.truth.objects.insert(ObjectKey::Escalation(8), value.clone());
    setup.truth.object_projects.insert(ObjectKey::Escalation(8), 1);
    let engine = Engine::new(setup.clone(), 1).unwrap();
    assert_eq!(engine.truth.objects[&ObjectKey::Escalation(8)], value);
    // The embedded project is authoritative even if an obsolete fallback differs.
    setup.truth.object_projects.insert(ObjectKey::Escalation(8), 2);
    assert!(Engine::new(setup.clone(), 1).is_ok());
    assert_eq!(
        crate::Change::Object { project: 2, key: ObjectKey::Escalation(8), value: value.clone() }
            .validate(&Bounds::default()),
        Err(Invalid::Identity)
    );
    setup.truth.objects.remove(&ObjectKey::Escalation(8));
    setup.truth.objects.insert(ObjectKey::Escalation(99), value);
    assert_eq!(Engine::new(setup, 1).err(), Some(Invalid::Identity));
}
