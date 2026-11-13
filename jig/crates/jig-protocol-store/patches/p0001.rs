//! Retained project delegation grants meet the deployment ceiling (jig/docs/design/protocol/store/records.md, 1).
use super::{
    Context,
    kept::jig_store_people::{TEXTS, p0001::old},
};
use jig_domain_authority::Executor;
use skein_codec::{
    patch::{Build, Halt, Old},
    piece,
    schema::{self, Schema},
    value::{Node, Value},
};
use skein_lib::{List, Reader, Writer};

/// Narrow the project ceiling and each role without granting any new executor.
pub fn stored_policy_value(old: &Old, new: &mut Build, at: Node, cx: &mut Context) -> Result<(), Halt> {
    let schema =
        schema::parse(TEXTS.first().expect("people 14 kept text").as_bytes()).expect("kept people text parses");
    let value = old.field(at, old::stored_policy::VALUE)?;
    let executors = &cx.executors;
    new.open()?;
    authority(old, new, old.field(value, old::policy_value::CEILING)?, executors, &schema)?;
    for field in [old::policy_value::ESCALATION_ROLE, old::policy_value::PERIOD_SPEND, old::policy_value::PROJECTIONS] {
        new.carry(old, old.field(value, field)?)?;
    }
    let roles = old.field(value, old::policy_value::ROLES)?;
    let mut items = old.items(roles)?;
    new.open()?;
    for _ in 0..old.count(roles)? {
        let role = old.next_item(&mut items).ok_or(Halt::Misread(roles))?;
        new.open()?;
        new.carry(old, old.field(role, old::policy_role::NUMBER)?)?;
        authority(old, new, old.field(role, old::policy_role::AUTHORITY)?, executors, &schema)?;
        for field in [old::policy_role::PERIOD_SPEND, old::policy_role::REQUESTS, old::policy_role::DECIDES] {
            new.carry(old, old.field(role, field)?)?;
        }
        new.close()?;
    }
    new.close()?;
    for field in [old::policy_value::REQUIREMENTS, old::policy_value::PERMISSIONS, old::policy_value::TEMPLATES] {
        new.carry(old, old.field(value, field)?)?;
    }
    new.close()
}

fn authority(old: &Old, new: &mut Build, at: Node, executors: &[Executor], schema: &Schema) -> Result<(), Halt> {
    new.open()?;
    for field in [old::authority::TOOLS, old::authority::MODELS, old::authority::GRANTS] {
        new.carry(old, old.field(at, field)?)?;
    }
    let delegation = old.field(at, old::authority::DELEGATION)?;
    let kinds = old.field(delegation, old::delegation::KINDS)?;
    let bytes = old.content(kinds)?;
    let count = if executors.is_empty() {
        0
    } else {
        match piece::count(bytes) {
            Ok(count) => count,
            Err(_) => return Err(Halt::Misread(kinds)),
        }
    };
    let mut reader = Reader::new(bytes);
    let mut kept = List::with_capacity(count);
    let mut length = 0_usize;
    let executor = schema.find(b"Executor").expect("kept people has Executor");
    // An executor has three nodes: its tag, payload record and number.
    let mut piece_value = Value::with_room(3);
    for _ in 0..count {
        let item = piece::counted(&mut reader);
        let root = piece_value.decode(schema, executor, item).or(Err(Halt::Misread(kinds)))?;
        let tag = piece_value.tag(root).or(Err(Halt::Misread(kinds)))?;
        let payload = piece_value.payload(root).or(Err(Halt::Misread(kinds)))?.ok_or(Halt::Misread(kinds))?;
        let field = if tag == old::executor::CHARTER {
            old::executor_charter::VALUE
        } else if tag == old::executor::PROCEDURE {
            old::executor_procedure::VALUE
        } else if tag == old::executor::ROLE {
            old::executor_role::VALUE
        } else if tag == old::executor::JOB {
            old::executor_job::KIND
        } else {
            return Err(Halt::Misread(kinds));
        };
        let number = piece_value.field(payload, field).or(Err(Halt::Misread(kinds)))?;
        let number = piece_value.uint(number).or(Err(Halt::Misread(kinds)))?;
        let number = u32::try_from(number).or(Err(Halt::Misread(kinds)))?;
        let value = if tag == old::executor::CHARTER {
            Executor::Charter(number)
        } else if tag == old::executor::PROCEDURE {
            Executor::Procedure(number)
        } else if tag == old::executor::ROLE {
            Executor::Role(number)
        } else {
            Executor::Job(number)
        };
        let item = piece_value.take().into_exposed();
        if executors.contains(&value) {
            length = length.checked_add(4).ok_or(Halt::Misread(kinds))?;
            length = length.checked_add(item.len()).ok_or(Halt::Misread(kinds))?;
            kept.push(item).expect("no more entries than the old piece");
        }
    }
    let mut writer = Writer::new(length);
    for item in &kept {
        piece::write(&mut writer, item).expect("measured retained pieces");
    }
    new.open()?;
    new.put_content(&writer.finish())?;
    new.carry(old, old.field(delegation, old::delegation::TASKS)?)?;
    new.carry(old, old.field(delegation, old::delegation::DEPTH)?)?;
    new.close()?;
    for field in
        [old::authority::SPEND, old::authority::DEADLINE, old::authority::NOTES, old::authority::NOTE_RESOURCES]
    {
        new.carry(old, old.field(at, field)?)?;
    }
    new.close()
}
