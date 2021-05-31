// 内部结构
// 地址抽象
// 支持页号-地址转换、数字-地址转换、数字-页号转换
//frame_allocator集成了rcore中page_table  frame_allocator 部分address的内容


#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysAddr(pub usize);
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtAddr(pub usize);
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysPageNum(pub usize);
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtPageNum(pub usize);
impl PhysAddr { // 页4K对齐
	pub fn floor(&self) -> PhysPageNum { PhysPageNum(self.0 / PAGE_SIZE) }
	pub fn ceil(&self) -> PhysPageNum { PhysPageNum((self.0 + PAGE_SIZE - 1) / PAGE_SIZE) }
	pub fn page_offset(&self) -> usize { self.0 & (PAGE_SIZE - 1) }
}
impl PhysPageNum{	// 展开物理页内容
  pub fn get_pte_array(&self) -> &'static mut [PageTableEntry]{
	let pa: PhysAddr = self.clone().into();
	unsafe {
		core::slice::from_raw_parts_mut(pa.0 as *mut PageTableEntry, 512)
	}
}

  pub fn get_bytes_array(&self) -> &'static mut [u8]{
	let pa: PhysAddr = self.clone().into();
	unsafe {
		core::slice::from_raw_parts_mut(pa.0 as *mut u8, 4096)
	}
}

  pub fn get_mut<T>(&self) -> &'static mut T{
	let pa: PhysAddr = self.clone().into();
	pa.get_mut()
}
}
//至此为address的内容
impl VirtPageNum{
  pub fn vpn_split(&self) -> [usize; 3]; // 三级页号
}
pub type VPNRange = SimpleRange<VirtPageNum>;
// 页表项抽象
bitflags! {
	pub struct PTEFlags: u8 {
		const V = 1 << 0;
		const R = 1 << 1;
		const W = 1 << 2;
		const X = 1 << 3;
		const U = 1 << 4;
		const G = 1 << 5;
		const A = 1 << 6;
		const D = 1 << 7;
	}
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct PageTableEntry {
	pub bits: usize,
}
impl PageTableEntry {
	pub fn new(ppn: PhysPageNum, flags: PTEFlags) -> Self{
        PageTableEntry {
            bits: ppn.0 << 10 | flags.bits as usize,
        }
    }	// 构造页表项
	pub fn empty() -> Self{
        PageTableEntry {
            bits: 0,
        }
    } // 清零的页表项
	pub fn ppn(&self) -> PhysPageNum{
        (self.bits >> 10 & ((1usize << 44) - 1)).into()
    }
	pub fn flags(&self) -> PTEFlags{
        PTEFlags::from_bits(self.bits as u8).unwrap()
    }
  pub fn is_valid(&self) -> bool {
	(self.flags() & PTEFlags::V) != PTEFlags::empty()
}
}

// 页框分配器抽象
trait FrameAlloc {
	fn new() -> Self; // 初始分配器，无空闲页表
	fn init(&mut self, start_ppn: PhysPageNum, end_ppn: PhysPageNum);
	fn alloc(&mut self) -> Option<PhysPageNum>;
	fn dealloc(&mut self, ppn: PhysPageNum);
}
pub struct FrameAllocator{
    current: usize,
    end: usize,
    recycled: Vec<usize>,
}	// 此数据结构维护空闲页框列表   此处FrameAllocator对应rcore中的StackFrameAllocator;原本rcore中没有LockedFrameAllocator
pub struct LockedFrameAllocator(Mutex<FrameAllocator>);

impl FrameAlloc for FrameAllocator{
	pub fn init(&mut self, l: PhysPageNum, r: PhysPageNum) {
        self.current = l.0;
        self.end = r.0;
        println!("last {} Physical Frames.", self.end - self.current);
    }

	fn new() -> Self {
        Self {
            current: 0,
            end: 0,
            recycled: Vec::new(),
        }
    }

    fn alloc(&mut self) -> Option<PhysPageNum> {
        if let Some(ppn) = self.recycled.pop() {
            Some(ppn.into())
        } else {
            if self.current == self.end {
                None
            } else {
                self.current += 1;
                Some((self.current - 1).into())
            }
        }
    }

    fn dealloc(&mut self, ppn: PhysPageNum) {
        let ppn = ppn.0;
        // validity check
        if ppn >= self.current || self.recycled
            .iter()
            .find(|&v| {*v == ppn})
            .is_some() {
            panic!("Frame ppn={:#x} has not been allocated!", ppn);
        }
        // recycle
        self.recycled.push(ppn);
    }

}
lazy_static! {
	pub static ref FRAME_ALLOCATOR: LockedFrameAllocator = LockedFrameAllocator::new();
}

impl LockedFrameAllocator{
	pub fn new() -> Self{
		LockedFrameAllocator(Mutex::new(FrameAllocator::new()))
	}
}
fn frame_dealloc(ppn: PhysPageNum){
    FRAME_ALLOCATOR
        .lock()
        .dealloc(ppn);
}
pub trait Drop {
	pub fn drop(&mut self);
}
impl Drop for Page{
	frame_dealloc(self.ppn);
}

// 页抽象
pub struct Page{
	pub ppn: PhysPageNum,
}
impl Page{
  pub fn new(ppn: PhysPageNum) -> Self; // 页框内清零
}

// 页表抽象   此次的Page对应rcore中的FrameTracker
pub struct PageTable(root_ppn: usize, pages: Vec<Page>); // 维护根页表，页列表

impl PageTable{
  pub fn new() -> Self {
	let frame = frame_alloc().unwrap();
	PageTable {
		root_ppn: frame.ppn,
		frames: vec![frame],
	}
} // 新分配页表

  pub fn from_token(satp: usize) -> Self{
	Self {
		root_ppn: PhysPageNum::from(satp & ((1usize << 44) - 1)),
		frames: Vec::new(),
	}
} // 根据satp寄存器设置确定页表所在物理页号
  
	pub fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags){
        let pte = self.find_pte_create(vpn).unwrap();
        assert!(!pte.is_valid(), "vpn {:?} is mapped before mapping", vpn);
        *pte = PageTableEntry::new(ppn, flags | PTEFlags::V);
    } //创建页表映射

	pub fn unmap(&mut self, vpn: VirtPageNum){
        let pte = self.find_pte_create(vpn).unwrap();
        assert!(pte.is_valid(), "vpn {:?} is invalid before unmapping", vpn);
        *pte = PageTableEntry::empty();
    } //删除页表映射

  fn find_pte_create(&self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
	let idxs = vpn.indexes();
	let mut ppn = self.root_ppn;
	let mut result: Option<&mut PageTableEntry> = None;
	for i in 0..3 {
		let pte = &mut ppn.get_pte_array()[idxs[i]];
		if i == 2 {
			result = Some(pte);
			break;
		}
		if !pte.is_valid() {
			let frame = frame_alloc().unwrap();
			*pte = PageTableEntry::new(frame.ppn, PTEFlags::V);
			self.frames.push(frame);
		}
		ppn = pte.ppn();
	}
	result
}

	fn find_pte_no_create(&self, vpn: VirtPageNum) -> Option<&PageTableEntry> {
        let idxs = vpn.indexes();
        let mut ppn = self.root_ppn;
        let mut result: Option<&PageTableEntry> = None;
        for i in 0..3 {
            let pte = &ppn.get_pte_array()[idxs[i]];
            if i == 2 {
                result = Some(pte);
                break;
            }
            if !pte.is_valid() {
                return None;
            }
            ppn = pte.ppn();
        }
        result
    } // 查找最后一层页表项，不创建

	pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry>{
        self.find_pte(vpn)
            .map(|pte| {pte.clone()})
    } // 翻译返回对应最后一层页表项

  pub fn token(&self) -> usize{
        8usize << 60 | self.root_ppn.0
    } // 页表对应satp值
}


pub fn frame_alloc() -> Option<Page>{
    FRAME_ALLOCATOR
        .lock()
        .alloc()
        .map(|ppn| Page::new(ppn))
}
pub fn init_frame_allocator(start_ppn: PhysPageNum, end_ppn: PhysPageNum){
    extern "C" {
        fn ekernel();
    }
    FRAME_ALLOCATOR
        .lock()
        .init(start_ppn, end_ppn);
}
pub fn translated_byte_buffer(token: usize, ptr: *const u8, len: usize) -> Vec<&'static mut [u8]> {
    let page_table = PageTable::from_token(token);
    let mut start = ptr as usize;
    let end = start + len;
    let mut v = Vec::new();
    while start < end {
        let start_va = VirtAddr::from(start);
        let mut vpn = start_va.floor();
        let ppn = page_table
            .translate(vpn)
            .unwrap()
            .ppn();
        vpn.step();
        let mut end_va: VirtAddr = vpn.into();
        end_va = end_va.min(VirtAddr::from(end));
        if end_va.page_offset() == 0 {
            v.push(&mut ppn.get_bytes_array()[start_va.page_offset()..]);
        } else {
            v.push(&mut ppn.get_bytes_array()[start_va.page_offset()..end_va.page_offset()]);
        }
        start = end_va.into();
    }
    v
}