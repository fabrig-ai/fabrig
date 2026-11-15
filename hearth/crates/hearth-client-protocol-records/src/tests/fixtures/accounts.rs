use crate::records::Record;
use alloc::boxed::Box;
use hearth_domain_render_vocabulary as render;
use render::parts::{Part, PartRecord, config::*};

pub fn accounts_spec() -> Record {
    let mut spec = super::x1_prod();
    spec.parts = Box::new([Part {
        digest: hearth_lib::encode::Digest([0; 32]),
        record: PartRecord::Config(Config {
            presets: Box::new([]),
            tmpfiles: Box::new([]),
            scripts: Box::new([]),
            tree: Box::new([]),
            groups: Box::new([
                Group { name: b"mail".as_slice().into(), kind: GroupKind::Package },
                Group { name: b"ops".as_slice().into(), kind: ops() },
            ]),
            users: Box::new([
                User { name: b"alice".as_slice().into(), kind: account_user(1000, b"Alice User") },
                User { name: b"postfix".as_slice().into(), kind: UserKind::Package },
            ]),
            members: Box::new([member(b"ops")]),
        }),
    }]);
    Record::Spec(spec)
}
pub fn accounts_change() -> Record {
    use render::diff::Change;
    use render::diff::PartChange;
    use render::diff::Rebuild;
    use render::diff::TargetChange;
    use render::parts::config::Change as ConfigChange;
    use render::parts::config::GroupKeyed;
    use render::parts::config::MemberChange;
    use render::parts::config::UserKeyed;
    Record::Change(Change {
        target: b"x1-prod".as_slice().into(),
        change: TargetChange::Changed {
            rebuild: Rebuild::FromConfig,
            parts: Box::new([PartChange::Config(ConfigChange {
                presets: Box::new([]),
                tmpfiles: Box::new([]),
                scripts: None,
                tree: Box::new([]),
                groups: Box::new([GroupKeyed::Removed { key: b"ops".as_slice().into(), value: ops() }]),
                users: Box::new([UserKeyed::Changed {
                    key: b"alice".as_slice().into(),
                    before: UserKind::Package,
                    after: account_user(1001, b""),
                }]),
                members: Box::new([MemberChange::Removed(member(b"ops")), MemberChange::Added(member(b"video"))]),
            })]),
        },
    })
}

fn account_user(uid: u32, gecos: &[u8]) -> UserKind {
    UserKind::Sysusers {
        uid: Some(uid),
        group: b"alice".as_slice().into(),
        gecos: gecos.into(),
        home: b"/home/alice".as_slice().into(),
        shell: b"/bin/bash".as_slice().into(),
    }
}
fn ops() -> GroupKind {
    GroupKind::Sysusers { gid: Some(1500) }
}
fn member(group: &[u8]) -> Member {
    Member { user: b"alice".as_slice().into(), group: group.into() }
}
