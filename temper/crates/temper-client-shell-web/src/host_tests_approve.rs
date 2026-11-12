//! The host keeps the approval place and prints its code as shown (client/web/README.md, 2.3).
use super::*;
use jig_client_protocol_wire::v20 as w;
use skein_lib::Reader;
#[test]
fn the_approval_address_prints_its_code_and_reads_once() {
    for (path, printed, reads) in [
        (b"/terminal?code=wdjb-mjht".as_slice(), Some(b"/terminal?code=WDJB-MJHT".as_slice()), 1),
        (b"/terminal?code=AEIO", Some(b"/terminal".as_slice()), 0),
        (b"/terminal?code=WDJB-MJHT", None, 1),
    ] {
        let mut host = super::tab::linked();
        host.show = None;
        let address = host.address.clone();
        host.went(path);
        assert_eq!(host.address, address, "the place keeps the pages' address");
        assert_eq!(host.show, printed.map(|path| (Box::from(path), true)), "{path:?}");
        let mut count = 0;
        for request in host.requests() {
            if let Asked::Exchange { call, .. } = request
                && call.path.as_ref() == b"/jig/v1/approval/read"
            {
                let read = w::ApprovalRead::decode(&w::CEILINGS, &mut Reader::new(&call.body)).unwrap();
                assert_eq!(read.code(), b"WDJBMJHT");
                count += 1;
            }
        }
        assert_eq!(count, reads);
        host.due();
        assert!(!host.requests().iter().any(
            |request| matches!(request, Asked::Exchange { call, .. } if call.path.as_ref() == b"/jig/v1/approval/read")
        ));
    }
}
