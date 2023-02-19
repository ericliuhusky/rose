use 系统调用_输出::系统调用_输出;
use 系统调用_退出::系统调用_退出;
use crate::格式化输出并换行;

const 系统调用标识_输出: usize = 1;
const 系统调用标识_退出: usize = 2;

pub fn 系统调用(系统调用标识: usize, 参数: [usize; 3]) -> isize {
    match 系统调用标识 {
        系统调用标识_输出 => {
            系统调用_输出(参数[0] as *const u8, 参数[1])
        },
        系统调用标识_退出 => 系统调用_退出(参数[0] as i32),
        _ => {
            格式化输出并换行!("[kernel] Unsupported syscall_id: {}", 系统调用标识);
            -1
        }
    }
}

mod 系统调用_输出 {
    use crate::输出::输出;

    pub fn 系统调用_输出(字节串指针: *const u8, 字节串长度: usize) -> isize {
        let 字节串 = unsafe { core::slice::from_raw_parts(字节串指针, 字节串长度) };
        let 字符串 = core::str::from_utf8(字节串).unwrap();
        输出(字符串);
        字节串长度 as isize
    }
}

mod 系统调用_退出 {
    use crate::batch::批处理系统;
    use crate::格式化输出并换行;

    pub fn 系统调用_退出(代码: i32) -> isize {
        格式化输出并换行!("[kernel] Application exited with code {}", 代码);
        批处理系统::运行下一个应用();
        -1
    }
}
