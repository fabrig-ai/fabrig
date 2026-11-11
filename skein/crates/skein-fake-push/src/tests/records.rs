use super::*;
#[test]
fn sealed_requests_record_metadata_scripts_record_nothing_and_full_refuses() {
    let (mut service, request) = fixture();
    let taken = service.take(request.clone(), WALL);
    assert_eq!(taken.status, 201);
    let record = taken.received.expect("accepted record");
    assert_eq!(record.target.as_ref(), b"/push/alice");
    assert_eq!(record.topic.as_deref(), Some(b"updates".as_slice()));
    assert_eq!(record.ttl, 60);
    assert_eq!(record.urgency.as_ref(), b"high");
    assert_eq!(record.length, 108);
    assert_eq!(record.at, WALL);
    assert_eq!(service.received(), &[record]);
    for (script, status) in
        [(Script::Gone, 410), (Script::Later { seconds: 37 }, 429), (Script::Fail { status: 503 }, 503)]
    {
        service.script(request.target.clone(), script);
        let taken = service.take(request.clone(), WALL);
        assert_eq!(taken.status, status);
        assert!(taken.received.is_none());
        if status == 429 {
            assert_eq!(taken.headers[0].name.as_ref(), b"Retry-After");
            assert_eq!(taken.headers[0].value.as_ref(), b"37");
        }
        assert_eq!(service.received().len(), 1);
    }
    service.script(request.target.clone(), Script::Accept);
    for _ in 1..RECEIVED {
        assert_eq!(service.take(request.clone(), WALL).status, 201);
    }
    assert!(service.full());
    assert_eq!(service.received().len(), usize::try_from(RECEIVED).expect("bound"));
    assert_eq!(service.take(request, WALL).status, 507);
}
