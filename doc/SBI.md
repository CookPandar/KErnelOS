# RISC-V SBI

```
Links：https://github.com/riscv/riscv-sbi-doc/blob/master/riscv-sbi.adoc 标准
       https://github.com/luojia65/rustsbi 实现
      
rustsbi：https://github.com/luojia65/rustsbi
		https://github.com/oscomp/proj1-rustsbi/
		https://docs.rs/rustsbi/0.2.0-alpha.2/rustsbi/
		
opensbi：https://github.com/riscv/opensbi
		https://github.com/lizhirui/opensbi
		https://github.com/riscv/riscv-sbi-doc/
```

RiSC-V Supervisor Binary Interface.

![](https://github.com/riscv/riscv-sbi-doc/raw/master/riscv-sbi-intro1.png)

Figure:RISC-V System without H-extension

```
ECALL:control transfer instruction

SBI函数必须返回(a0,a1)值对，a0返回错误码

struct sbiret {
	long error;
	long value;
};
```

错误码类型：

|        Error Type         | Value |
| :-----------------------: | :---: |
|        SBI_SUCCESS        |   0   |
|      SBI_ERR_FAILED       |  -1   |
|   SBI_ERR_NOT_SUPPORTED   |  -2   |
|   SBI_ERR_INVALID_PARAM   |  -3   |
|      SBI_ERR_DENIED       |  -4   |
|  SBI_ERR_INVALID_ADDRESS  |  -5   |
| SBI_ERR_ALREADY_AVAILABLE |  -6   |

```rust
# rustsbi中ecall实现的sbi函数有：

# base extension  # base.rs
0 : fn get_spec_version() -> SbiRet
1 : fn get_sbi_impl_id() -> SbiRet
2 : fn get_sbi_impl_version() -> SbiRet
3 : fn probe_extension(extension_id: usize) -> SbiRet
4 : fn get_mvendorid() -> SbiRet
5 : fn get_marchid() -> SbiRet
6 : fn get_mimpid() -> SbiRet

# legacy extension # legacy.rs
0x00 ：pub fn set_timer_64(time_value: usize) -> SbiRet
       pub fn set_timer_32(arg0: usize, arg1: usize) -> SbiRet
[在timer.rs中也有两个实现]
	  fn set_timer(arg0: usize, arg1: usize) -> SbiRet
	  fn set_timer(arg0: usize) -> SbiRet
[rustsbi中实现而adoc中没有]
0x01 ：pub fn console_putchar(param0: usize) -> SbiRet
0x02 : pub fn console_getchar() -> SbiRet
0x04 : pub fn send_ipi(hart_mask_addr: usize) -> SbiRet
0x08 : pub fn shutdown() -> SbiRet
Not Found in file : void sbi_clear_ipi(void)

# remote fence # rfence.rs 
0x05 : fn remote_fence_i(hart_mask: usize, hart_mask_base: usize) -> SbiRet
0x06 : fn remote_sfence_vma(hart_mask: usize, hart_mask_base: usize, start_addr: usize, size: usize) -> SbiRet
0x07 : fn remote_sfence_vma_asid(hart_mask: usize, hart_mask_base: usize, start_addr: usize, size: usize, asid: usize) -> SbiRet
FID #3 : fn remote_hfence_gvma_vmid(hart_mask: usize, hart_mask_base: usize, start_addr: usize, size: usize, vmid: usize) -> SbiRet
FID #4 : fn remote_hfence_gvma(hart_mask: usize, hart_mask_base: usize, start_addr: usize, size: usize) -> SbiRet
FID #5 : fn remote_hfence_vvma_asid(hart_mask: usize, hart_mask_base: usize, start_addr: usize, size: usize, asid: usize) -> SbiRet
fn remote_hfence_vvma(hart_mask: usize, hart_mask_base: usize, start_addr: usize, size: usize) -> SbiRet

# Hart State Management Extension # hsm.rs
FID #0 : fn hart_start(hartid: usize, start_addr: usize, private_value: usize) -> SbiRet
FID #1 : fn hart_stop(hartid: usize) -> SbiRet
FID #2 : fn hart_get_status(hartid: usize) -> SbiRet
Not Found in file : struct sbiret sbi_hart_suspend()

# System Reset Extension # srst.rs
FID #0 : fn system_reset(reset_type: usize, reset_reason: usize) -> SbiRet
```



