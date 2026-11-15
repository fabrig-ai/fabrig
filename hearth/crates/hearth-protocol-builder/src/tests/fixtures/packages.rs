use crate::render::parts::packages as target;

#[must_use]
pub fn package_records() -> [target::Packages; 2] {
    use skein_lib::bytes::copy_of;
    [target::Variant::Apt, target::Variant::Container].map(|variant| target::Packages {
        initramfs_file: Some(super::ABC),
        sources: target::Sources {
            suite: copy_of(b"forky"),
            architecture: copy_of(b"arm64"),
            components: Box::new([
                target::Component::Main,
                target::Component::Contrib,
                target::Component::NonFree,
                target::Component::NonFreeFirmware,
            ]),
            archive: copy_of(b"20261001T030405Z"),
            updates: true,
            security: Some(copy_of(b"20261101T030405Z")),
        },
        request: Box::new([copy_of(b"curl"), copy_of(b"tzdata")]),
        variant,
        preseeds: Box::new(
            [
                target::PreseedKind::Select,
                target::PreseedKind::Multiselect,
                target::PreseedKind::String,
                target::PreseedKind::Boolean,
            ]
            .map(|kind| target::Preseed {
                question: copy_of(b"owner/question"),
                owner: copy_of(b"owner"),
                kind,
                value: copy_of(b"value"),
            }),
        ),
        initramfs: match variant {
            target::Variant::Apt => {
                Some(target::Initramfs { modules: Box::new([copy_of(b"nvme")]), microcode: target::Microcode::Generic })
            }
            target::Variant::Container => None,
        },
        apt: target::Apt { install_recommends: true },
        seeds: target::Seeds {
            users: Box::new([target::SeedUser { name: copy_of(b"mail"), uid: 120, gid: Some(121) }]),
            groups: Box::new([target::SeedGroup { name: copy_of(b"mail"), gid: 121 }]),
        },
    })
}
