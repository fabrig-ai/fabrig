use super::*;
#[test]
fn public_names_are_kept_as_a_key_without_an_expiry() {
    let value = kit::Kept { digest: skein_lib::Secret::new(Box::new([17; 32])), expires: Wall::from_nanos(u64::MAX) };
    let (_, request) =
        secret_request(kit::SecretsRequest::Keep { owner: kit::SecretOwner::PublicNames, value: value.clone() });
    let jig_protocol_store::Secrets::Keep { owner, number, value: stored } = request else {
        panic!("keep public key");
    };
    assert_eq!(owner, jig_protocol_store::SecretOwner::PublicNames);
    assert_eq!(number, 0);
    let jig_protocol_store::SecretValue::Key(_) = &stored else {
        panic!("a key without expiry");
    };
    let restored = kept(stored).expect("key record");
    assert_eq!(restored.digest, value.digest);
    let (_, request) = secret_request(kit::SecretsRequest::Get { owner: kit::SecretOwner::PublicNames });
    let jig_protocol_store::Secrets::Get { owner: jig_protocol_store::SecretOwner::PublicNames, number: 0 } = request
    else {
        panic!("get the public names row");
    };
}

#[test]
fn endings_leave_in_commit_order_behind_their_own_commit() {
    let mut endings = Endings::new(2);
    for (commit, sign_in) in [(58, 43), (58, 44), (57, 41), (57, 42)] {
        endings.ended(commit, sign_in);
    }
    assert_eq!(endings.next(), None, "each ending waits for its own durable commit");
    endings.committed(57);
    for sign_in in [41, 42] {
        assert_eq!(endings.next(), Some(sign_in), "endings leave in commit order");
    }
    assert_eq!(endings.next(), None, "58's endings stay behind their own commit");
    endings.committed(58);
    for sign_in in [43, 44] {
        assert_eq!(endings.next(), Some(sign_in), "endings leave in commit order");
    }
    assert_eq!(endings.next(), None, "each ending leaves once");
}
