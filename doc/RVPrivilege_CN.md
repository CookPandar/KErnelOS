# RISC-V特权级寄存器及指令文档

v0.1 2021/05/12 张树文 KErnelOS Group

v0.2 2021/05/24 张树文 KErnelOS Group

前言：本文介绍了与RISC-V特权级相关的寄存器，从特权级上大致可分为**M态寄存器**和**S态寄存器**。

##### 什么是特权级？

在任何时候，一个RISC-V硬件线程(hart)都处在某种特权级别运行，这些特权级信息记录在一个或多个CSR(控制和状态寄存器)中。目前有3个特权级，特权级由高到低依次是：M态(Machine)，S态(Supervisor)，U态(User/Application)。特权级别用于在软件堆栈的不同组件之间提供保护，低特权级无权访问高特权级的内存空间，也无权使用高特权级的指令，以实现内核对于用户程序的绝对控制，保证程序运行的安全。

##### 特权级寄存器有哪些？

特权级的切换与**中断**和**内存**映射密切相关，特权级寄存器既储存了中断管理信息，也有地址映射的管理机制。
从**中断**的角度上讲，为了储存中断的使能信息、实现中断发生时上下文的保存、判断中断发生的原因、寻找中断处理例程的入口，需要有一组能够储存对应信息的寄存器；
从**内存**的角度上讲，为了限制低特权级对于内核空间的访问，需要有一些记录用户程序可访问空间的寄存器，也就是要管理内存映射的范围；同时地址映射模式也是可以进行调整的：机器一定实现了M态，但未必都有S态与U态，因此地址映射模式需要依据机器的实际情况进行调整，在RISC-V中，这些标志位是由 mstatus 寄存器中的VM位规定的。

##### 为什么这样划分？

M态寄存器主要记录了从低于M态的特权级切换到M态时的信息，S态记录了从低于S态到S态切换时的信息，这两组寄存器功能上大体类似，且覆盖了所有与特权级切换有关的部分。特权级切换并非是一级一级切换的，当用户需要执行对应等级的系统调用时，将会切换由对应的特权级管理程序代为执行操作，由于大多数系统调用都是在M态下实际执行的，所以经常可以看到由低权限切换到M态。当然，RISC-V也提供了中断委托机制，使得M态可以选择将一些调用交由S态实现，我们的文档中也有对应的中断委托寄存器。

## M态寄存器

### 1. ISA寄存器 misa（read-write）

![寄存器结构](https://img-blog.csdnimg.cn/20210505114439668.png)
**base域**：
1.编码了内部支持的ISA宽度.
2.当重置时，总是设置为支持的最宽ISA
![字母-扩展对应表](https://img-blog.csdnimg.cn/20210505114527993.png?x-oss-process=image/watermark,type_ZmFuZ3poZW5naGVpdGk,shadow_10,text_aHR0cHM6Ly9ibG9nLmNzZG4ubmV0L1BhbmRhY29va2Vy,size_16,color_FFFFFF,t_70)

**extensions域**：
功能：
1.编码了现有的标准扩展。依照每个字母一位的方式 ，第0位代表A，第25位代表Z。 保留位总返回0
2.重置时为最大的扩展集合
![字母扩展对应表](https://img-blog.csdnimg.cn/20210505115615251.png?x-oss-process=image/watermark,type_ZmFuZ3poZW5naGVpdGk,shadow_10,text_aHR0cHM6Ly9ibG9nLmNzZG4ubmV0L1BhbmRhY29va2Vy,size_16,color_FFFFFF,t_70)

### 2. mstatus 寄存器

XLEN-bit read/write register

**寄存器结构：**

![mstatus寄存器](https://img-blog.csdnimg.cn/20210505115710702.png?x-oss-process=image/watermark,type_ZmFuZ3poZW5naGVpdGk,shadow_10,text_aHR0cHM6Ly9ibG9nLmNzZG4ubmV0L1BhbmRhY29va2Vy,size_16,color_FFFFFF,t_70)
**位描述：**
**xIE域**：  当硬件线程运行于x模式，对应的中断使能位xIE置位1。低优先级的中断总是关闭，高优先级的中断总是使能。高优先级可以改变特定级别的使能位。

**xPIE域**：双层的栈，用于支持嵌套trap。其保存trap发生前的使能情况

**xPP域**保存trap发生前的特权级，UPP总为0。例如，由特权级y进入特权级x。则xPP设为y；xPIE设为yIE

xRET指令：设xPP的值为y，则yIE设为xPIE，xPIE为1，xPP设为U

**虚拟内存管理域：**
**VM[4:0]** 表明**现在激活的虚拟内存映射**与**保护机制**配置
各模式介绍：
Mbare模式：重置后默认进入的模式。该模式下没有内存映射机制，也没有特权等级，虚拟地址等于物理地址。
Mbb模式(base-and-bounds) 存在于至少有U和M两种特权级的机器中。适用于需要低开销的内存转换和不需要页机制的内存映射中。该模式下，地址和数据段分离以实现
代码段共享。
Sv32模式：为RV32系统提供的32位页式内存管理架构。提供32位虚拟地址内存映射，以支持类UNIX系统
Sv39模式：为RV64系统提供的39位页式内存管理架构。
Sv48模式：为RV64系统提供的48位页式内存管理架构。
![在这里插入图片描述](https://img-blog.csdnimg.cn/20210508204350317.png?x-oss-process=image/watermark,type_ZmFuZ3poZW5naGVpdGk,shadow_10,text_aHR0cHM6Ly9ibG9nLmNzZG4ubmV0L1BhbmRhY29va2Vy,size_16,color_FFFFFF,t_70)

内存优先级域：
**MPRV**：修改了 load 和 store 执行的特权级。当 MPRV=0 时，翻译和保护如同寻常一样。当MPRV=1时，数据存储器地址就如同PRV被设置为当前PRV1字段的值一样被翻译和保护。指令地址翻译和保护不受影响：

**MXR:** 修改加载存取虚拟内存的特权等级。当MXR=0时，只能加载被标示为可读的页(R=1)；当MXR=1，可以加载被标识为可读或可执行(R=1或X=1)的页

**PUM(Protect User Memory):**
MXR: 当PUM=0，翻译和保护如同寻常一样；当PUM为1，s模式下对应于User模式内存地址的页是默认的。PUM在页机制无效时不起作用。

**拓展域**：在不改变特权级的前提下，扩展一些用户态的指令
**FS**   用于维护或者反映浮点单元状态的位域，并追踪指向的单元的状态。这个位域可以用于操作系统上下文切换时对浮点状态的判断，在内容切换周期内通过检查这些位域可以快速判断是否有存储内容需要更新
**Off**：访问off状态的回应是一个异常
**Initial**：Initial状态的回应是初始值
**Clean**：Clean表示自上次修改后没有改变，但与初值不同
**Dirty**：自上次修改后有改变，需要复写入存储空间中
**XS**   用于维护或者反映用户自定义扩展指令单元的状态。这个位域可以用于操作系统上下文切换时对用户自定义扩展指令单元状态的判断。
**SD位**：用于反映FS或者XS的位域是否处于脏（dirty）状态。这个位域是 FS，XS 状态的汇总，方便操作系统上下文切换时快速判断
![在这里插入图片描述](https://img-blog.csdnimg.cn/20210508204421670.png?x-oss-process=image/watermark,type_ZmFuZ3poZW5naGVpdGk,shadow_10,text_aHR0cHM6Ly9ibG9nLmNzZG4ubmV0L1BhbmRhY29va2Vy,size_16,color_FFFFFF,t_70)

## 3. xtvec寄存器：

**BASE**： 必须4字节对齐，用于存储自陷向量的基址
**MODE**：  0 表示所有异常都把PC设置为BASE； 1表示在部分中断中将PC设为(BASE+(4*cause))

**中断委托机制**：
默认情况下，任何特权级别的所有陷阱都会在机器模式下处理，当然机器模式处理程序可以使用 MRET 指令将陷阱重定向回适当的模式级别。但是为了提高性能，RISC-V 提供了一种硬件机制，那就是异常中断委托机制。有了这个机制后，就不再需要软件程序上使用 MRET 指令将陷阱重定向回想要的模式级别。
medeleg 和 mideleg 寄存器中提供单独的读/写位，来指定某些异常和中断类型可以直接由某一较低的模式来处理。

## 4.trap授权寄存器

medeleg and mideleg  （XLEN-bit read/write registers）

有一组**功能类似**、专门用于**中断委托机制**的寄存器：

#### Medeleg [machine exception delegation register]

机器异常委托寄存器

### Mideleg [machine interrupt delegation register]

机器中断委托寄存器

**功能：**
**1**.置位将把S或U态的trap转交给S态的trap处理程序；
**2**.另外：S态也有类似的sedeleg sideleg寄存器使得能够将U态的trap交由U态的trap处理程序做。

一个trap被授权到x的低等级模式时，除mstatus外高特权级的其他寄存器不会有变化，但x的寄存器会有以下操作：
1. xcause     写入trap的原因
2. xepc       写入产生trap的指令的虚拟地址
mstatus寄存器的操作：
1. xPP域写入激活的特权级
2. xPIE域   激活特权级的位被使能
3. xIE域      清空

**寄存器结构：**
![在这里插入图片描述](https://img-blog.csdnimg.cn/20210508205436717.png?x-oss-process=image/watermark,type_ZmFuZ3poZW5naGVpdGk,shadow_10,text_aHR0cHM6Ly9ibG9nLmNzZG4ubmV0L1BhbmRhY29va2Vy,size_16,color_FFFFFF,t_70)
![在这里插入图片描述](https://img-blog.csdnimg.cn/20210508205443196.png?x-oss-process=image/watermark,type_ZmFuZ3poZW5naGVpdGk,shadow_10,text_aHR0cHM6Ly9ibG9nLmNzZG4ubmV0L1BhbmRhY29va2Vy,size_16,color_FFFFFF,t_70)

**medeleg**：依据trap发生的原因，设置为对应的mcause中的编码值
Mideleg 中编码值的含义和mip寄存器中的一致

### 4. 中断寄存器 mip 和mie

XLEN-bit read/write register
![在这里插入图片描述](https://img-blog.csdnimg.cn/20210508205801897.png?x-oss-process=image/watermark,type_ZmFuZ3poZW5naGVpdGk,shadow_10,text_aHR0cHM6Ly9ibG9nLmNzZG4ubmV0L1BhbmRhY29va2Vy,size_16,color_FFFFFF,t_70)

当一个中断由mideleg寄存器中的对应位授权为x模式后，对应的xip寄存器可见，xie寄存器设置标记

**mip位含义**：包含关于未决中断即已置位中断标志位，但还未进入中断服务程序）的信息
USIP, SSIP, HSIP 对应低特权级的软中断
MTIP, HTIP, STIP, UTIP 位对应M H S U模式的时钟未决中断（UTIP, STIP and HTIP  位可由M模式将时钟中断传递到低特权级，并且可以通过AEE SEE HEE的调用清零；MTIP位是只读的，且通过写到内存映射的机器模式定时器比较寄存器清零
包含中断使能位的xlen位读/写寄存器对应外部未决中断 ，这些位只读且由平台特定的中断控制置位与清零
MEIP, HEIP, SEIP, UEIP 是外部未决中断位

**mie位含义**：包含中断使能位的xlen位读/写寄存器
USIE, SSIE, HSIE 对应低特权级的软中断使能位
MTIE, HTIE, STIE, and UTIE位是单独的时钟中断使能位
MEIE, HEIE, SEIE, and UEIE是独立的外部中断使能位

**流程**：如果mip和mie对应位都置位，且全局中断使能有效，那么中断将发生。

**全局中断使能有效的情况**：
1 .默认情况下，如果hart的当前特权模式小于M，则全局启用m模式中断
2.当前特权模式为M，并且mstatus寄存器中的MIE位被设置。如果i位在mideleg中置位
，那么当hart目前的特权级等同于授权特权级且该模式的中断启用
位(mstatus中的HIE、SIE或UIE)置位时，全局中断有效。或者当前特权模式小于授权模式

相同特权级下的中断处理优先级由高到低： 外部中断 软中断  时钟中断  同步trap

### 5. Mtvec

XLEN-bit read/write register
![在这里插入图片描述](https://img-blog.csdnimg.cn/20210508205826707.png?x-oss-process=image/watermark,type_ZmFuZ3poZW5naGVpdGk,shadow_10,text_aHR0cHM6Ly9ibG9nLmNzZG4ubmV0L1BhbmRhY29va2Vy,size_16,color_FFFFFF,t_70)

**功能**：储存了M模式下的trap向量基址，mtvec寄存器中的值必须4字节边界对齐。默认情况下 M态的traps都将使PC的值置为mtvec中的值，重置向量和不可屏蔽中断向量的位置是实现定义的。

## 6. mtime 真实时间计数器

**寄存器结构**：
![在这里插入图片描述](https://img-blog.csdnimg.cn/20210508205832510.png?x-oss-process=image/watermark,type_ZmFuZ3poZW5naGVpdGk,shadow_10,text_aHR0cHM6Ly9ibG9nLmNzZG4ubmV0L1BhbmRhY29va2Vy,size_16,color_FFFFFF,t_70)

## 7. mtimcmp寄存器：

**功能**：
当mtime寄存器包含的值大于等于mtimecmp寄存器中的值时，产生一个时钟中断请求，且会一直发送中断请求直到写mtimecmp寄存器而清除。该中断只有当中断使能且mie寄存器中的MTIE位使能时发生。

**寄存器结构**：
![在这里插入图片描述](https://img-blog.csdnimg.cn/20210508205839536.png?x-oss-process=image/watermark,type_ZmFuZ3poZW5naGVpdGk,shadow_10,text_aHR0cHM6Ly9ibG9nLmNzZG4ubmV0L1BhbmRhY29va2Vy,size_16,color_FFFFFF,t_70)

## 8. 一系列硬件性能监控计数器：

**mcycle** ：记录了某个任意时间之后hart已执行的时钟周期数量
**minstret**:  记录某个任意时间之后hart已执行的指令数量
**29个额外事件计数器 mhpmcounter3-mhpmcounter31**  : 都是WARL寄存器，管理相应事件的计数器会递增；事件的含义由平台定义，但0事件保留为无事件
![ ](https://img-blog.csdnimg.cn/20210508210004797.png?x-oss-process=image/watermark,type_ZmFuZ3poZW5naGVpdGk,shadow_10,text_aHR0cHM6Ly9ibG9nLmNzZG4ubmV0L1BhbmRhY29va2Vy,size_16,color_FFFFFF,t_70)

## 9. m[h|s|u]counteren    机器计数使能寄存器

**功能**：控制硬件性能监控计数器对各个特权级的监控状态
寄存器结构：
![在这里插入图片描述](https://img-blog.csdnimg.cn/20210508210359469.png?x-oss-process=image/watermark,type_ZmFuZ3poZW5naGVpdGk,shadow_10,text_aHR0cHM6Ly9ibG9nLmNzZG4ubmV0L1BhbmRhY29va2Vy,size_16,color_FFFFFF,t_70)

**位功能**：
当 m[h|s|u]counteren 寄存器的CY, TM, IR, or HPMn位清零后，在[]态下任何访问cycle time instret hpmcountern寄存器的操作
都抛出异常；这几位中有任意一个不为0时，这些寄存器的操作是合法的。

## 10.  mscratch

n XLEN-bit read/write register
**功能**：
通常，它用于保存一个指向机器模式hart-local上下文空间的指针，并在进入m模式陷阱处理程序时与用户寄存器交换。
![在这里插入图片描述](https://img-blog.csdnimg.cn/20210508210408394.png?x-oss-process=image/watermark,type_ZmFuZ3poZW5naGVpdGk,shadow_10,text_aHR0cHM6Ly9ibG9nLmNzZG4ubmV0L1BhbmRhY29va2Vy,size_16,color_FFFFFF,t_70)

## 11. mepc：

XLEN-bit read/write register

**功能**：
当trap发生时，mepc将记录发生异常的指令的虚拟地址。

**位介绍**：
![在这里插入图片描述](https://img-blog.csdnimg.cn/20210508210415918.png?x-oss-process=image/watermark,type_ZmFuZ3poZW5naGVpdGk,shadow_10,text_aHR0cHM6Ly9ibG9nLmNzZG4ubmV0L1BhbmRhY29va2Vy,size_16,color_FFFFFF,t_70)

## 12. mcause：

XLEN-bit read-write register
**位功能**：
**1.	Exception Code**：记录上次异常的原因代码   WLRL域
**2.	2.Interrupt位**：当trap由一个中断造成时，对应的中断位置1

**寄存器结构**：![在这里插入图片描述](https://img-blog.csdnimg.cn/20210508210423605.png?x-oss-process=image/watermark,type_ZmFuZ3poZW5naGVpdGk,shadow_10,text_aHR0cHM6Ly9ibG9nLmNzZG4ubmV0L1BhbmRhY29va2Vy,size_16,color_FFFFFF,t_70)

**中断原因代码表**：
![在这里插入图片描述](https://img-blog.csdnimg.cn/20210508210428924.png?x-oss-process=image/watermark,type_ZmFuZ3poZW5naGVpdGk,shadow_10,text_aHR0cHM6Ly9ibG9nLmNzZG4ubmV0L1BhbmRhY29va2Vy,size_16,color_FFFFFF,t_70)

**13. mbadaddr寄存器**：
XLEN-bit read-write register
**功能**：
当一个硬件触发断点，或指令获取、加载或存储地址不对齐或发生访问异常，mbadaddr被写入错误地址。Mbadaddr不因为其他异常修改。
对于具有变长指令的RISC-V系统上的指令读取访问错误，mbadaddr将指向导致错误的指令部分，而mepc将指向指令的开头。

**寄存器结构**：
![在这里插入图片描述](https://img-blog.csdnimg.cn/2021050821044126.png?x-oss-process=image/watermark,type_ZmFuZ3poZW5naGVpdGk,shadow_10,text_aHR0cHM6Ly9ibG9nLmNzZG4ubmV0L1BhbmRhY29va2Vy,size_16,color_FFFFFF,t_70)

## 特权级指令：

**1.	trap返回指令**：
xRET指令可以被x模式或更高的模式执行，将会弹出相关的低特权级的中断使能和特权级堆栈，同时pc将指向xepc寄存器中指向的值。
![在这里插入图片描述](https://img-blog.csdnimg.cn/20210508210451460.png?x-oss-process=image/watermark,type_ZmFuZ3poZW5naGVpdGk,shadow_10,text_aHR0cHM6Ly9ibG9nLmNzZG4ubmV0L1BhbmRhY29va2Vy,size_16,color_FFFFFF,t_70)

**2. WFI (Wait for Interrupt)指令**:
MS特权级均可使用  U特权级是可选的
如果一个已启用的中断存在或之后在hart被停止时出现，中断
异常将在以下指令上被接受，即，在trap处理程序中继续执行和
Mepc = PC + 4。
当中断被禁用时，也可以执行WFI指令。WFI的运作
必须不受mstatus中全局中断位的影响(MIE/HIE/SIE/UIE)(即hart
必须恢复，如果本地启用的中断成为挂起)，但应该尊重个人
中断启用(例如，MTIE)(即，实现应该避免恢复hart如果中断
正在挂起，但没有单独启用)。还要求恢复当地的执行在任何特权级别启用中断挂起，而不考虑每个级别启用的全局中断
特权级别。
![在这里插入图片描述](https://img-blog.csdnimg.cn/20210508210457346.png?x-oss-process=image/watermark,type_ZmFuZ3poZW5naGVpdGk,shadow_10,text_aHR0cHM6Ly9ibG9nLmNzZG4ubmV0L1BhbmRhY29va2Vy,size_16,color_FFFFFF,t_70)

**3.reset指令**
重置时，hart的特权级被设置为M
mstatus的MIE MPRV域被设置为0  VM域被设置为Mbare、
PC设置为已定义的扩展重置向量
Mcause 设置为能够表明重置原因的值；如果具体的实现中不区分重置原因，返回0
所有其他hart的状态是未定义的

**4.None-maskable interrupts 不可屏蔽中断指令**
**功能**： 只用于硬件错误条件，并导致立即跳转到一个实现定义的运行在m模式的NMI向量，而不管hart的中断启用位的状态如何。
在使用NMI时，用下一条要执行的指令的地址写入mepc寄存器，并且mcause被设置为一个指示NMI来源的值。NMI因此可以覆盖一个主动机器模式中断处理程序的状态。
NMI不重置处理器状态或启用硬件错误的诊断、报告和可能的遏制。

**5.Physical Memory Attributes**

**PMA的概念**：
一个完整系统的物理内存映射包括各种地址范围，一些对应于内存区域，一些对应于内存映射的控制寄存器，一些对应于地址空间中的空洞。一些内存区域可能不支持读、写或执行;有些可能不支持子字或子块访问;有些可能不支持原子操作;有些可能不支持缓存一致性，或者可能有不同的内存模型。类似地，内存映射控制寄存器在其支持的访问宽度、对原子操作的支持以及是否读写访问方面也各不相同
在RISC-V系统中，这些特性以及机器物理地址空间的每个区域的能力被称为物理内存属性(pma)。

**PMA的检查**：
PMA值为硬件固有属性，运行期间很少改变。不同存储区域的PMA有的可以改动，有的在制作时就已经固定了，例如ROM。
大多数系统将要求至少有一些pma动态检入硬件物理地址之后的执行管道是已知的，因为某些操作将不被支持
在所有物理内存地址，并且一些操作需要知道当前设置可配置的PMA属性
对于RISC-V，我们将pma的规范和检查分离到一个单独的硬件结构中，

**PMA检查器**：
PMA检查器负责PMA的规范和检查，每个物理的属性在系统设计时是已知的地址区域，并可以硬连线到PMA检查器。在许多情况下。属性在运行时的可以提供可配置的、特定于平台的内存映射控制寄存器来指定这些属性的粒度适合于平台上的每个区域(例如，对于片上SRAM,这可以灵活地划分为可缓存和不可缓存的用途)。pma检查任何对物理内存的访问，包括对物理内存的虚拟访问与翻译。
PMA对于软件是只读的

如果平台支持pma的动态重新配置，将提供一个接口来设置
属性通过向机器模式驱动程序传递请求来正确地重新配置
平台。例如，在某些内存区域上切换可缓存性属性可能涉及到
特定于平台的操作，如缓存刷新，仅对机器模式可用。

**6.Physical Memory Protection   PMP**
为了支持安全处理并包含错误，最好限制运行在hart上的低权限上下文可访问的物理地址。可以提供一个物理内存保护(PMP)单元，通过per-hart机器模式控制寄存器允许为每个物理内存区域指定物理内存访问权限(读、写、执行)。

**各平台的PMP设计不尽相同**：
有些PMP设计可能只会使用一些csr以保护少量的物理内存段，而其他人可能使用内存常驻保护表，保护表缓存由保护表基寄存器索引以细粒度保护大型物理内存空间。系统与保护表基寄存器通常也提供一个物理保护域ID (PDID)寄存器来表示当前物理保护域。

**PMP的检查范围**：
hart运行在S或U模式，且mstatus寄存器中MPRV置位且MPP域包含S或U。

**7 Mbare addressing environment**
**条件**：当重置或在此后的任何时候通过向mstatus寄存器中的VM字段写0时可以进入Mbare模式。
**特性**：
在Mbare环境中，所有虚拟地址都被转换为物理地址，而没有进行任何转换地址，截断任何多余的高阶位。物理内存保护可以用来约束低权限模式的访问。

**8.	Base-and-Bound environments**
Mbb虚拟化环境，提供基本绑定的转换和保护方案。有两个基址和边界的变体：Mbb和Mbbid，这取决于是否有单个Mbb或单独的Mbb用于指令获取和数据访问(Mbbid)。这种简单的转换和保护方案具有低复杂度和确定性高性能的优点，因为在运行过程中从来没有任何TLB丢失。

**(1) Mbb寄存器**：  Single Base-and-Bound registers (mbase, mbound)
功能：
base-and-bound寄存器定义了一个从virtual开始的连续虚拟地址段
地址0，长度由mbound中的值指定，以字节为单位。该虚拟地址段为
mbase寄存器中给出的物理地址开始的连续物理地址段。
此模式下：U、S模式：物理地址=虚拟地址+mbase的值
同时，将虚拟地址与mbound寄存器中的值进行比较。
如果虚拟地址大于等于虚拟地址，则会产生地址故障异常
M模式下的地址不经过Mbb检查

**使能方式**：
Mbb通过向mstatus寄存器中的VM域写1使能。

**寄存器结构**：
简单的Mbb系统有一个基址寄存器mbase和一个边界寄存器mbound。
![在这里插入图片描述](https://img-blog.csdnimg.cn/20210508210513752.png?x-oss-process=image/watermark,type_ZmFuZ3poZW5naGVpdGk,shadow_10,text_aHR0cHM6Ly9ibG9nLmNzZG4ubmV0L1BhbmRhY29va2Vy,size_16,color_FFFFFF,t_70)

**(2) Mbbid寄存器**   分隔指令与数据基址-边界寄存器
![在这里插入图片描述](https://img-blog.csdnimg.cn/20210508210519324.png?x-oss-process=image/watermark,type_ZmFuZ3poZW5naGVpdGk,shadow_10,text_aHR0cHM6Ly9ibG9nLmNzZG4ubmV0L1BhbmRhY29va2Vy,size_16,color_FFFFFF,t_70)

Mbbid方案将指令读取和数据访问的虚拟地址段分开，允许两个或多个用户级虚拟地址空间共享一个物理指令段，同时为每个虚拟地址空间分配一个单独的数据段。
使能方式：将2写入mstatus寄存器的VM字段
寄存器含义：
mibase与mibound定义指令段的物理地址起始地址与指令长度；
mdbase与mdbound定义了数据段的物理地址起始地址与指令长度
数据段虚拟地址从0开始，而指令虚拟地址段从虚拟地址空间的一半开始。起始地址为前导1，后面跟着XLEN-1尾随零(例如，32位地址空间系统的0x8000 0000)
地址检查：
虚拟地址对于较低特权模式的指令，首先检查其取指令以确保其高位已设置;如果不是这样,
产生异常。随后，当加基时，高位被视为零并在检查时绑定虚拟地址。

## S态特权级

**寄存器：**

### 1. Sstatus

XLEN-bit read/write register

![在这里插入图片描述](https://img-blog.csdnimg.cn/20210508210527469.png?x-oss-process=image/watermark,type_ZmFuZ3poZW5naGVpdGk,shadow_10,text_aHR0cHM6Ly9ibG9nLmNzZG4ubmV0L1BhbmRhY29va2Vy,size_16,color_FFFFFF,t_70)

**位说明**：
（1）**SPP**表明进入S态之前的模式：当之前为user模式，则为0，其他为1
（2）**SIE**使能S态的全局中断。为0时S态的中断不发生；当运行在U态，SIE中的值被忽略，S态中断使能总被允许；可以通过sie寄存器关闭S态的独立中断源
（3）**SPIE**记录进入S态之前的中断使能情况
（4）**UIE**使能U模式的中断：U模式中断使能当且仅当UIE置位且hart运行在U模式
（5）**UPIE**表明在处理u模式trap前，u模式中断是否使能；当一个URET指令被执行时，UIE被设置为UPIE，且UPIE设置为1。用户级中断是可选的。如果省略，表示UIE和UPIE位固定为零
（6）**PUM**(Protect User Memory) 修改s模式加载、存储和取指令访问虚拟内存的权限.

PUM机制可以防止管理软件无意中访问用户内存。
当**PUM=0**时，翻译和保护行为正常。当**PUM=1**时，s模式内存访问U模式可访问的页面将故障。在u模式下执行PUM不起作用。

sstatus寄存器是mstatus寄存器的子集。在一个简单的实现中，读取或写入sstatus中的任何字段等同于读取或写入mstatus中的同名字段。

### 2. stvec寄存器

XLEN-bit read/write register
**功能**：
保存s模式的trap向量基址。stvec总是4字节对齐

**寄存器结构**：![在这里插入图片描述](https://img-blog.csdnimg.cn/20210508210550431.png?x-oss-process=image/watermark,type_ZmFuZ3poZW5naGVpdGk,shadow_10,text_aHR0cHM6Ly9ibG9nLmNzZG4ubmV0L1BhbmRhY29va2Vy,size_16,color_FFFFFF,t_70)

**3.	Sip sie寄存器**

XLEN-bit read/write register

**寄存器结构**：![在这里插入图片描述](https://img-blog.csdnimg.cn/20210508210556717.png?x-oss-process=image/watermark,type_ZmFuZ3poZW5naGVpdGk,shadow_10,text_aHR0cHM6Ly9ibG9nLmNzZG4ubmV0L1BhbmRhY29va2Vy,size_16,color_FFFFFF,t_70)

sip寄存器是一个**xlen位**的**读/写寄存器**，包含关于挂起中断的信息。sip寄存器中除了SSIP和USIP之外的所有位都是只读的
而sie是对应的包含中断使能位的xlen位读/写寄存器。

### 4. Supervisor Timers and Performance Counters

S态软件使用相同的硬件性能监控设施作为用户模式软件，
包括时间、周期和插入csr。SBI应该提供一种修改机制
计数器的值。
SBI必须提供一种根据实时计数器来调度计时器中断的工具time

### 5. Sscratch Supervisor Scratch Register XLEN-bit read/write register

![在这里插入图片描述](https://img-blog.csdnimg.cn/20210508210607609.png?x-oss-process=image/watermark,type_ZmFuZ3poZW5naGVpdGk,shadow_10,text_aHR0cHM6Ly9ibG9nLmNzZG4ubmV0L1BhbmRhY29va2Vy,size_16,color_FFFFFF,t_70)

通常，当hart执行用户代码时，sscratch用于保存指向hart-local supervisor上下文的指针。在trap处理程序的开头，sscratch与用户寄存器交换，以提供初始工作寄存器。

### 6. Sepc [ S态程序异常计数器 ] XLEN-bit read/write register

![在这里插入图片描述](https://img-blog.csdnimg.cn/20210508210613796.png?x-oss-process=image/watermark,type_ZmFuZ3poZW5naGVpdGk,shadow_10,text_aHR0cHM6Ly9ibG9nLmNzZG4ubmV0L1BhbmRhY29va2Vy,size_16,color_FFFFFF,t_70)
**功能**：
sepc的低位
(sepc[0])总是零。不支持指令集扩展的实现
16位指令对齐时，两个低位(sepc[1:0])总是零。
当trap被捕获时，sepc被写入遇到异常的指令的虚拟地址。

### 7. Scause [S态原因寄存器] XLEN-bit read-write register

**寄存器结构**：
![在这里插入图片描述](https://img-blog.csdnimg.cn/2021050821062149.png?x-oss-process=image/watermark,type_ZmFuZ3poZW5naGVpdGk,shadow_10,text_aHR0cHM6Ly9ibG9nLmNzZG4ubmV0L1BhbmRhY29va2Vy,size_16,color_FFFFFF,t_70)

**原因及值对应表**
![在这里插入图片描述](https://img-blog.csdnimg.cn/20210508210626356.png?x-oss-process=image/watermark,type_ZmFuZ3poZW5naGVpdGk,shadow_10,text_aHR0cHM6Ly9ibG9nLmNzZG4ubmV0L1BhbmRhY29va2Vy,size_16,color_FFFFFF,t_70)

## 8. sbadaddr寄存器 [Supervisor Bad Address]

XLEN-bit read/write register

**寄存器结构**：
![在这里插入图片描述](https://img-blog.csdnimg.cn/20210508210631771.png?x-oss-process=image/watermark,type_ZmFuZ3poZW5naGVpdGk,shadow_10,text_aHR0cHM6Ly9ibG9nLmNzZG4ubmV0L1BhbmRhY29va2Vy,size_16,color_FFFFFF,t_70)

**功能**：
对于具有变长指令的RISC-V系统上的指令读取访问错误，使用sbadaddr
指向导致故障的指令部分，而sepc指向指令的开始。

## 9. sptbr寄存器 [ Supervisor Page-Table Base Register ]

**功能**：
sptbr寄存器只出现在支持**分页虚拟内存**的系统上。这个寄存器保存根页表的物理页号(PPN)，也就是它的s态物理地址除以4 KiB，以及一个地址空间标识符(ASID)，这便于操作基于每个地址空间的地址转换栅栏。
s态**物理地址位的数量**由实现定义;任何未实现的地址位在SPTBR寄存器中硬连线为零。ASID位的个数也是实现定义的，可能为零。实现的ASID位的数量可以通过在ASID字段的每一个位位上写一个位来确定，然后读回sptbr中的值以查看ASID字段中哪些域是1。

**RV32版本**：
![在这里插入图片描述](https://img-blog.csdnimg.cn/20210508210638984.png?x-oss-process=image/watermark,type_ZmFuZ3poZW5naGVpdGk,shadow_10,text_aHR0cHM6Ly9ibG9nLmNzZG4ubmV0L1BhbmRhY29va2Vy,size_16,color_FFFFFF,t_70)

**RV64版本**：
![在这里插入图片描述](https://img-blog.csdnimg.cn/20210508210859686.png?x-oss-process=image/watermark,type_ZmFuZ3poZW5naGVpdGk,shadow_10,text_aHR0cHM6Ly9ibG9nLmNzZG4ubmV0L1BhbmRhY29va2Vy,size_16,color_FFFFFF,t_70)

## S态指令

### 1. M态中有定义过的SRET指令

### 2. SFENCE.VM

S态内存管理栅格指令

![在这里插入图片描述](https://img-blog.csdnimg.cn/20210508210906273.png?x-oss-process=image/watermark,type_ZmFuZ3poZW5naGVpdGk,shadow_10,text_aHR0cHM6Ly9ibG9nLmNzZG4ubmV0L1BhbmRhY29va2Vy,size_16,color_FFFFFF,t_70)

**功能**：
监控器内存管理栅栏指令SFENCE.VM用于将当前执行的更新同步到内存中内存管理数据结构。指令执行导致对这些数据结构的隐式读写;但是，这些隐含的引用
对于指令流中的加载和存储，通常是没有顺序的。执行一个SFENCE.VM指令保证任何存储在SFENCE.VM之前的指令流进行排序。此外，执行一个SFENCE.VM保证任何由先前指令引起的隐式写操作被排序。

SFENCE.VM的行为依赖于sptbr寄存器中ASID字段的当前值。
如果ASID！=0，则SFENCE.VM只对当前地址中的地址转换生效;
如果ASID=0，则SFENCE.VM影响所有地址空间的地址转换。

寄存器操作数rs1包含一个可选的虚地址参数。如果rs1=x0，栅栏会影响所有的虚拟地址转换并且存储任何级别的页表。
当rs1!=x0时，SFENCE.VM序列只存储到rs1中虚拟地址的叶页表，而不存储到其他页表的级别。

### 3. S态对Mbare环境的操作

当在mstatus的VM字段中选择Mbare环境时，S模式虚拟地址被截断并直接映射到S模式物理地址。
在直接转换为机器级物理地址前，将使用任何物理内存保护结构()检查S态物理地址

### 4. S态对Base and Bound环境的操作

当mstatus的VM字段中选择Mbb或Mbbid时，S态虚拟地址根据适当的机器级基础和检查绑定寄存器。之后使用任何物理
内存保护结构(M态的内存保护机制)检查生成的管理器级物理地址，然后直接转换为机器级物理地址。

### 5. Sv32: 页基址32位虚拟内存系统：

当Sv32写入mstatus寄存器中的VM字段时，管理程序以32-操作位分页虚拟内存系统。在RV32系统上支持Sv32，并被设计为包括
机制足以支持现代基于unix的操作系统。

**（1）Sv32页基址39位虚拟内存系统**

**[1] 地址与内存保护**
Sv32虚拟地址、物理地址、页表入口结构：
![在这里插入图片描述](https://img-blog.csdnimg.cn/20210508210920429.png?x-oss-process=image/watermark,type_ZmFuZ3poZW5naGVpdGk,shadow_10,text_aHR0cHM6Ly9ibG9nLmNzZG4ubmV0L1BhbmRhY29va2Vy,size_16,color_FFFFFF,t_70)

**Sv32虚拟地址**：由**虚拟页数**(VPN)与**页偏移**。当mstatus寄存器中的VM字段写入Sv32时，S态虚拟地址到S态物理地址的映射要经过两级页表。20位的VPN将翻译为22位的物理内存页数PPN， 剩余的12位页偏移不变。翻译完成后，将进行物理地址保护机制检查(M态所提到过的)，再转换成机器态物理地址。

**Sv32页表**由210个**页表项**(PTE)组成，每个PTE为4字节。页表的大小与页面的大小相同，并且必须始终与页面边界对齐。根页表的物理页号存储在sptbr寄存器中

**位含义**：
V位表明该PTE是否合法，若是0，则PTE的31-1bit位不关心且可以由软件自由使用。
RWX位为权限位，表明该页是否可读、写、执行；当三者均为0时，PTE是一个指向下级页表的指针；否则，是一个叶页表项
U位表明该页表是否可由U态使用；为1时，可由U态使用，同时若sstatus寄存器中的PUM位清零，则S态软件也可以在U位为1的条件下获取页。但通常情况下，S态的sstatus的PUM为1，因此S态一般不可以访问用户页。
G位表示全局映射。全局映射是存在于所有地址空间中的映射。对于非叶pte，全局设置意味着页面后续级别中的所有映射表是全局性的。注意，不将全局映射标记为全局映射只会降低性能，而将非全局映射标记为全局则是错误的。
A获取位， 虚拟地址被读写或匹配时，对应的PTE的A位被置位
D脏位    当虚拟地址被写时，对应的PTE的D位被设置

**[2]虚拟地址翻译流程**

**虚拟地址**转换为**物理地址**的方法：

> 1. 让a=sptbr.ppn × PAGESIZE，让i = LEVELS−1。(对于Sv32, PAGESIZE=2^12且 LEVELS= 2)。
> 2. 设pte为地址a+va.vpn[i]×PTESIZE的PTE值。(Sv32 的PTESIZE = 4)。
>    3.如果pte.v = 0，或者pte.r = 0且pte.w = 1，则停止并引发访问异常。
> 3. 否则，PTE有效。如果pte.r = 1或pte.x = 1，请转步骤5。否则，PTE为 指向下一级页表的指针。设i = i−1。如果i < 0，停止并引发访问 例外。否则，让a = pte.ppn × PAGESIZE，然后转到步骤2。
> 4. 一片叶子PTE被找到。通过读取pte.r 、pte.w和pte.x位以确定请求的内存访问是否允许访问， 如果不是，停止并引发访问异常。否则，翻译为 成功的。设置pte.a为1，如果内存访问是存储，则设置pte.d为1。翻译后的 物理地址如下:
>    •pa.pgoff = va.pgoff。 •如果i > 0，那么这是一个超页翻译并且pa.ppn [i−1:0]=
>    va.vpn[i−1:0]。 •pa.ppn[LEVELS−1:i] = pte.ppn[LaaEVELS−1:i]。

**（2） Sv39 页基址39位虚拟内存系统**
**[1] 地址与内存保护**
**虚拟地址**：在翻译64位地址时，必须使63-39位与38位的值相同，否则访问错误将会产生。27位的VPN经过三级页表转化为38位的PPN，剩余的12bit页偏移不翻译
**Sv39页表**：Sv39页表包含2^9个页表项(pte)，每个8字节。页表就是这样页面的大小，并且必须始终与页面边界对齐。根节点的物理地址页表存储在SPTBR寄存器中
**Sv39的PTE格式**如图所示。位9-0与Sv32的含义相同。
位63-48为将来使用保留，为了向前兼容，软件必须将其置零。
![在这里插入图片描述](https://img-blog.csdnimg.cn/20210508210932685.png?x-oss-process=image/watermark,type_ZmFuZ3poZW5naGVpdGk,shadow_10,text_aHR0cHM6Ly9ibG9nLmNzZG4ubmV0L1BhbmRhY29va2Vy,size_16,color_FFFFFF,t_70)

[2]**虚拟地址到物理地址的转换算法**
与Sv32相同，但不同之处是LEVELS等于3,PTESIZE等于8

**（3）Sv48页基址与虚拟内存系统**
**[1] 地址与内存保护**

**虚拟地址**：
支持48位虚拟地址，分成4KiB的页。存储和加载64位有效地址时，63-48必须和47bit相同，否则会产生访问错误。36bit的VPN通过4级页表被翻译成38bit的PPN，剩余的12bit页偏移不翻译。
![在这里插入图片描述](https://img-blog.csdnimg.cn/20210508210942214.png?x-oss-process=image/watermark,type_ZmFuZ3poZW5naGVpdGk,shadow_10,text_aHR0cHM6Ly9ibG9nLmNzZG4ubmV0L1BhbmRhY29va2Vy,size_16,color_FFFFFF,t_70)
![在这里插入图片描述](https://img-blog.csdnimg.cn/20210508210948260.png?x-oss-process=image/watermark,type_ZmFuZ3poZW5naGVpdGk,shadow_10,text_aHR0cHM6Ly9ibG9nLmNzZG4ubmV0L1BhbmRhY29va2Vy,size_16,color_FFFFFF,t_70)

PTE格式：9-0bit与Sv32含义相同，
任何
级别的PTE可能是叶子PTE，所以除了4 KiB页外，Sv48还支持2 MiB megpage，
1GiB gigapages和512GiB terapages，每个都必须在虚拟和物理上对齐
与其大小相等的边界

**[2] 虚拟地址到物理地址的转换算法**
与Sv32相同，但不同之处是LEVELS等于4,PTESIZE等于8


