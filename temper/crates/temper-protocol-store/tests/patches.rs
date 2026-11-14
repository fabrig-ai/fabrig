//! The owner's patch rules (protocol/baseline.md, 7).
mod patches {
    pub use temper_protocol_store::patches::written;

    #[cfg(test)]
    mod tests {
        use std::path::PathBuf;

        fn owner() -> PathBuf {
            skein_scratch::paths::package()
        }

        #[test]
        fn current() {
            assert_eq!(skein_codegen::patch::current(&owner()), Ok(()), "run skein-codegen patch for this owner");
        }

        #[test]
        fn p0001_goldens() {
            assert_eq!(skein_codegen::patch::goldens(&owner(), 1, super::written), Ok(()), "patch 0001 goldens");
        }

        #[test]
        fn p0002_goldens() {
            assert_eq!(skein_codegen::patch::goldens(&owner(), 2, super::written), Ok(()), "patch 0002 goldens");
        }

        #[test]
        fn p0003_goldens() {
            assert_eq!(skein_codegen::patch::goldens(&owner(), 3, super::written), Ok(()), "patch 0003 goldens");
        }

        #[test]
        fn p0004_goldens() {
            assert_eq!(skein_codegen::patch::goldens(&owner(), 4, super::written), Ok(()), "patch 0004 goldens");
        }

        #[test]
        fn p0005_goldens() {
            assert_eq!(skein_codegen::patch::goldens(&owner(), 5, super::written), Ok(()), "patch 0005 goldens");
        }

        #[test]
        fn p0006_goldens() {
            assert_eq!(skein_codegen::patch::goldens(&owner(), 6, super::written), Ok(()), "patch 0006 goldens");
        }

        #[test]
        fn p0007_goldens() {
            assert_eq!(skein_codegen::patch::goldens(&owner(), 7, super::written), Ok(()), "patch 0007 goldens");
        }

        #[test]
        fn p0008_goldens() {
            assert_eq!(skein_codegen::patch::goldens(&owner(), 8, super::written), Ok(()), "patch 0008 goldens");
        }

        #[test]
        fn p0009_goldens() {
            assert_eq!(skein_codegen::patch::goldens(&owner(), 9, super::written), Ok(()), "patch 0009 goldens");
        }

        #[test]
        fn p0010_goldens() {
            assert_eq!(skein_codegen::patch::goldens(&owner(), 10, super::written), Ok(()), "patch 0010 goldens");
        }

        #[test]
        fn p0011_goldens() {
            assert_eq!(skein_codegen::patch::goldens(&owner(), 11, super::written), Ok(()), "patch 0011 goldens");
        }

        #[test]
        fn p0012_goldens() {
            assert_eq!(skein_codegen::patch::goldens(&owner(), 12, super::written), Ok(()), "patch 0012 goldens");
        }

        #[test]
        fn p0013_goldens() {
            assert_eq!(skein_codegen::patch::goldens(&owner(), 13, super::written), Ok(()), "patch 0013 goldens");
        }

        #[test]
        fn p0014_goldens() {
            assert_eq!(skein_codegen::patch::goldens(&owner(), 14, super::written), Ok(()), "patch 0014 goldens");
        }
    }
}
