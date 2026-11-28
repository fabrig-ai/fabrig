use core::convert::Infallible;

use jig_protocol_worker_wire as wire;
use jig_protocol_worker_wire::Record;
use skein_lib::{Reader, Rng, Writer};
use skein_world::{
    sweep::{Case, cases},
    this_test,
};

macro_rules! check {
    ($bytes:expr, $record:ident) => {
        if let Ok(value) = wire::$record::decode(&wire::CEILINGS, &mut Reader::new($bytes)) {
            let mut writer = Writer::new(skein_lib::length::usize_of(
                value.measure(&jig_protocol_worker_wire::CEILINGS).expect("record within limits"),
            ));
            value.write(&jig_protocol_worker_wire::CEILINGS, &mut writer).expect("measured room");
            assert_eq!(writer.finish().as_ref(), $bytes);
        }
    };
}

fn random_bytes(rng: &mut Rng) -> Vec<u8> {
    let len = usize::try_from(rng.below(256)).expect("small length");
    (0..len).map(|_| u8::try_from(rng.below(256)).expect("a byte")).collect()
}

#[test]
fn fuzzy_up_decoders_never_panic_and_reencode_what_they_read() -> Result<(), Case<Infallible>> {
    cases(this_test!("fuzzy_up_decoders_never_panic_and_reencode_what_they_read"), [0x22_0001], |seed| {
        let mut rng = Rng::new(seed);
        for _ in 0..20_000 {
            let bytes = random_bytes(&mut rng);
            check!(&bytes, Credential);
            check!(&bytes, Hello);
            check!(&bytes, Turn);
            check!(&bytes, Call);
            check!(&bytes, Answer);
            check!(&bytes, JobAnswer);
            check!(&bytes, Fact);
            check!(&bytes, Piece);
            check!(&bytes, InputCall);
            check!(&bytes, LiveAsk);
            check!(&bytes, Unauthorized);
            check!(&bytes, Limited);
            check!(&bytes, AccountRefused);
            check!(&bytes, Logins);
            check!(&bytes, SignedOut);
        }
        Ok(())
    })
}

#[test]
fn fuzzy_down_decoders_never_panic_and_reencode_what_they_read() -> Result<(), Case<Infallible>> {
    cases(this_test!("fuzzy_down_decoders_never_panic_and_reencode_what_they_read"), [0x22_0002], |seed| {
        let mut rng = Rng::new(seed);
        for _ in 0..20_000 {
            let bytes = random_bytes(&mut rng);
            check!(&bytes, Assignment);
            check!(&bytes, Job);
            check!(&bytes, Inbound);
            check!(&bytes, Relayed);
            check!(&bytes, TurnKept);
            check!(&bytes, TurnBusy);
            check!(&bytes, Acknowledge);
            check!(&bytes, Cancel);
            check!(&bytes, Granted);
            check!(&bytes, Sources);
            check!(&bytes, Check);
            check!(&bytes, SignOut);
        }
        Ok(())
    })
}
