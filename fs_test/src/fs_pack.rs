use fs::{BlockDevice, EasyFileSystem};
use std::{
    fs::{read_dir, File},
    io::{Read, Write},
    rc::Rc,
};
use fs::TOTAL_BLOCK_NUM;

static mut BLOCKS: [[u8; 0x200]; TOTAL_BLOCK_NUM as usize] = [[0; 0x200]; TOTAL_BLOCK_NUM as usize];

pub struct MemoryBlockDevice;

impl MemoryBlockDevice {
    fn show_block(i: usize) {
        let blocks = unsafe { &BLOCKS };
        let block = blocks[i];
        println!("[{}]", i);
        for j in 0..0x20 {
            print!("{:03x}:  ", j * 0x10);
            for k in 0..0x10 {
                let byte = block[j * 0x10 + k];
                print!("{:02x} ", byte);
            }
            println!();
        }
    }
}

impl BlockDevice for MemoryBlockDevice {
    fn read_block(&self, block_id: usize, buf: &mut [u8]) {
        let block = unsafe { BLOCKS[block_id] };
        for i in 0..buf.len() {
            buf[i] = block[i];
        }
    }

    fn write_block(&self, block_id: usize, buf: &[u8]) {
        let block = unsafe { &mut BLOCKS[block_id] };
        for i in 0..buf.len() {
            block[i] = buf[i];
        }
    }
}

pub fn fs_pack() {
    let block_device = Rc::new(MemoryBlockDevice);
    let fs = EasyFileSystem::create(block_device);
    let root_inode = EasyFileSystem::root_inode(&fs);

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
        let inode = root_inode.create(&app).unwrap();
        inode.write_at(0, &all_data);
    }

    let mut f = File::create(format!(
        "../user/target/riscv64gc-unknown-none-elf/release/fs.img",
    ))
    .unwrap();
    f.write_all(unsafe { &BLOCKS.concat() }).unwrap();
}
