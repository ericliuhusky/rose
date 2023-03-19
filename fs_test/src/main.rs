mod fs_pack;
use fs_pack::{MemoryBlockDevice, fs_pack};
use fs::FileSystem;
use std::rc::Rc;

fn main() {
    fs_pack();
}

#[test]
fn efs_test() -> std::io::Result<()> {
    let block_device = Rc::new(MemoryBlockDevice);
    FileSystem::create(block_device.clone());
    let efs = FileSystem::open(block_device.clone());
    let root_inode = FileSystem::root_inode(&efs);
    root_inode.create("filea");
    root_inode.create("fileb");
    for name in root_inode.ls() {
        println!("{}", name);
    }
    let filea = root_inode.find("filea").unwrap();
    let greet_str = "Hello, world!";
    filea.write_at(0, greet_str.as_bytes());
    let mut buffer = [0u8; 233];
    let len = filea.read_at(0, &mut buffer);
    assert_eq!(greet_str, core::str::from_utf8(&buffer[..len]).unwrap(),);

    filea.clear();
    assert_eq!(filea.read_at(0, &mut buffer), 0);
    Ok(())
}
