# RISC-V SBI简述与RustSBI实现接口

v0.1: 2021/04/25 白胜泷 KErnelOS Group

v0.2: 2021/05/04 程乾宇 KErnelOS Group

> Links:
> https://github.com/riscv/riscv-sbi-doc/blob/master/riscv-sbi.adoc RISC-V标准文档
>
> RustSBI:
> https://github.com/luojia65/rustsbi
> https://docs.rs/rustsbi/0.2.0-alpha.3/rustsbi/
>
> OpenSBI:
> https://github.com/riscv/opensbi
> https://github.com/lizhirui/opensbi

### SBI简述及其调用规范

SBI (管程二进制接口, Supervisor Binary Interface) 通过对机器平台特定功能进行抽象，使得运行在S态 (管程态, Supervisor Mode) 的程序可以在不同的RISC-V硬件实现中进行移植。

运行在高特权级M态 (机器态, Machine Mode), 并为运行在S态的程序提供SBI的程序, 称作**SBI实现**或**SEE** (管程态执行环境, Supervisor Execution Environment)。

> ![RISC-V System without H-extension](https://github.com/riscv/riscv-sbi-doc/raw/master/riscv-sbi-intro1.png)
>
> Figure.1 RISC-V System without H-extension

SBI标准文档预先定义了一批接口函数, 为每个函数定义**EID** (Extension ID) 与**FID** (Function ID)。

处理器处于S态, 需要调用SBI时: 

- 在寄存器a7编码EID, 寄存器a6编码FID。
- 在寄存器a0-a5编码接口参数。
- 执行ecall, 转移到M态，执行接口函数。

每个接口函数均在寄存器a0, a1上返回二元组。其中a0为错误码, a1为返回值。

```rust
#[repr(C)]
pub struct SbiRet {
    /// Error number
    pub error: usize,
    /// Result value
    pub value: usize,
}
```

错误码类型: 

|        Error Type         |  Description   | Value |
| :-----------------------: | :------------: | :---: |
|        SBI_SUCCESS        |    正常执行    |   0   |
|      SBI_ERR_FAILED       |    执行失败    |  -1   |
|   SBI_ERR_NOT_SUPPORTED   | 系统调用不支持 |  -2   |
|   SBI_ERR_INVALID_PARAM   |  输入参数无效  |  -3   |
|      SBI_ERR_DENIED       |    拒绝访问    |  -4   |
|  SBI_ERR_INVALID_ADDRESS  |  输入地址无效  |  -5   |
| SBI_ERR_ALREADY_AVAILABLE | 被调用者已启动 |  -6   |

### RustSBI已实现扩展 (For RV64)

##### 基本信息接口扩展 (Base Extension) (完整实现)

|               Function               |      Description      | FID  | EID  |
| :----------------------------------: | :-------------------: | :--: | :--: |
|        get_sbi_spec_version()        | 获取使用的SBI标准版本 |  0   | 0x10 |
|          get_sbi_impl_id()           |     获取SBI实现ID     |  1   | 0x10 |
|        get_sbi_impl_version()        |    获取SBI实现版本    |  2   | 0x10 |
| probe_extension(extension_id: usize) | 检测指定接口扩展支持  |  3   | 0x10 |
|           get_mvendorid()            |   获取机器供应商ID    |  4   | 0x10 |
|            get_marchid()             |    获取机器架构ID     |  5   | 0x10 |
|             get_mimpid()             |    获取机器实现ID     |  6   | 0x10 |

##### 传统接口扩展 (Legacy Extension) (部分实现)

|            Function             | Description  | FID  | EID  |         Replacement EID          |
| :-----------------------------: | :----------: | :--: | :--: | :------------------------------: |
| set_timer_64(time_value: usize) |   设置时钟   |  0   | 0x00 | 0x54494D45 (For Timer Extension) |
| console_putchar(param0: usize)  | 输出一个字符 |  0   | 0x01 |               N/A                |
|        console_getchar()        | 接收一个字符 |  0   | 0x02 |               N/A                |
| send_ipi(hart_mask_addr: usize) | 发送核间中断 |  0   | 0x04 |  0x00735049 (For IPI Extension)  |
|           shutdown()            |     关机     |  0   | 0x08 |               N/A                |

##### 系统重置接口扩展 (System Reset Extension) (仅为QEMU虚拟机平台实现)

|                       Function                       |     Description     | FID  |    EID     |
| :--------------------------------------------------: | :-----------------: | :--: | :--------: |
| system_reset(reset_type: usize, reset_reason: usize) | 系统重置(冷/热重启) |  0   | 0x53525354 |

##### 特定固件SBI接口扩展 (Firmware Specific SBI Extension) (实现者自定义)

For K210 board: 

|                Function                 |       Description       |  FID  |    EID     |
| :-------------------------------------: | :---------------------: | :---: | :--------: |
| sbi_rustsbi_k210_sext(phys_addr: usize) | 注册S态外部中断处理函数 | 0x210 | 0x0A000004 |