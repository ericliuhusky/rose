pub mod task;
mod pid;

use core::cell::{RefCell, Ref, RefMut};

use crate::loader::{读取应用数目, 读取应用数据, 通过名称读取应用数据};
use alloc::{vec::Vec, rc::Rc};
use crate::格式化输出并换行;
use crate::终止::终止;
use self::task::{任务, 任务状态};


pub struct 任务管理器 {
    pub 当前任务: Option<Rc<RefCell<任务>>>,
    就绪任务队列: Vec<Rc<RefCell<任务>>>,
}

impl 任务管理器 {
    pub fn 当前任务() -> Ref<'static, 任务> {
        unsafe {
            任务管理器.当前任务.as_ref().unwrap().borrow()
        }
    }

    pub fn 可变当前任务<F, T>(f: F) -> T where F: FnOnce(RefMut<'static, 任务>) -> T {
        f(unsafe {
            任务管理器.当前任务.as_ref().unwrap().borrow_mut()
        })
    }

    pub fn 添加任务(任务: Rc<RefCell<任务>>) {
        unsafe {
            任务管理器.就绪任务队列.push(任务);
        }
    }

    pub fn 暂停并运行下一个任务() {
        Self::可变当前任务(|mut 任务| {
            任务.状态 = 任务状态::就绪;
        });
        unsafe {
            Self::添加任务(Rc::clone(任务管理器.当前任务.as_ref().unwrap()));
        }
        Self::运行下一个任务();
    }

    pub fn 终止并运行下一个任务(终止代码: i32) {
        if Self::当前任务().进程标识符.0 == 0 {
            格式化输出并换行!("[Kernel] exit!");
            终止();
        }
        Self::可变当前任务(|mut 任务| {
            任务.状态 = 任务状态::终止;
            任务.终止代码 = 终止代码;
            任务.子进程列表.clear();
            任务.地址空间.回收物理帧();
        });
        Self::运行下一个任务();
    }

    pub fn 运行下一个任务() {
        unsafe {
            let 下一个任务 = 任务管理器.就绪任务队列.remove(0);
            下一个任务.borrow_mut().状态 = 任务状态::运行;
            任务管理器.当前任务 = Some(下一个任务);
            
            extern "C" {
                fn __restore(user_satp: usize);
            }
            __restore(Self::当前任务().地址空间.token());
        }
    }

    pub fn 添加初始进程() {
        Self::添加任务(Rc::new(RefCell::new(
            任务::新建(通过名称读取应用数据("initproc\0"))
        )));
    }
}

pub static mut 任务管理器: 任务管理器 = 任务管理器 {
    当前任务: None,
    就绪任务队列: Vec::new(),
};
