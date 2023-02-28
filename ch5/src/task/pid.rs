use alloc::vec::Vec;

pub struct 进程标识符管理器 {
    应当分配的进程标识符: usize,
    已回收可用的进程标识符列表: Vec<usize>,
}

impl 进程标识符管理器 {
    pub fn 分配进程标识符() -> 进程标识符 {
        unsafe {
            if let Some(pid) = 进程标识符管理器.已回收可用的进程标识符列表.pop() {
                进程标识符(pid)
            } else {
                let 应当分配的标识符 = 进程标识符管理器.应当分配的进程标识符;
                进程标识符管理器.应当分配的进程标识符 += 1;
                进程标识符(应当分配的标识符)
            }
        }
    }

    fn 回收进程标识符(被回收的进程标识符: usize) {
        unsafe {
            进程标识符管理器.已回收可用的进程标识符列表.push(被回收的进程标识符);
        }
    }
}

static mut 进程标识符管理器: 进程标识符管理器 = 进程标识符管理器 {
    应当分配的进程标识符: 0,
    已回收可用的进程标识符列表: Vec::new(),
};

pub struct 进程标识符(pub usize);

impl Drop for 进程标识符 {
    fn drop(&mut self) {
        进程标识符管理器::回收进程标识符(self.0);
    }
}
