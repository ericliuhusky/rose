mod fs_pack;
use fs_pack::{MemoryBlockDevice, fs_pack};
use fs::FileSystem;
use std::rc::Rc;

use crate::fs_pack::{INODE_BITMAP_BLOCK_NUM, INODE_AREA_BLOCK_NUM, DATA_BITMAP_BLOCK_NUM, DATA_AREA_BLOCK_NUM};

fn main() {
    fs_pack();
}

#[test]
fn efs_test() -> std::io::Result<()> {
    let block_device = Rc::new(MemoryBlockDevice);
    FileSystem::create(block_device.clone(), INODE_BITMAP_BLOCK_NUM, INODE_AREA_BLOCK_NUM, DATA_BITMAP_BLOCK_NUM, DATA_AREA_BLOCK_NUM);
    let mut efs = FileSystem::open(block_device.clone());
    efs.create_inode("filea");
    efs.create_inode("fileb");
    for name in efs.ls() {
        println!("{}", name);
    }
    let filea = efs.find("filea").unwrap();
    let greet_str = "Hello, world!";
    efs.write_at(filea, 0, greet_str.as_bytes());
    let mut buffer = [0u8; 233];
    let len = efs.read_at(filea, 0, &mut buffer);
    assert_eq!(greet_str, core::str::from_utf8(&buffer[..len]).unwrap(),);

    efs.clear(filea);
    assert_eq!(efs.read_at(filea, 0, &mut buffer), 0);
    Ok(())
}
