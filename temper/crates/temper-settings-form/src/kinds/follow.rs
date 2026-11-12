//! Kind 12's settings-file form (temper/docs/design/domain/forge/follows.md, 2).
use crate::file::{Decoder, Error, Fields, error, field, optional};
use alloc::boxed::Box;
use skein_json::writer::Encoder;
use skein_lib::List;
use temper_client_protocol_wire::v1 as w;

fn text(d: &Decoder<'_>, fields: &Fields, name: &[u8], required: bool) -> Result<Box<[u8]>, Error> {
    let Some(at) = optional(fields, name) else {
        return if required { Err(error(name, b"missing member")) } else { Ok(Box::new([])) };
    };
    match d.string(at) {
        Ok(value) => Ok(value.into()),
        Err(why) => Err(why.at(name)),
    }
}
fn repository(d: &Decoder<'_>, fields: &Fields) -> Result<w::FollowRepository, Error> {
    let forge = u16::try_from(super::number(d, fields, b"forge")?).or(Err(error(b"forge", b"u16 bound")))?;
    let repository = super::number(d, fields, b"repository")?;
    w::FollowRepository::new(&d.kinds, w::FollowRepositoryParts { forge, repository })
        .or(Err(error(b"", b"wire value bound")))
}
fn source(d: &Decoder<'_>, at: u32) -> Result<w::FollowSource, Error> {
    let fields = d.object(at, &[b"forge", b"repository", b"branch", b"paths"])?;
    let mut paths = List::with_capacity(d.kinds.follow_source_paths.min(w::CEILINGS.follow_source_paths));
    if let Some(at) = optional(&fields, b"paths") {
        for at in &d.array(at, d.kinds.follow_source_paths.min(w::CEILINGS.follow_source_paths), b"paths")? {
            let path = match d.string(*at) {
                Ok(value) => value.into(),
                Err(why) => return Err(why.at(b"paths")),
            };
            paths.push(path).expect("counted paths");
        }
    }
    w::FollowSource::new(
        &d.kinds,
        w::FollowSourceParts {
            repository: repository(d, &fields)?,
            branch: text(d, &fields, b"branch", false)?,
            paths,
        },
    )
    .or(Err(error(b"", b"wire value bound")))
}
fn target(d: &Decoder<'_>, at: u32) -> Result<w::FollowTarget, Error> {
    let fields = d.object(at, &[b"forge", b"repository", b"branch"])?;
    w::FollowTarget::new(
        &d.kinds,
        w::FollowTargetParts { repository: repository(d, &fields)?, branch: text(d, &fields, b"branch", false)? },
    )
    .or(Err(error(b"", b"wire value bound")))
}
fn check(d: &Decoder<'_>, at: u32) -> Result<w::FollowCheck, Error> {
    let fields = d.object(at, &[b"workflow", b"artifact", b"format"])?;
    let format = match d.string(field(&fields, b"format")?)? {
        b"junit" => w::FollowFormat::Junit,
        b"lines" => w::FollowFormat::Lines,
        _ => return Err(error(b"format", b"junit or lines required")),
    };
    w::FollowCheck::new(
        &d.kinds,
        w::FollowCheckParts {
            workflow: text(d, &fields, b"workflow", true)?,
            artifact: text(d, &fields, b"artifact", true)?,
            format,
        },
    )
    .or(Err(error(b"", b"wire value bound")))
}

pub(super) fn read(d: &Decoder<'_>, at: u32) -> Result<w::Follow, Error> {
    let fields = d.object(at, &[b"source", b"target", b"owner", b"start", b"producer", b"check"])?;
    let source = match source(d, field(&fields, b"source")?) {
        Ok(value) => value,
        Err(why) => return Err(why.at(b"source")),
    };
    let target = match target(d, field(&fields, b"target")?) {
        Ok(value) => value,
        Err(why) => return Err(why.at(b"target")),
    };
    let owner = d.object(field(&fields, b"owner")?, &[b"person", b"task"])?;
    if owner.len() != 1 {
        return Err(error(b"owner", b"one variant required"));
    }
    let owner = if let Some(at) = optional(&owner, b"person") {
        w::FollowOwner::Person(
            w::FollowPerson::new(&d.kinds, w::FollowPersonParts { person: d.number(at)? })
                .or(Err(error(b"owner", b"wire value bound")))?,
        )
    } else {
        w::FollowOwner::Task(
            w::FollowOwnerTask::new(&d.kinds, w::FollowOwnerTaskParts { task: d.number(field(&owner, b"task")?)? })
                .or(Err(error(b"owner", b"wire value bound")))?,
        )
    };
    let start = match text(d, &fields, b"start", false)?.as_ref() {
        b"" | b"owner" => w::FollowStart::Owner,
        b"code" => w::FollowStart::Code,
        _ => return Err(error(b"start", b"owner or code required")),
    };
    let producer = text(d, &fields, b"producer", false)?;
    let producer = if producer.is_empty() { None } else { Some(producer) };
    let verification = match check(d, field(&fields, b"check")?) {
        Ok(value) => value,
        Err(why) => return Err(why.at(b"check")),
    };
    w::Follow::new(&d.kinds, w::FollowParts { source, target, owner, start, producer, verification })
        .or(Err(error(b"", b"wire value bound")))
}

fn render_repository(value: &w::FollowRepository, branch: &[u8], out: &mut Encoder) {
    out.key(b"forge");
    out.unsigned(u64::from(value.forge()));
    out.key(b"repository");
    out.unsigned(u64::from(value.repository()));
    out.key(b"branch");
    out.string(branch);
}
pub(super) fn render(value: &w::Follow, out: &mut Encoder) {
    out.object_start();
    out.key(b"source");
    out.object_start();
    render_repository(value.source().repository(), value.source().branch(), out);
    out.key(b"paths");
    out.array_start();
    for path in value.source().paths() {
        out.string(path);
    }
    out.array_end();
    out.object_end();
    out.key(b"target");
    out.object_start();
    render_repository(value.target().repository(), value.target().branch(), out);
    out.object_end();
    out.key(b"owner");
    out.object_start();
    match value.owner() {
        w::FollowOwner::Person(person) => {
            out.key(b"person");
            out.unsigned(person.person());
        }
        w::FollowOwner::Task(task) => {
            out.key(b"task");
            out.unsigned(task.task());
        }
    }
    out.object_end();
    out.key(b"start");
    out.string(match value.start() {
        w::FollowStart::Owner => b"owner",
        w::FollowStart::Code => b"code",
    });
    if let Some(producer) = value.producer() {
        out.key(b"producer");
        out.string(producer);
    }
    out.key(b"check");
    out.object_start();
    out.key(b"workflow");
    out.string(value.verification().workflow());
    out.key(b"artifact");
    out.string(value.verification().artifact());
    out.key(b"format");
    out.string(match value.verification().format() {
        w::FollowFormat::Junit => b"junit",
        w::FollowFormat::Lines => b"lines",
    });
    out.object_end();
    out.object_end();
}
