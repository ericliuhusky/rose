use clap::{App, Arg};
use fs::{BlockDevice, EasyFileSystem};
use std::fs::{read_dir, File};
use std::io::Read;
use std::sync::Arc;

const BLOCK_SZ: usize = 512;

static mut BLOCKS: [[u8; 0x200]; 0x2000] = [[0; 0x200]; 0x2000];

struct MemoryBlockDevice;

impl MemoryBlockDevice {
    fn show() {
        let blocks = unsafe { &BLOCKS };
        for i in 0..blocks.len() {
            Self::show_block(i);
            println!();
        }
    }

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

fn main() {
    easy_fs_pack().expect("Error when packing easy-fs!");
}

fn easy_fs_pack() -> std::io::Result<()> {
    let matches = App::new("EasyFileSystem packer")
        .arg(
            Arg::with_name("source")
                .short("s")
                .long("source")
                .takes_value(true)
                .help("Executable source dir(with backslash)"),
        )
        .arg(
            Arg::with_name("target")
                .short("t")
                .long("target")
                .takes_value(true)
                .help("Executable target dir(with backslash)"),
        )
        .get_matches();
    let src_path = matches.value_of("source").unwrap();
    let target_path = matches.value_of("target").unwrap();
    println!("src_path = {}\ntarget_path = {}", src_path, target_path);
    let block_file = Arc::new(MemoryBlockDevice);
    // 16MiB, at most 4095 files
    let efs = EasyFileSystem::create(block_file, 16 * 2048, 1);
    let root_inode = Arc::new(EasyFileSystem::root_inode(&efs));
    let apps: Vec<_> = read_dir(src_path)
        .unwrap()
        .into_iter()
        .map(|dir_entry| {
            let mut name_with_ext = dir_entry.unwrap().file_name().into_string().unwrap();
            name_with_ext.drain(name_with_ext.find('.').unwrap()..name_with_ext.len());
            name_with_ext
        })
        .collect();
    for app in apps {
        // load app data from host file system
        let mut host_file = File::open(format!("{}{}", target_path, app)).unwrap();
        let mut all_data: Vec<u8> = Vec::new();
        host_file.read_to_end(&mut all_data).unwrap();
        // create a file in easy-fs
        let inode = root_inode.create(app.as_str()).unwrap();
        // write data to easy-fs
        inode.write_at(0, all_data.as_slice());
    }
    // list apps
    // for app in root_inode.ls() {
    //     println!("{}", app);
    // }
    Ok(())
}

#[test]
fn efs_test() -> std::io::Result<()> {
    let block_device = Arc::new(MemoryBlockDevice);
    EasyFileSystem::create(block_device.clone(), 0x1000, 1);
    let efs = EasyFileSystem::open(block_device.clone());
    let root_inode = EasyFileSystem::root_inode(&efs);
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