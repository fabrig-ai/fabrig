//! Project funding names the ceiling and its remedy (jig/docs/design/domain/authority.md, 6).
use super::{Text, c};

pub(super) fn text(
    project: u32,
    holder: c::CeilingHolder,
    dimension: c::CeilingDimension,
    value: Option<u64>,
    application: &[u8],
    out: &mut Text,
) {
    out.put(b"project fund: ");
    if dimension == c::CeilingDimension::PeriodSpend {
        match holder {
            c::CeilingHolder::Rules => out.put(b"past the rules' period spend"),
            c::CeilingHolder::Policy => out.put(b"past the project's period spend"),
            c::CeilingHolder::Role(role) => {
                out.put(b"below role ");
                out.number(u64::from(role));
                out.put(b"'s period spend");
            }
        }
        scalar(value, out);
        match holder {
            c::CeilingHolder::Rules => {
                out.put(b"; raise the rules first with ");
                out.put(application);
                out.put(b" ctl settings apply");
            }
            c::CeilingHolder::Policy => {
                out.put(b"; fund the project first: ");
                out.put(application);
                out.put(b" ctl project fund ");
                out.number(u64::from(project));
                out.put(b" AMOUNT");
            }
            c::CeilingHolder::Role(role) => {
                out.put(b"; lower the role first: ");
                out.put(application);
                out.put(b" ctl project fund ");
                out.number(u64::from(project));
                out.put(b" AMOUNT --role ");
                out.number(u64::from(role));
            }
        }
        return;
    }
    match holder {
        c::CeilingHolder::Rules => out.put(b"the project's authority is past the rules' "),
        c::CeilingHolder::Policy => out.put(b"a role's authority is past the project's "),
        c::CeilingHolder::Role(role) => {
            out.put(b"the project's authority is below role ");
            out.number(u64::from(role));
            out.put(b"'s ");
        }
    }
    out.put(match dimension {
        c::CeilingDimension::PeriodSpend => b"period spend",
        c::CeilingDimension::Spend => b"spend",
        c::CeilingDimension::Tools => b"tools",
        c::CeilingDimension::Tasks => b"delegation's tasks",
        c::CeilingDimension::Depth => b"depth",
        c::CeilingDimension::Executors => b"executors",
        c::CeilingDimension::Grants => b"grants",
        c::CeilingDimension::Deadline => b"deadline",
        c::CeilingDimension::Notes => b"notes",
    });
    scalar(value, out);
    match holder {
        c::CeilingHolder::Rules => {
            out.put(b"; raise the rules with ");
            out.put(application);
            out.put(b" ctl settings apply, or narrow the project's policy");
        }
        c::CeilingHolder::Policy | c::CeilingHolder::Role(_) => {
            out.put(b"; read the policy with ");
            out.put(application);
            out.put(b" ctl settings show, and narrow the role or raise the project's policy");
        }
    }
}
fn scalar(value: Option<u64>, out: &mut Text) {
    if let Some(value) = value {
        out.put(b", ");
        out.number(value);
    }
}
