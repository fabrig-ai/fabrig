//! Largest values at the ceilings, or at the largest clamp under the cap.
//! The drift tests pin ceiling bounds in the families' bounds files.

use super::codec_memory::family;
use skein_codec::Record;

const CAP: u64 = 64 * 1024 * 1024;

#[test]
fn largest_roots_stay_within_decoded_bounds() {
    family!(
        CAP,
        smith_channel::CHANNEL_SCHEMA,
        smith_channel::v14,
        roots [
            ListAsk,
            ListAnswer,
            Start,
            HostAnswer,
            Acknowledge,
            GrantRefresh,
            Cancel,
            Drain,
            Renewals,
            Admitted,
            MessageRefused,
            Call,
            Withdraw,
            Turn,
            Part,
            PartKept,
            KeptPart,
            Waiting,
            Long,
            LongDone,
            Unauthorized,
            Limited,
            Usage,
            AccountRefused,
            Fact,
            Guides,
            Answer,
            Live,
            Signalled,
        ]
    );
    family!(CAP, smith_charter::SCHEMA, smith_charter::v5, roots [Charter, RunResult]);
    family!(CAP, smith_local_protocol::SCHEMA, smith_local_protocol::local_store_v1, roots [State, Turn, Delivery, Layouts]);
    family!(CAP, smith_transcript::SCHEMA, smith_transcript::v8, roots [Hit, Turn]);
}
