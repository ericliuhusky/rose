pub mod task;
mod pid;

use core::cell::{RefCell, Ref, RefMut};

use crate::loader::通过名称读取应用数据;
use crate::trap::trap_return;
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

    // fn take_current() -> Rc<RefCell<TaskControlBlock>> {
    //     unsafe {
    //         TASK_MANAGER.current.take().unwrap()
    //     }
    // }

    pub fn 添加任务(任务: Rc<RefCell<任务>>) {
        unsafe {
            任务管理器.就绪任务队列.push(任务);
        }
    }

    pub fn 暂停并运行下一个任务() {
        Self::可变当前任务(|mut 任务| {
            任务.状态 = 任务状态::就绪;
        });
        // TODO: 纠结用take还是clone
        // let task = Self::take_current();
        // task.borrow_mut().task_status = TaskStatus::Ready;
        // Self::add(task);
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
        // TODO: 将子进程挂在初始进程下面，也就是删除了当前进程它的子进程却不一定也结束
        // {
        //     let mut initproc_inner = INITPROC.inner.exclusive_access();
        //     for child in inner.children.iter() {
        //         initproc_inner.children.push(child.clone());
        //     }
        // }
        Self::运行下一个任务();
    }

    pub fn 运行下一个任务() {
        unsafe {
            let 下一个任务 = 任务管理器.就绪任务队列.remove(0);
            下一个任务.borrow_mut().状态 = 任务状态::运行;
            任务管理器.当前任务 = Some(下一个任务);
            
            trap_return();
        }
    }

    pub fn 添加初始进程() {
        Self::添加任务(Rc::new(RefCell::new(
            任务::新建(通过名称读取应用数据("initproc\0").unwrap())
        )));
    }
}

pub static mut 任务管理器: 任务管理器 = 任务管理器 {
    当前任务: None,
    就绪任务队列: Vec::new(),
};
