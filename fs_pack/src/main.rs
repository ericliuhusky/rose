use fs::{BlockDevice, FileSystem};
use std::fs::{read_dir, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::mem::MaybeUninit;
use std::rc::Rc;

const BLOCK_SIZE: u64 = 0x200;

static mut FILE: MaybeUninit<File> = MaybeUninit::uninit();

struct FileBlockDevice;
impl BlockDevice for FileBlockDevice {
    fn read_block(&self, i: usize, buf: &mut [u8]) {
        unsafe { FILE.assume_init_ref() }
            .seek(SeekFrom::Start(i as u64 * BLOCK_SIZE))
            .unwrap();
        unsafe { FILE.assume_init_ref() }.read(buf).unwrap();
    }

    fn write_block(&self, i: usize, buf: &[u8]) {
        unsafe { FILE.assume_init_ref() }
            .seek(SeekFrom::Start(i as u64 * BLOCK_SIZE))
            .unwrap();
        unsafe { FILE.assume_init_ref() }.write(buf).unwrap();
    }
}

pub fn fs_pack() {
    unsafe {
        FILE = MaybeUninit::new(
            OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open("../user/target/riscv64imac-unknown-none-elf/release/fs.img")
                .unwrap(),
        );
    }
    let block_device = Rc::new(FileBlockDevice);
    let fs = FileSystem::format(block_device);

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
            "../user/target/riscv64imac-unknown-none-elf/release/{}",
            app
        ))
        .unwrap();
        let mut all_data = Vec::<u8>::new();
        f.read_to_end(&mut all_data).unwrap();
        let mut f = fs.open(&app, true).unwrap();
        f._write(&all_data);
    }
}

fn main() {
    fs_pack();
}
