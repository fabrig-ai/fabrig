use crate::{
    records::diff as records,
    render::{diff, parts::meta},
    translate::changes,
};
pub(super) fn back(change: records::MetaChange) -> meta::Change {
    meta::Change {
        base: name_pair_back(change.base),
        member: member_pair_back(change.member),
        environment: name_pair_back(change.environment),
        placement: placement_pair_back(change.placement),
    }
}
fn name_pair_back(pair: Option<records::NamePair>) -> Option<meta::NamePair> {
    let pair = pair?;
    Some(meta::NamePair { before: pair.before, after: pair.after })
}
fn member_pair_back(pair: Option<records::MemberPair>) -> Option<meta::MemberPair> {
    let pair = pair?;
    Some(meta::MemberPair { before: pair.before, after: pair.after })
}
fn placement_back(value: crate::records::parts::meta::Placement) -> meta::Placement {
    meta::Placement {
        kind: super::client_kind_back(value.kind),
        firmware: super::optional_client_firmware_back(value.firmware),
        hardware: value.hardware,
        on: value.on,
    }
}
#[test]
fn each_meta_change_translates_and_back() {
    for (row, input) in crate::tests::fixtures::meta_changes().into_iter().enumerate() {
        let records::PartChange::Meta(change) = changes::part_change(diff::PartChange::Meta(input.clone())) else {
            panic!("row {row}: meta change");
        };
        assert_eq!(back(change), input, "row {row}: meta change");
    }
}

fn placement_pair_back(pair: Option<records::PlacementPair>) -> Option<meta::PlacementPair> {
    let pair = pair?;
    Some(meta::PlacementPair { before: placement_back(pair.before), after: placement_back(pair.after) })
}
