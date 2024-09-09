use core_ext::cell::SafeCell;
use super::linked_list::LinkedList;
use core::alloc::{GlobalAlloc, Layout};
use core::cmp::min;
use core::mem::size_of;

pub struct Global {
    /*
    由32个链表组成的可用地址列表
    链表在列表中的索引代表链表的层级level
    链表中存放起始地址start
    代表从起始地址开始，尺寸为1 << level(2的level次方)的地址块为可用地址
    start..(start + (1 << level))
     */
    free_list: SafeCell<[LinkedList; 32]>,
}

impl Global {
    pub const fn new() -> Self {
        Self {
            free_list: SafeCell::new([LinkedList::new(); 32]),
        }
    }

    pub fn init(&self, start: usize, size: usize) {
        let free_list = self.free_list.borrow_mut();

        let mut start = start;
        let end = start + size;
        while start < end {
            // 尺寸二进制对齐，但不能超过初始化时所提供的尺寸
            let size = prev_power_of_two(end - start);
            let level = size.trailing_zeros() as usize;

            free_list[level].push(start);
            start += size;
        }
    }
}

unsafe impl GlobalAlloc for Global {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let free_list = self.free_list.borrow_mut();

        // 尺寸二进制对齐，要大于所需要的尺寸才能放得下将要存放的数据
        let size = layout.size().next_power_of_two();
        let level = size.trailing_zeros() as usize;

        // 找到列表中第一个不为空的链表，即有可用地址的链表
        if let Some(i) = (level..free_list.len()).find(|i| !free_list[*i].is_empty()) {
            // 如果有可用地址的链表层级比需要分配的层级大，就对大的地址块进行二分拆分
            if i > level {
                let mut j = i;
                while j >= level + 1 {
                    let start = free_list[j].pop();
                    // 二分拆分 start..(start + (1 << j)) => start..(start + (1 << (j - 1))), (start + (1 << (j - 1)))..(start + (1 << j))
                    free_list[j - 1].push(start + (1 << (j - 1)));
                    free_list[j - 1].push(start);
                    j -= 1;
                }
            }

            // 如果需要分配的层级对应的链表中有可用地址，直接弹出链表中的起始地址，代表地址块被分配占用
            free_list[level].pop() as *mut u8
        } else {
            panic!("heap memory used out");
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let free_list = self.free_list.borrow_mut();

        let size = layout.size().next_power_of_two();
        let level = size.trailing_zeros() as usize;

        // 将指针地址压入链表，代表地址块被回收释放
        free_list[level].push(ptr as usize);

        // 尝试对地址块进行二分合成
        let mut start = ptr as usize;
        for i in level..free_list.len() {
            let buddy_start = start ^ (1 << i);
            // 如果链表中有地址块的兄弟地址块，就把两个地址块二分合成一个更大的地址块
            if free_list[i].remove(buddy_start) {
                free_list[i].pop();
                start = min(start, buddy_start);
                free_list[i + 1].push(start);
            } else {
                break;
            }
        }
    }
}

fn prev_power_of_two(num: usize) -> usize {
    1 << (8 * (size_of::<usize>()) - num.leading_zeros() as usize - 1)
}
