use super::fixtures;
fn replaced(input: &[u8], from: &[u8], to: &[u8]) -> Box<[u8]> {
    let at = skein_lib::bytes::find(input, from).expect("fixture member");
    let end = at.checked_add(from.len()).unwrap();
    let length = input.len().checked_sub(from.len()).unwrap().checked_add(to.len()).unwrap();
    let mut out = skein_lib::Writer::new(length);
    out.put(input.get(..at).unwrap()).unwrap();
    out.put(to).unwrap();
    out.put(input.get(end..).unwrap()).unwrap();
    out.finish()
}

#[test]
fn configured_accounts_are_refused_at_their_path() {
    let document = replaced(&fixtures::document(), br#""llm": {"#, br#""llm": {"accounts": [],"#);
    let found =
        crate::named(&fixtures::engine_part(&document), &jig_protocol_config::Directories::working()).unwrap_err();
    assert_eq!(&*crate::startup::lines(&found), b"engine.llm.accounts: unknown member\n");
}

#[test]
fn an_endpoint_account_is_refused_at_its_path() {
    let document = replaced(&fixtures::document(), br#""dialect": 3,"#, br#""dialect": 3,"account": 1,"#);
    let found =
        crate::named(&fixtures::engine_part(&document), &jig_protocol_config::Directories::working()).unwrap_err();
    assert_eq!(&*crate::startup::lines(&found), b"engine.llm.endpoints[0].account: unknown member\n");
}
