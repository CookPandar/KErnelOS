## 有关内核赛评测姬的执行流程

2021/04/20 程乾宇 KErnelOS Group

- 首先：评测机只编译操作系统内核，不编译用户程序。测试时只考虑串口输出结果的正确性。编译生成的内核固件文件名应为：k210.bin。项目中应附加Makefile用于编译构建。评测机测试时执行的命令为：

  ```shell
  make all
  ```

- 用户程序中，标准输出通过使用write函数向1号文件描述符写入实现，所以串口挂载在1号文件描述符上。

- 编译后的用户程序使用FAT32文件系统镜像保存，评测机已经将用户程序镜像文件烧写到开发板的SD卡上。此外，评测机还会将编译得到的操作系统内核固件烧写到K210的Flash上。FAT32镜像文件使用mkfs.vfat命令生成，具体操作方式如下。有关FAT32实现标准文档，以及格式化真实SD卡为FAT32的方法见[此处](https://github.com/oscomp/testsuits-for-oskernel/blob/main/fat32-info.md)。

  ```shell
  dd if=/dev/zero of=disk.img bs=3M count=10
  mkfs.vfat -F 32 disk.img
  # 如果挂载镜像文件
  mount -o loop -t vfat disk.img /mnt
  # 如果挂载SD卡(路径为/dev/sdX)
  # mount -t vfat /dev/sdX /mnt
  cp user/build/riscv64/* /mnt
  umount /mnt
  ```

- 执行测试的过程也是写在操作系统内核中的。内核进程将用户程序拆解，并由此创建用户进程，执行并输出结果。思路参见[rCore: 解析 ELF 文件并创建线程](https://rcore-os.github.io/rCore-Tutorial-deploy/docs/lab-6/guide/part-3.html)。

- 用户程序可以在本机进行编译测试，为C程序。需要下载配置[专供K210的GCC交叉编译工具链](https://github.com/oscomp/testsuits-for-oskernel/blob/main/riscv-syscalls-testing/res/kendryte-toolchain-ubuntu-amd64-8.2.0-20190409.tar.xz)到PATH环境变量。编译方法见[此处](https://github.com/oscomp/testsuits-for-oskernel/blob/main/riscv-syscalls-testing/README.md)。

- （综上所述，赛方完全可以提供一份预编译用户程序镜像供虚拟机上测试用）

- 评测机Rust和C编译环境为:

  - SiFive GCC kendryte Toolchain 8.3.0-2020.04.0

    Platform: riscv64-unknown-elf

  - Rust nightly-2020-06-27 with cargo-binutils and llvm-tools-preview

    Platform: riscv64imac-unknown-none-elf

  无任何第三方库。所有第三方库必须以源代码方式提交（运行在M态的基础程序可以不以源代码格式提交）。