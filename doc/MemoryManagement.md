# KErnelOS: 内存管理模块

v0.1 2021/05/04 程乾宇 KErnelOS Group

v0.2 2021/05/17 张树文，程乾宇 KErnelOS Group

## 简述

**内存管理**是操作系统所要解决的基本问题之一。在现代操作系统中，通过MMU (内存管理单元, Memory Management Unit) 硬件所提供的**地址映射**功能，可以对指定的连续内存地址空间进行离散化的空间分配。基于这一特性，为了减少内存碎片(Fragmentation)对物理内存空间的浪费，我们一般使用分页策略分配物理内存。

参照RISC-V sv39分页机制标准，虚拟地址中保留其低12位作为页偏移量。因此，我们以4KB大小为单位将物理内存划分为页框 (Frame)，进行内存对齐。为了实现高效的**离散空间（页框）分配**，我们需要专门实现**页分配的分配器**，在用户程序内容进入内存，请求分配空间时，以特定策略选择要分配的空闲页框号。

值得注意的是，现代操作系统提供了两种内存分配方式：**静态分配与动态分配**。

其中，动态分配在**运行时**完成，在操作系统预先定义的**堆**上进行**连续空间分配**。由于动态分配发生时间的特殊性，我们还需要专门实现**堆分配的分配器**。（实际上，这里的连续空间指的是虚拟地址空间，物理分配上还是离散的）

而静态分配在**编译时**完成，包括分配在**栈**上的，来自于正在执行的函数/函数调用栈上栈帧的**局部变量**，以及分配在程序数据段的**全局变量**等。我们不需要直接关注这种分配模式。

考虑到内核代码的灵活性，我们还在操作系统内部，对**地址空间分段**进行了软件实现。同时，我们使用上述实现的页分配分配器，在段的集合（地址空间数据结构）上完成了从虚拟地址到物理地址的映射。

最终，我们将内存管理模块划分为三个子模块以实现内存管理的基本功能：堆分配子模块，页分配子模块，地址空间管理子模块。模块整体实例化三个数据结构：页分配分配器，堆分配分配器，内核地址空间，以及一个初始化函数。

## 堆分配子模块

利用此模块，我们可以在用户程序中使用由rust提供的与动态分配相关的数据结构。如向量Vec<T>, B树BTreeMap<K, V>, 链表LinkedList<T>等。

### 内部数据结构

```rust

pub struct HeapAllocator; // 此数据结构中以某种方式维护空闲堆空间列表
pub struct LockedHeapAllocator(Mutex<HeapAllocator>); // 挂互斥锁的HeapAllocator

pub struct core::alloc::Layout {
	size_: usize, // 分配空间大小
	align_: NonZeroUsize, // 内存对齐单位大小
}
trait core::alloc::GlobalAlloc{ // 全局分配器接口
	unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8; // 依照layout，分配堆空间
	unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout); // 给出指针，依据layout，释放堆空间
}

impl HeapAllocator{
	pub fn new() -> Self; // 新建空堆分配器
	unsafe pub fn init(&mut self, start: usize, size: usize); // 向新建的堆分配器提供堆空间，以start为起始地址，空间大小为size
}
impl core::alloc::GlobalAlloc for LockedHeapAllocator; // 为指定的分配器实现GlobalAlloc接口

impl LockedHeapAllocator{
	pub fn new() -> Self;
}

```

### 实现逻辑

模块中实例化了一个LockedHeapAllocator类全局对象，并定义为#[global_allocator]，使其作为堆分配器生效。除此之外，子模块将某一函数定义为#[alloc_error_handler]，用来处理alloc有关的panic。

子模块中创建了一个全局数组作为堆空间，其指针与空间大小需要通过分配器的`init()`方法传递给给动态分配器，至此堆分配器初始化完成。以上工作在`init_heap_allocator()`中实现。

```rust
pub fn init_heap_allocator(heap_ptr: *const u8, size: usize);
```

## 页分配子模块

利用此模块，可以从内存中未被程序占用的空闲空间中分配物理页框，以及为物理页与指定虚拟页号间创建映射。

### 内部数据结构

#### 地址抽象

将数值包装为物理地址、物理页号、虚拟地址、虚拟页号。利用类型转换接口，实现了几个自定义类型间的页号-地址转换、数字-地址转换、数字-页号转换功能。

```rust
pub struct PhysAddr(pub usize);
pub struct VirtAddr(pub usize);
pub struct PhysPageNum(pub usize);
pub struct VirtPageNum(pub usize);
impl PhysAddr { // 地址4K对齐，获取页号
	pub fn floor(&self) -> PhysPageNum { PhysPageNum(self.0 / PAGE_SIZE) }
	pub fn ceil(&self) -> PhysPageNum { PhysPageNum((self.0 + PAGE_SIZE - 1) / PAGE_SIZE) }
	pub fn page_offset(&self) -> usize { self.0 & (PAGE_SIZE - 1) }
}
impl PhysPageNum{
	// 展开物理页内容
	pub fn get_pte_array(&self) -> &'static mut [PageTableEntry]; // 识别为页表，获取页表项内容
	pub fn get_bytes_array(&self) -> &'static mut [u8]; // 识别为页，按字节获取页内容
	pub fn get_mut<T>(&self) -> &'static mut T; // 提供页内容作为指定类型的引用
}
impl VirtPageNum{
	pub fn vpn_split(&self) -> [usize; 3]; // 基于sv39三级页表标准，提供每一级页表的页号。
}
```

#### 页框分配器

```rust
trait FrameAlloc { // 页框分配器抽象
	fn new() -> Self; // 新建空分配器
	fn init(&mut self, start_ppn: PhysPageNum, end_ppn: PhysPageNum); // 向新建分配器提供内存中未被程序占用的空闲页框号范围
	fn alloc(&mut self) -> Option<PhysPageNum>; // 选择页框分配
	fn dealloc(&mut self, ppn: PhysPageNum); // 释放页框
}
pub struct FrameAllocator;	// 此数据结构以某种方式维护空闲页框列表
pub struct LockedFrameAllocator(Mutex<FrameAllocator>);
impl FrameAlloc for FrameAllocator;
impl LockedFrameAllocator{
	pub fn new() -> Self;
}
```

#### 页表项抽象

封装数值为页表项，将页表项创建封装为函数。

```rust
bitflags! {
	pub struct PTEFlags: u8; // 访问权限标志，对应sv39标准的权限位配置
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct PageTableEntry {
	pub bits: usize,
}
impl PageTableEntry {
	pub fn new(ppn: PhysPageNum, flags: PTEFlags) -> Self;	// 使用权限标志与物理页号构造页表项
	pub fn empty() -> Self; // 返回清零的页表项
	pub fn ppn(&self) -> PhysPageNum
	pub fn flags(&self) -> PTEFlags
	pub fn is_valid(&self) -> bool
}
```

#### 页抽象

将物理页号封装为页，实现一些号码本身不具备的特性，如自动释放页内容等。

```rust
pub struct Page{
	pub ppn: PhysPageNum,
}
pub trait core::ops::Drop {
	pub fn drop(&mut self); // 回收页对象时的析构函数，调用释放页分配
}
impl core::ops::Drop for Page;
impl Page{
	pub fn new(ppn: PhysPageNum) -> Self; // 新建页，页框内清零
}
```

#### 页表抽象

将根页表物理页号与其下分配的页，封装为页表类型，实现指定虚拟页号与物理页号的映射。

```rust
pub struct PageTable(root_ppn: usize, pages: Vec<Page>); // 维护根页表，页表树下的页列表
impl PageTable{
	pub fn new() -> Self; // 新分配一个页表
	pub fn from_token(satp: usize) -> Self; // 根据satp寄存器配置，确定页表所在物理页号
	pub fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags); // 创建页表映射
	pub fn unmap(&mut self, vpn: VirtPageNum); // 删除页表映射
	fn find_pte_create(&self, vpn: VirtPageNum) -> Option<&mut PageTableEntry>; // 查找最后一层页表项，找不到自动创建
	fn find_pte_no_create(&self, vpn: VirtPageNum) -> Option<&PageTableEntry>; // 查找最后一层页表项，不创建
	pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry>; // 翻译返回对应最后一层页表项
	pub fn translate_va(&self, va: VirtAddr) -> Option<PhysAddr>; // 翻译返回对应物理地址
	pub fn token(&self) -> usize; // 页表对应的satp值
}
```

### 实现逻辑

模块中实例化了一个LockedFrameAllocator类全局对象作为页分配器。

子模块查询内核空间终止地址，根据内存终止地址，使用`init()`方法向页分配器传递传递页分配器可用的物理页号范围。以上工作在`init_frame_allocator()`中实现。

子模块提供分配和释放页框的接口，以及每个页表中虚拟物理页号映射创建释放的接口。

```rust
// 子模块接口
pub fn init_frame_allocator(start_ppn: PhysPageNum, end_ppn: PhysPageNum);
pub fn frame_alloc()-> Option<Page>;
pub fn frame_dealloc(ppn: PhysPageNum);

impl PageTable{
	pub fn new() -> Self; // 新分配页表
	pub fn from_token(satp: usize) -> Self; // 根据satp寄存器设置确定页表所在物理页号
	pub fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags); // 创建页表映射
	pub fn unmap(&mut self, vpn: VirtPageNum); // 删除页表映射
	pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry>; // 翻译返回对应最后一层页表项
	pub fn translate_va(&self, va: VirtAddr) -> Option<PhysAddr>; // 翻译返回对应物理地址
	pub fn token(&self) -> usize; // 页表对应satp值
}
```

## 地址空间管理子模块

每个程序都有映射在同一地址范围的独立地址空间，不同的程序可能占用同一个虚拟地址。利用此模块，可以为不同的进程创建地址空间并进行映射、管理。

### 内部数据结构

#### 段抽象

对整个段，也就是一个范围内的虚拟页号分配页框，进行映射。

```rust
pub enum SegType { // 段的映射类型
	Linear,	// 线性映射
	Framed,	// 按页映射
}
bitflags! {
	pub struct SegFlags: u8; // 段访问权限标志
}
pub struct Segment {
	start_vpn: VirtPageNum;
	end_vpn: VirtPageNum;
	frames: BTreeMap<VirtPageNum, Page>,	// 可能不需要维护
	type: SegType,
	flags: SegFlags,
}
impl Segment{
	pub fn new(start_va: VirtAddr, end_va: VirtAddr, type: SegType, flags: SegFlags) -> Self; // 新建指定地址范围的段
	pub fn map_page(&mut self, page_table: &mut PageTable, vpn: VirtPageNum); // 在指定页表上根据虚拟页号范围进行映射
	pub fn map(&mut self, page_table: &mut PageTable); // 根据虚拟页号范围进行映射
	pub fn unmap_page(&mut self, page_table: &mut PageTable, vpn: VirtPageNum); // 在指定页表中根据虚拟页号范围释放映射
	pub fn unmap(&mut self, page_table: &mut PageTable); // 根据虚拟页号范围释放映射
	pub fn copy_data(&mut self, page_table: &mut PageTable, data: &[u8]); // 复制数据到指定页表在虚拟页号范围内所映射的各页空间，起始地址需要4K对齐
}
```
#### 地址空间抽象

对内核进程与用户进程中的不同段进行映射，激活地址转换。

```rust
pub struct AddrSpace{ // 地址空间，各段集合附加一个页表
	page_table: PageTable;
	segments: Vec<Segment>;
}
impl AddrSpace{
	pub fn new() -> Self; // 新建空地址空间, 随之新分配一个页表
	fn push(&mut self, mut segment: Segment, data: Option<&[u8]>); // 把一个段插入到地址空间，在页表上映射
	pub fn push_framed_seg(&mut self, start_va: VirtAddr, end_va: VirtAddr, flags: SegFlags); // 插入分页映射段
	fn map_trampoline(&mut self); // 映射一个Trap handler(Trampoline页)
	pub fn new_kernel() -> Self; // 新建内核地址空间，返回内核地址空间对象，此对象不分配内核栈段
	pub fn new_user_from_elf(elf_data: &[u8]) -> (Self, usize, usize) // 从elf可执行文件数据中新建用户地址空间，返回用户地址空间对象、用户栈顶地址、用户程序入口地址
	pub fn new_user_from_existed(user_space: &AddrSpace) -> AddrSpace // 由已有的用户地址空间创建一个新的用户地址空间，子进程相关
	pub fn activate(&self); // 创建空间完成后需要使用页表的物理页号，激活satp以开启地址转换
	pub fn token(&self) -> usize; // 地址空间的页表对应的satp值
}
pub struct LockedAddrSpace(Mutex<AddrSpace>);
impl LockedAddrSpace{
	pub fn new_kernel() -> Self;
}
```

### 实现逻辑

模块调用`new_kernel()`方法，实例化LockedAddrSpace为内核地址空间。

创建内核空间后，子模块使用该空间的页表的物理页号，激活控制状态寄存器的sv39地址转换功能选项，使物理地址空间转换到虚拟地址空间。以上工作在`init_kernel_space()`中实现。

测试时，模块调用`new_user_from_elf()`方法，使用字节化的用户程序内容，实例化AddrSpace为不同用户程序的用户地址空间，绑定到PCB (进程控制块, Process Control Block) 上。

子模块提供了创建内核地址空间、创建用户地址空间、插入按页映射段、激活地址映射等功能的的接口。

地址空间参考结构 (需要设计好Trap handler模块):

> Figure from: xv6 book, Chap.3 Page Tables
>
> ![Kernel Address Space](./img/kernel_addr_space.png)
>
> ![User Address Space](./img/user_addr_space.png)

```rust
// 子模块接口
impl AddrSpace{
	pub fn new() -> Self;	// 空地址空间, 新分配页表
	pub fn push_framed_seg(&mut self, start_va: VirtAddr, end_va: VirtAddr, flags: SegFlags); // 插入分页映射段
	pub fn new_user_from_elf(elf_data: &[u8]) -> (Self, usize, usize) // 从elf可执行文件数据中新建用户地址空间，返回用户地址空间对象、用户栈顶地址、用户程序入口地址
	pub fn new_user_from_existed(user_space: &AddrSpace) -> AddrSpace // 由已有的用户地址空间创建一个新的用户地址空间，子进程相关
	pub fn activate(&self); // 激活页表
	pub fn token(&self) -> usize; // 地址空间的页表对应satp值
}
impl LockedAddrSpace{
	pub fn new_kernel() -> Self; // 新建内核地址空间，返回内核地址空间对象
}
pub fn get_kernel_ref(token: usize, ptr: *const u8, len: usize) -> Vec<&'static mut [u8]>; // 将用户地址空间的指针，翻译为内核地址空间可用的数组引用列表
pub fn init_kernel_space(); // 激活内核地址空间
```

## 整体数据结构与接口

模块中的全局数据结构定义在`mm/mod.rs`。仅向主程序直接提供初始化接口。

部分操作系统所需要的内存管理分配功能在系统调用中实现。

```rust
extern "C" {
    fn ekernel(); // 内核内容终结地址
}
static mut HEAP_SPACE: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE]; // 对空间
#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::new(); // 全局堆分配器
lazy_static! {
	pub static ref FRAME_ALLOCATOR: LockedFrameAllocator = LockedFrameAllocator::new(); // 页分配器
}
lazy_static! {
	pub static ref KERNEL_SPACE: Arc<LockedAddrSpace> = Arc::new( // 内核地址空间
		LockedAddrSpace::new_kernel()
	);
}
pub fn init() {
	heap_allocator::init_heap_allocator(HEAP_SPACE.as_ptr(), KERNEL_HEAP_SIZE); // 初始化堆分配器
	frame_allocator::init_frame_allocator( // 初始化页分配器
		PhysAddr::from(ekernel as usize).ceil(), 
		PhysAddr::from(MEMORY_END).floor()
	);
	addr_space::init_kernel_space(); // 初始化内核地址空间
}

```
