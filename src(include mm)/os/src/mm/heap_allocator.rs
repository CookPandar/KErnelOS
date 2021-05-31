use crate::config::KERNEL_HEAP_SIZE;//需要config块
//github中提供的接口都在此文件中实现，在依赖的BuddySystemAllocator中没有重复的实现。
use crate::mm::mmlib::BuddySystemAllocator;

//HeapAllocator与 LockedHeapAllocator(Mutex<HeapAllocator>)在mmlib库中

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeapAllocator = LockedHeapAllocator::empty();

#[alloc_error_handler]
pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}



trait core::alloc::GlobalAlloc{
  unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8;
  unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) ;
}

unsafe impl GlobalAlloc for LockedHeapAllocator {
  unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
      self.0
          .lock()
          .alloc(layout)
          .ok()
          .map_or(0 as *mut u8, |allocation| allocation.as_ptr())
  }

  unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
      self.0.lock().dealloc(NonNull::new_unchecked(ptr), layout)
  }
}


impl LockedHeapAllocator{
  pub  fn new() -> LockedHeapAllocator {
    LockedHeapAllocator(Mutex::new(HeapAllocator::new()))
}
}

impl HeapAllocator{
  pub fn new() -> Self{  HeapAllocator {
    free_list: [linked_list::LinkedList::new(); 32],
    user: 0,
    allocated: 0,
    total: 0,
}}


unsafe pub fn init(&mut self, start: usize, size: usize){
    self.add_to_heap(heap_ptr, heap_ptr + size);
}
}

pub fn init_heap_allocator(heap_ptr: *const u8, size: usize)
{ unsafe {
    HEAP_ALLOCATOR
        .lock()
        .init(heap_ptr as usize, size);
}}





