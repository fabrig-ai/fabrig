use crate::Service;
use alloc::collections::BTreeMap;
use skein_io::kernel::{Complete, Done, Error, Exit, Fd, Op};
use skein_lib::{List, Queue};
const BOX_BYTES: u32 = 128;
pub struct Kernel {
    pub next: i32,
    pub pipes: BTreeMap<Fd, (Box<[u8]>, usize)>,
    pub unusable: bool,
    pub spawned: u32,
    pub files: u32,
    pub root_closed: bool,
    pub closed: List<Fd>,
    pub box_writes: u32,
    pub box_answers: Queue<u8>,
    pub box_written: List<u8>,
    pub box_read_cancelled: bool,
}
impl Kernel {
    pub fn new() -> Self {
        Self {
            next: 20,
            pipes: BTreeMap::new(),
            unusable: false,
            spawned: 0,
            files: 0,
            root_closed: false,
            closed: List::with_capacity(256),
            box_writes: 0,
            box_answers: Queue::with_capacity(BOX_BYTES),
            box_written: List::with_capacity(BOX_BYTES),
            box_read_cancelled: false,
        }
    }
    fn descriptor(&mut self) -> Fd {
        self.next = self.next.checked_add(1).unwrap();
        Fd::new(self.next)
    }
    fn spawn(&mut self, spawn: &mut skein_io::kernel::Spawn) -> Result<Done, Error> {
        assert_eq!(&*spawn.program, b"/usr/bin/git");
        assert_eq!(spawn.args.last().unwrap().as_ref(), b"version");
        self.spawned = self.spawned.checked_add(1).unwrap();
        if self.unusable {
            Err(Error::NotFound)
        } else {
            for pipe in &mut spawn.pipes {
                let fd = self.descriptor();
                pipe.parent = Some(fd);
                let bytes = match pipe.child {
                    0 | 2 => b"".as_slice(),
                    1 => b"git version 2.53.0\n",
                    _ => panic!("standard pipe"),
                };
                self.pipes.insert(fd, (bytes.into(), 0));
            }
            let pidfd = self.descriptor();
            let sender = skein_io::kernel::Sender::new(u64::try_from(self.next).unwrap());
            Ok(Done::Spawned { pidfd, sender })
        }
    }
    fn box_read(&mut self, buf: &mut [u8]) -> Done {
        let count = buf.len().min(usize::try_from(self.box_answers.len()).unwrap());
        for target in buf.iter_mut().take(count) {
            *target = self.box_answers.pop().unwrap();
        }
        Done::Count(u32::try_from(count).unwrap())
    }
    #[expect(clippy::too_many_lines, reason = "the service fixture routes layout for workspaces, stores and home")]
    pub fn answer(&mut self, service: &mut Service) {
        for _ in 0..service.submissions().len() {
            let mut submit = service.submissions().pop().unwrap();
            let result = match &mut submit.kind {
                Op::Socket { .. } => Ok(Done::Fd(self.descriptor())),
                Op::Bind { addr, .. } => Ok(Done::Bound(*addr)),
                Op::Listen { .. } | Op::MakeDirectory { .. } => Ok(Done::Nothing),
                Op::Close { fd } => {
                    self.root_closed |= *fd == Fd::new(9);
                    self.closed.push(*fd).unwrap();
                    self.pipes.remove(fd);
                    Ok(Done::Nothing)
                }
                Op::Accept { .. } | Op::Connect { .. } => {
                    service.submissions().push(submit);
                    continue;
                }
                Op::Open { path, .. } => {
                    assert!(
                        path.as_ref() == b"trash" || path.as_ref() == b"workspaces" || path.as_ref() == b"stores",
                        "layout opens {path:?}"
                    );
                    self.files = self.files.checked_add(1).unwrap();
                    Ok(Done::Fd(self.descriptor()))
                }
                Op::Rename { from, .. } => {
                    assert!(
                        from.as_ref() == b"workspaces" || from.as_ref() == b"home" || from.as_ref() == b"stores",
                        "layout moves {from:?}"
                    );
                    self.files = self.files.checked_add(1).unwrap();
                    Err(Error::NotFound)
                }
                Op::List { .. } => {
                    self.files = self.files.checked_add(1).unwrap();
                    Ok(Done::Count(0))
                }
                Op::Spawn { spawn } => self.spawn(spawn),
                Op::PipeRead { fd, buf } if *fd == Fd::new(4) => {
                    if self.box_answers.is_empty() {
                        // The box has written nothing; io's bounded read remains in flight.
                        service.submissions().push(submit);
                        continue;
                    }
                    Ok(self.box_read(buf))
                }
                Op::PipeWrite { fd, bytes, from } if *fd == Fd::new(3) => {
                    self.box_writes = self.box_writes.checked_add(1).unwrap();
                    let written = bytes.get(usize::try_from(*from).unwrap()..).unwrap();
                    for byte in written {
                        self.box_written.push(*byte).unwrap();
                    }
                    let left = written.len();
                    Ok(Done::Count(u32::try_from(left).unwrap()))
                }
                Op::PipeRead { fd, buf } => {
                    let (bytes, at) = self.pipes.get_mut(fd).expect("spawned pipe");
                    let n = buf.len().min(bytes.len().checked_sub(*at).unwrap());
                    let end = at.checked_add(n).unwrap();
                    for (target, source) in buf.iter_mut().zip(bytes.get(*at..end).unwrap()) {
                        *target = *source;
                    }
                    *at = end;
                    Ok(Done::Count(u32::try_from(n).unwrap()))
                }
                Op::Wait { .. } => Ok(Done::Exit(Exit::Code(0))),
                Op::Signal { signal, to, .. } => {
                    assert_eq!(*signal, skein_io::kernel::Signal::Kill);
                    assert_eq!(*to, skein_io::kernel::Target::Group);
                    Ok(Done::Nothing)
                }
                Op::Cancel { target } => {
                    for _ in 0..service.submissions().len() {
                        let held = service.submissions().pop().unwrap();
                        if held.op == *target {
                            if let Op::PipeRead { fd, .. } = &held.kind {
                                self.box_read_cancelled |= *fd == Fd::new(4);
                            }
                            service.completions().push(Complete {
                                op: held.op,
                                kind: held.kind,
                                result: Err(Error::Cancelled),
                            });
                        } else {
                            service.submissions().push(held);
                        }
                    }
                    Ok(Done::Nothing)
                }
                other @ (Op::HardLink { .. }
                | Op::Lock { .. }
                | Op::Recv { .. }
                | Op::Send { .. }
                | Op::Shutdown { .. }
                | Op::Read { .. }
                | Op::Write { .. }
                | Op::Append { .. }
                | Op::Truncate { .. }
                | Op::Sync { .. }
                | Op::Stat { .. }
                | Op::Remove { .. }
                | Op::Usage
                | Op::Window { .. }
                | Op::ReadSignal { .. }
                | Op::PipeWrite { .. }) => panic!("unexpected kernel record {other:?}"),
            };
            service.completions().push(Complete { op: submit.op, kind: submit.kind, result });
        }
    }
}
