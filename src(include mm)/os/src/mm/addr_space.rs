//Segment对应rcore中的MapArea，SegType对应MapType，SegFlags对应MapPermission  对应memorySet
use super::frame_allocator::{PageTable, PageTableEntry, PTEFlags};
use super::frame_allocator::{VirtPageNum, VirtAddr, PhysPageNum, PhysAddr};
use super::frame_allocator::{Page, frame_alloc};
use super::frame_allocator::{VPNRange};

pub enum SegType {
	Linear,	// 线性映射
	Framed,	// 页表映射
}
bitflags! {
	pub struct SegFlags: u8 {
		const R = 1 << 1;
		const W = 1 << 2;
		const X = 1 << 3;
		const U = 1 << 4;
	}
}
pub struct Segment {
	start_vpn: VirtPageNum;
  end_vpn: VirtPageNum;
	frames: BTreeMap<VirtPageNum, Page>,	//可能不需要维护
	type: SegType,
	flags: SegFlags,
}
impl Segment{
  pub fn new(start_va: VirtAddr, end_va: VirtAddr, type: SegType, flags: SegFlags) -> Self{
    let start_vpn: VirtPageNum = start_va.floor();
    let end_vpn: VirtPageNum = end_va.ceil();
    Self {
        vpn_range: VPNRange::new(start_vpn, end_vpn),
        data_frames: BTreeMap::new(),
        map_type,
        map_perm,
    }
} // 新建指定地址范围段

  pub fn map_page(&mut self, page_table: &mut PageTable, vpn: VirtPageNum){
    let ppn: PhysPageNum;
    match self.map_type {
        MapType::Identical => {
            ppn = PhysPageNum(vpn.0);
        }
        MapType::Framed => {
            let frame = frame_alloc().unwrap();
            ppn = frame.ppn;
            self.data_frames.insert(vpn, frame);
        }
    }
    let pte_flags = PTEFlags::from_bits(self.map_perm.bits).unwrap();
    page_table.map(vpn, ppn, pte_flags);
}

  pub fn map(&mut self, page_table: &mut PageTable){
    for vpn in self.vpn_range {
        self.map_one(page_table, vpn);
    }
}

  pub fn unmap_page(&mut self, page_table: &mut PageTable, vpn: VirtPageNum){
    match self.map_type {
        MapType::Framed => {
            self.data_frames.remove(&vpn);
        }
        _ => {}
    }
    page_table.unmap(vpn);
}

  pub fn unmap(&mut self, page_table: &mut PageTable){
    for vpn in self.vpn_range {
        self.unmap_one(page_table, vpn);
    }
}

  pub fn copy_data(&mut self, page_table: &mut PageTable, data: &[u8]){
    assert_eq!(self.map_type, MapType::Framed);
    let mut start: usize = 0;
    let mut current_vpn = self.vpn_range.get_start();
    let len = data.len();
    loop {
        let src = &data[start..len.min(start + PAGE_SIZE)];
        let dst = &mut page_table
            .translate(current_vpn)
            .unwrap()
            .ppn()
            .get_bytes_array()[..src.len()];
        dst.copy_from_slice(src);
        start += PAGE_SIZE;
        if start >= len {
            break;
        }
        current_vpn.step();
    }
} // 起始地址4K对齐

}

// 地址空间抽象 此处的AddrSpace对应rcore中的MemorySet
pub struct AddrSpace{
  page_table: PageTable;
  segments: Vec<Segment>;
}
impl AddrSpace{
  pub fn new() -> Self{
    Self {
        page_table: PageTable::new(),
        areas: Vec::new(),
    }
}	// 空地址空间, 新分配页表
  fn push(&mut self, mut segment: Segment, data: Option<&[u8]>){
    map_area.map(&mut self.page_table);
    if let Some(data) = data {
        map_area.copy_data(&mut self.page_table, data);
    }
    self.areas.push(map_area);
}
  pub fn push_framed_seg(&mut self, start_va: VirtAddr, end_va: VirtAddr, flags: SegFlags){
    self.push(MapArea::new(
        start_va,
        end_va,
        MapType::Framed,
        permission,
    ), None);
} // 插入分页映射段

	fn map_trampoline(&mut self){
    self.page_table.map(
        VirtAddr::from(TRAMPOLINE).into(),
        PhysAddr::from(strampoline as usize).into(),
        PTEFlags::R | PTEFlags::X,
    );
} // 映射Trap handler(Trampoline页)
pub fn from_another(another: &Segment) -> Self {
    Self {
        vpn_range: VPNRange::new(another.vpn_range.get_start(), another.vpn_range.get_end()),
        data_frames: BTreeMap::new(),
        map_type: another.map_type,
        map_perm: another.map_perm,
    }
}
  pub fn new_kernel() -> Self{
    let mut memory_set = Self::new_bare();
    // map trampoline
    memory_set.map_trampoline();
    // map kernel sections
    println!(".text [{:#x}, {:#x})", stext as usize, etext as usize);
    println!(".rodata [{:#x}, {:#x})", srodata as usize, erodata as usize);
    println!(".data [{:#x}, {:#x})", sdata as usize, edata as usize);
    println!(".bss [{:#x}, {:#x})", sbss_with_stack as usize, ebss as usize);
    println!("mapping .text section");
    memory_set.push(MapArea::new(
        (stext as usize).into(),
        (etext as usize).into(),
        MapType::Identical,
        MapPermission::R | MapPermission::X,
    ), None);
    println!("mapping .rodata section");
    memory_set.push(MapArea::new(
        (srodata as usize).into(),
        (erodata as usize).into(),
        MapType::Identical,
        MapPermission::R,
    ), None);
    println!("mapping .data section");
    memory_set.push(MapArea::new(
        (sdata as usize).into(),
        (edata as usize).into(),
        MapType::Identical,
        MapPermission::R | MapPermission::W,
    ), None);
    println!("mapping .bss section");
    memory_set.push(MapArea::new(
        (sbss_with_stack as usize).into(),
        (ebss as usize).into(),
        MapType::Identical,
        MapPermission::R | MapPermission::W,
    ), None);
    println!("mapping physical memory");
    memory_set.push(MapArea::new(
        (ekernel as usize).into(),
        MEMORY_END.into(),
        MapType::Identical,
        MapPermission::R | MapPermission::W,
    ), None);
    println!("mapping memory-mapped registers");
    for pair in MMIO {
        memory_set.push(MapArea::new(
            (*pair).0.into(),
            ((*pair).0 + (*pair).1).into(),
            MapType::Identical,
            MapPermission::R | MapPermission::W,
        ), None);
    }
    memory_set
}	// 返回内核地址空间对象，此对象不分配内核栈段

  pub fn new_user_from_elf(elf_data: &[u8]) -> (Self, usize, usize){
    let mut memory_set = Self::new_bare();
    // map trampoline
    memory_set.map_trampoline();
    // map program headers of elf, with U flag
    let elf = xmas_elf::ElfFile::new(elf_data).unwrap();
    let elf_header = elf.header;
    let magic = elf_header.pt1.magic;
    assert_eq!(magic, [0x7f, 0x45, 0x4c, 0x46], "invalid elf!");
    let ph_count = elf_header.pt2.ph_count();
    let mut max_end_vpn = VirtPageNum(0);
    for i in 0..ph_count {
        let ph = elf.program_header(i).unwrap();
        if ph.get_type().unwrap() == xmas_elf::program::Type::Load {
            let start_va: VirtAddr = (ph.virtual_addr() as usize).into();
            let end_va: VirtAddr = ((ph.virtual_addr() + ph.mem_size()) as usize).into();
            let mut map_perm = MapPermission::U;
            let ph_flags = ph.flags();
            if ph_flags.is_read() { map_perm |= MapPermission::R; }
            if ph_flags.is_write() { map_perm |= MapPermission::W; }
            if ph_flags.is_execute() { map_perm |= MapPermission::X; }
            let map_area = MapArea::new(
                start_va,
                end_va,
                MapType::Framed,
                map_perm,
            );
            max_end_vpn = map_area.vpn_range.get_end();
            memory_set.push(
                map_area,
                Some(&elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize])
            )
        }
    }
    // map user stack with U flags
    let max_end_va: VirtAddr = max_end_vpn.into();
    let mut user_stack_bottom: usize = max_end_va.into();
    // guard page
    user_stack_bottom += PAGE_SIZE;
    let user_stack_top = user_stack_bottom + USER_STACK_SIZE;
    memory_set.push(MapArea::new(
        user_stack_bottom.into(),
        user_stack_top.into(),
        MapType::Framed,
        MapPermission::R | MapPermission::W | MapPermission::U,
    ), None);
    // map TrapContext
    memory_set.push(MapArea::new(
        TRAP_CONTEXT.into(),
        TRAMPOLINE.into(),
        MapType::Framed,
        MapPermission::R | MapPermission::W,
    ), None);
    (memory_set, user_stack_top, elf.header.pt2.entry_point() as usize)
} 	// 返回用户地址空间对象、用户栈顶地址、用户程序入口地址

pub fn new_user_from_existed(user_space: &AddrSpace) -> AddrSpace{
    let mut memory_set = Self::new();
    // map trampoline
    memory_set.map_trampoline();
    // copy data sections/trap_context/user_stack
    for area in user_space.areas.iter() {
        let new_area = Segment::from_another(area);
        memory_set.push(new_area, None);
        // copy data from another space
        for vpn in area.vpn_range {
            let src_ppn = user_space.translate(vpn).unwrap().ppn();
            let dst_ppn = memory_set.translate(vpn).unwrap().ppn();
            dst_ppn.get_bytes_array().copy_from_slice(src_ppn.get_bytes_array());
        }
    }
    memory_set
}

pub fn activate(&self){
    let satp = self.page_table.token();
    unsafe {
        satp::write(satp);
        llvm_asm!("sfence.vma" :::: "volatile");
    }
} // 激活页表

  pub fn token(&self) -> usize{
    self.page_table.token()
} // 地址空间的页表对应satp值
}

pub struct LockedAddrSpace(Mutex<AddrSpace>);
impl LockedAddrSpace{
	pub fn new_kernel() -> Self;
}

lazy_static! {
    pub static ref KERNEL_SPACE: Arc<LockedAddrSpace> = Arc::new(
          LockedAddrSpace::new_kernel()
    )
  }


pub fn init_kernel_space(){
    KERNEL_SPACE.lock().activate();
} // 激活内核地址空间