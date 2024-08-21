use core::cell::RefCell;
use fs::{BlockDevice, FileSystem};
use lazy_static::lazy_static;
use std::fs::{read_dir, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::rc::Rc;

const BLOCK_SIZE: u64 = 0x200;

lazy_static! {
    static ref FILE: RefCell<File> = RefCell::new(
        OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("../user/target/riscv64gc-unknown-none-elf/release/fs.img")
            .unwrap()
    );
}

struct FileBlockDevice;
impl BlockDevice for FileBlockDevice {
    fn read_block(&self, i: usize, buf: &mut [u8]) {
        FILE.borrow_mut()
            .seek(SeekFrom::Start(i as u64 * BLOCK_SIZE))
            .unwrap();
        FILE.borrow_mut().read(buf).unwrap();
    }

    fn write_block(&self, i: usize, buf: &[u8]) {
        FILE.borrow_mut()
            .seek(SeekFrom::Start(i as u64 * BLOCK_SIZE))
            .unwrap();
        FILE.borrow_mut().write(buf).unwrap();
    }
}

pub fn fs_pack() {
    let block_device = Rc::new(FileBlockDevice);
    let fs = FileSystem::format(block_device, 64000000);

    let apps: Vec<String> = read_dir("../user/src/bin")
        .unwrap()
        .into_iter()
        .map(|dir_entry| {
            let name_with_ext = dir_entry.unwrap().file_name().into_string().unwrap();
            name_with_ext[..name_with_ext.find('.').unwrap()].to_string()
        })
        .collect();

    for app in apps {
        let mut f = File::open(format!(
            "../user/target/riscv64gc-unknown-none-elf/release/{}",
            app
        ))
        .unwrap();
        let mut all_data = Vec::<u8>::new();
        f.read_to_end(&mut all_data).unwrap();
        let i = fs.create(&app);
        fs.write(i, &all_data);
    }
}

fn main() {
    fs_pack();
}
