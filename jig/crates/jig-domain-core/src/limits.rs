//! Limit validity and per-value policy and settings carriers for the core
//! (`domain/engine/README.md`, section 5.6, and `domain/engine/briefs.md`, section 2, and `domain/engine/settings.md`, section 9; programming-model.md, 6.3).

pub use crate::below::limits::*;

#[cfg(test)]
mod tests {
    use super::{CORE_SECTIONS, brief_holds_documents, charter_owned_bytes, policy_owned_bytes};
    use crate::Document;
    use alloc::boxed::Box;
    use core::mem::size_of;
    use jig_domain_authority::Name;

    #[test]
    fn brief_holds_documents_at_the_section_boundary() {
        for documents in [0, 1, 2, 8] {
            let sections = CORE_SECTIONS + documents;
            assert!(brief_holds_documents(sections, documents));
            assert!(!brief_holds_documents(sections - 1, documents));
        }
        assert!(!brief_holds_documents(u32::MAX, u32::MAX));
    }

    #[test]
    fn charter_owned_bytes_counts_each_document_and_its_name_path_and_heading() {
        let config = crate::tests::config(1);
        let mut charter = config.defaults.charters.first().expect("fixture charter").clone();
        let policy = policy_owned_bytes(&charter.policy).expect("fixture policy bytes");
        assert_eq!(charter_owned_bytes(&charter), Some(policy));

        let first = Document {
            connector: 0,
            name: Name { segments: Box::new([Box::from(b"one".as_slice()), Box::from(b"two".as_slice())]) },
            path: Box::from(b"rules.md".as_slice()),
            heading: None,
        };
        let first_bytes = u64::try_from(size_of::<Document>() + 2 * size_of::<Box<[u8]>>() + 3 + 3 + 8)
            .expect("first document bytes");
        charter.documents = Box::new([first.clone()]);
        assert_eq!(charter_owned_bytes(&charter), Some(policy + first_bytes));

        let second = Document {
            connector: 1,
            name: Name { segments: Box::new([Box::from(b"other".as_slice())]) },
            path: Box::from(b"guide.md".as_slice()),
            heading: Some(Box::from(b"Work".as_slice())),
        };
        let second_bytes =
            u64::try_from(size_of::<Document>() + size_of::<Box<[u8]>>() + 5 + 8 + 4).expect("second document bytes");
        charter.documents = Box::new([first, second]);
        assert_eq!(charter_owned_bytes(&charter), Some(policy + first_bytes + second_bytes));
    }
}
