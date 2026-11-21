//! Deterministic arbitrary bytes through each top-level family record
//! (protocol/charter.md, section 8; protocol/transcript.md, section 9;
//! protocol/channel.md, section 10).

use skein_lib::{Reader, Rng};
use smith_channel::Record;

macro_rules! check {
    ($bytes:expr, $family:ident, $record:ident) => {
        if let Ok(value) = $family::$record::decode(&$family::CEILINGS, &mut Reader::new($bytes)) {
            let bytes = value.encode(&$family::CEILINGS).expect("decoded value encodes");
            assert_eq!(bytes.as_ref(), $bytes);
        }
    };
}

fn check_all(bytes: &[u8]) {
    check!(bytes, smith_charter, Charter);
    check!(bytes, smith_charter, RunResult);
    check!(bytes, smith_transcript, Turn);
    check!(bytes, smith_channel, Start);
    check!(bytes, smith_channel, Message);
    check!(bytes, smith_channel, MessageRefused);
    check!(bytes, smith_channel, HostAnswer);
    check!(bytes, smith_channel, Acknowledge);
    check!(bytes, smith_channel, GrantRefresh);
    check!(bytes, smith_channel, Cancel);
    check!(bytes, smith_channel, Admitted);
    check!(bytes, smith_channel, Call);
    check!(bytes, smith_channel, Withdraw);
    check!(bytes, smith_channel, Turn);
    check!(bytes, smith_channel, Waiting);
    check!(bytes, smith_channel, Long);
    check!(bytes, smith_channel, LongDone);
    check!(bytes, smith_channel, Unauthorized);
    check!(bytes, smith_channel, Limited);
    check!(bytes, smith_channel, AccountRefused);
    check!(bytes, smith_channel, Fact);
    check!(bytes, smith_channel, Live);
    check!(bytes, smith_channel, Answer);
}

#[test]
fn arbitrary_and_mutated_wires_never_panic_and_round_trip_when_valid() {
    let mut rng = Rng::new(0x51_17_0c_0d_ec_01);
    let mut buffer = [0_u8; 128];
    for _round in 0_u32..10_000 {
        let length = usize::try_from(rng.below(129)).expect("bounded length");
        for byte in buffer.get_mut(..length).expect("within buffer") {
            *byte = u8::try_from(rng.next_u64() & 0xff).expect("low byte");
        }
        check_all(buffer.get(..length).expect("within buffer"));
    }

    for (family, version) in [("charter", 5), ("transcript", 8), ("channel", 7)] {
        for value in crate::support::goldens::all(family, version)
            .into_iter()
            .filter(|value| value.name.starts_with("record_") && value.name.ends_with("_full.bin"))
        {
            let golden = value.bytes;
            check_all(&golden);
            for _round in 0_u32..100 {
                let mut mutated = golden.clone();
                if !mutated.is_empty() {
                    let index = usize::try_from(rng.below(skein_lib::len_u64(mutated.len()))).expect("index fits");
                    *mutated.get_mut(index).expect("chosen byte") =
                        u8::try_from(rng.next_u64() & 0xff).expect("low byte");
                }
                check_all(&mutated);
            }
        }
    }
}
