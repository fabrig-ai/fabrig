//! A settled record owns tool and answer bytes; its live address is transient (store.md, 5.1).
use super::*;

fn settled(answer: SettledAnswer) -> SettledCall {
    SettledCall { serial: 0, tool: b"tool".as_slice().into(), answer }
}

#[test]
fn a_settled_calls_owned_bytes_are_tool_and_answer_alone() {
    // Four bytes of tool and six bytes of host body or delivery evidence.
    const OWNED: u64 = 10;
    for answer in [
        SettledAnswer::Host { error: false, body: b"answer".as_slice().into() },
        SettledAnswer::Delivery { outcome: DeliveryOutcome::Delivered, evidence: b"proof!".as_slice().into() },
    ] {
        assert_eq!(settled(answer).owned_bytes(), Some(OWNED));
    }
}

#[test]
fn a_settlement_refusal_preserves_its_opaque_live_reply_address() {
    let (mut core, limits) = make_tree();
    let key = CallKey { task: 1, attempt: 2, completion: 3, position: 4 };
    let to = ReplyTo::new(Token::new(5));
    let Requests::Out(mut out) = crate::conversation::settle(
        &mut core,
        &limits,
        to,
        key,
        b"opaque address".as_slice().into(),
        settled(SettledAnswer::Host { error: false, body: b"answer".as_slice().into() }),
    );
    let Some(Request::Now(refused)) = out.pop() else { panic!("settlement refusal") };
    let Now::SettledCallRefused { to: reply, key: refused_key, name, tool } = *refused else {
        panic!("settlement refusal address")
    };
    assert_eq!(
        (reply.into_token(), refused_key, name.as_ref(), tool.as_ref()),
        (Token::new(5), key, b"opaque address".as_slice(), b"tool".as_slice())
    );
    let Some(Request::Decided) = out.pop() else { panic!("the decision closes") };
    assert!(out.is_empty());
}
