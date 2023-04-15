use super::File;
use page_table::PhysicalBufferList;

pub struct Stdin;
pub struct Stdout;

impl File for Stdin {
    fn read(&mut self, mut buf: PhysicalBufferList) -> usize {
        assert_eq!(buf.len(), 1);
        let c = sbi_call::getchar();
        buf[0] = c as u8;
        1
    }

    fn write(&mut self, _buf: PhysicalBufferList) -> usize {
        unimplemented!()
    }

    fn file_type(&self) -> super::FileType {
        super::FileType::STDIN
    }
}

impl File for Stdout {
    fn read(&mut self, _buf: PhysicalBufferList) -> usize {
        unimplemented!()
    }

    fn write(&mut self, buf: PhysicalBufferList) -> usize {
        let s = buf.to_string();
        print!("{}", s);
        buf.len()
    }

    fn file_type(&self) -> super::FileType {
        super::FileType::STDOUT
    }
}
