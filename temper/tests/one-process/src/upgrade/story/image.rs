//! A stopped world's four roots, copied through the fake machine's test reads.
use crate::referee;
use skein_fake_machine::{How, Item, Is, Made, Machine, Opened};

pub(super) struct Image { roots: Vec<Vec<Item>> }
impl Image {
    pub fn read(disk: &mut Machine) -> Self {
        let mut roots = Vec::new();
        for number in 0..4 {
            let root = disk.reopen_root(number);
            roots.push(directory(disk, root, b""));
            disk.close(root);
        }
        Self { roots }
    }
    pub fn restore(&self, store: &mut referee::Store) -> Machine {
        // Reset the composition's prepared-root bindings, as a new machine does.
        drop(temper_playground::machine::fresh());
        let mut disk = Machine::for_user(1000);
        for items in &self.roots { let root = disk.lay(items); disk.close(root); }
        for item in &self.roots[0] {
            if item.path.starts_with(b"transcripts/") && let Made::File(bytes) = &item.made {
                store.transcript_file(item.path.clone(), bytes.clone());
            }
        }
        disk
    }
}
fn directory(disk: &mut Machine, root: Opened, prefix: &[u8]) -> Vec<Item> {
    let fd = if prefix.is_empty() { root } else { disk.open(root, prefix, How::DirectoryNoFollow).expect("image directory") };
    let mut names = Vec::new();
    loop { let page = disk.list(fd,32,4096).expect("image names"); if page.is_empty() {break;} names.extend(page); }
    if !prefix.is_empty() { disk.close(fd); }
    let mut items = Vec::new();
    for (kind, name) in names {
        let path = if prefix.is_empty() { name.into_vec() } else { [prefix,b"/",&name].concat() };
        match kind {
            Is::Directory => {
                let fd = disk.open(root,&path,How::DirectoryNoFollow).expect("image directory mode");
                let stat = disk.stat(fd); disk.close(fd);
                items.push(Item::directory(&path).mode(stat.mode).owner(stat.owner));
                items.extend(directory(disk,root,&path));
            }
            Is::File => {
                let file = disk.open(root,&path,How::ReadNoFollow).expect("image file");
                let stat = disk.stat(file);
                let bytes = disk.read(file,0,u32::try_from(stat.size).expect("bounded story file")).expect("image bytes");
                assert_eq!(u64::try_from(bytes.len()).expect("size"),stat.size);
                items.push(Item::file(&path,&bytes).mode(stat.mode).owner(stat.owner));
                disk.close(file);
            }
            other => panic!("unexpected stopped world file {path:?}: {other:?}"),
        }
    }
    items
}
