//! Activity's own facts; objects are shown by their shared cards (objects.md, 10).
use crate::JigBindings;
use crate::words::Words;
use crate::words::phrases::jig::JigPhrase;
use skein_ui::Builder;
use skein_ui::phrases::Hole;

use crate::Borrowed;
use jig_client::contract::DecodedActivityLine;
pub(super) fn activity(
    b: &mut Builder,
    client: &Borrowed<'_>,
    value: &DecodedActivityLine,
    words: &Words,
    bindings: &mut JigBindings,
) {
    let conversation = client.conversation();

    let DecodedActivityLine { source, decoded } = value;
    crate::tree::fragments::activity::render(b, source, words);
    if let Some(archive) = decoded {
        words.text(b, JigPhrase::ArchiveCumulativeSpend, &[Hole::Amount(archive.cumulative)]);
        words.text(b, JigPhrase::ArchiveAcceptedAt, &[Hole::Wall(words.adjusted_wall(archive.at))]);
        match archive.read {
            Some(read) => words.text(b, JigPhrase::ArchiveReadThrough, &[Hole::Number(read)]),
            None => words.text(b, JigPhrase::ArchiveReadAbsent, &[]),
        }
    }
    if let Some(archive) = decoded {
        super::turn::turn(b, client, archive.name, &archive.turn, words, bindings);
    }
    if let Some((message, _)) = value.word_message() {
        super::word_state(b, conversation.words.state(message), words);
    }
}
