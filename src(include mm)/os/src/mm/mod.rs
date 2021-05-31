mod heap_allocator;
mod addr_space;
mod frame_allocator;
mod mmlib;
pub use frame_allocator::{PhysAddr, VirtAddr, PhysPageNum, VirtPageNum, StepByOne};
use crate::config::MEMORY_END;
use crate::config::KERNEL_HEAP_SIZE;
use crate::mm::mmlib::lazy_static::*;
use crate::mm::mmlib::spin::Mutex;

static mut HEAP_SPACE: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];

//整体数据接口结构与接口中的各项声明都放在了对应的模块中，此处只给出总函数init
//HEAP_SPACE在heap_allocator中声明
pub fn init() {
	heap_allocator::init_heap_allocator(HEAP_SPACE.as_ptr(), KERNEL_HEAP_SIZE);
	frame_allocator::init_frame_allocator(
		PhysAddr::from(ekernel as usize).ceil(), 
		PhysAddr::from(MEMORY_END).floor()
	);
	addr_space::init_kernel_space();
}