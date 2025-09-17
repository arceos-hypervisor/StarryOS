# 基于国产AI芯片的Rust内核组件开发赛

队伍名称：泉城把子肉

队伍成员：赵长收，周睿，柏乔森，杨金全，宋志勇

## **第一部分 项目概述**

基于 Rust 编程语言，完善 StarryOS 组件，并在基于 RK3588 开发板上移植启动，最终实现基于 RK3588 内置 NPU 运行 Yolo-v8 模型，实现流程分为以下四个阶段：

1. StarryOS AArch64 架构适配
2. 移植适配 RK3588 开发板，实现 StarryOS 启动
3. 添加电源驱动、时钟驱动、emmc驱动 、NPU 驱动、PCIe 驱动、USB 驱动、SPI 驱动
4. 添加必要的 StarryOS 组件
5. 移植适配 Yolo-v8 模型

## **第二**部分 硬件平台分析

### **2.1** **芯片/SoC特性分析**

给出你所选用芯片的型号，内置外设和AI加速单元的情况

1. 型号选择：

   Orange Pi5 Plus

2. 特性分析

   基本信息：

   - CPU八核64位（4×Cortex-A76+4×Cortex-A55），主频高达 2.4GHz；

   - ARM Mali-G610 MP4四核GPU，支持 OpenGL ES3.2/OpenCL 2.2/Vulkan1.1, 450 GFLOPS；

   - NPU 算力高达6TOPS(INT8)，支持 INT4/INT8/INT16 混合运算
   - ISP 集成 48MP ISP，支持HDR和3DNR
   - 硬解码 8K@60fps H.265/VP9/AVS2、8K@30fps H.264 AVC/MVC、4K@60fps AV1、1080P@60fps MPEG-2/-1/VC-1/VP8
   - 硬编码 8K@30fps H.265/H.264
   - 内存 LPDDR4/LPDDR4x（4GB/8GB/16GB 可选）
   - 存储 QSPI NOr FLASH: 16MB/32MB; MicroSD 卡插槽: up to 128GB; eMMC闪存插座，可外接 16GB/32GB/64GB/128GB/256GB eMMC模块; 用于 NVMe SSD(PCle 3.0x4) 的 M.2 2280 插槽高达2.000 MB/S

      接口参数：

   - 以太网：2 × PCIe 2.5G LAN(RTL8125BG)
   - 视频:  2xHDMI2.1 输出，高达 8K@60FPS; 1xType-C(DP1.4A) 输出，高达 8K@30FPS; 1xHDMI 输入，高达 4K@60FPS; 1xMIPIDSl4Lane 输出，高达 4K@60HZ
   - 音频输出 1 × 3.5mm音频接口（支持MIC录音，美标 CTIA）
   - TP接口 1x6Pin FPC插座
   - USB 2 × USB3.0（限流: 1A；上层：USB3.0(1)，与5G复用；下层：USB3.0(2)）
   - 扩展口 40Pin双排插针，具有以下复用功能: UART、I2C、SPI、CAN、I2S、PDM、AUDDSM、SDIO、PWM、GPIO
   - PCle M.2 M-KEY Socket： M.2 connector E key (top) for connectivity with PCle 3.0 x 4 lanes 2280 SSD固态硬盘
   - PCle M.2 E-KEY Socket： M.2 connector E key (top) for connectivity with PCle 2.0x1/PCM/UART/USB2.0，支持2230 Wi-Fi6 /BT模块

3. AI加速单元（NPU）

   算力：内置6TOPS NPU（INT4/INT8/INT16/FP16混合运算），支持端侧主流3B以下参数模型部署

   兼容性：

   - 支持主流AI框架：TensorFlow、PyTorch、MXNet、Caffe等
   - 支持模型转换工具（如MemryXDFP框架），简化部署流程

### **2.2** **开发板/Platform 特性分析**

1. 开发板商品名称 Orange Pi5 Plus

    是一款基于瑞芯微 RK3588 处理器的高性能单板计算机，适用于嵌入式系统开发、AI应用、边缘计算等多种场景。该开发板集成了丰富的接口和强大的计算能力，能够满足各种复杂应用的需求。
   <img src="img/Orange Pi5 Plus.png" alt="image-20250910160912933" style="zoom: 50%;" />

   - Orange Pi 5Plus 采用了瑞芯微 RK3588 八核64位处理器，具体为四核 A76+ 四核 A55，采用了 8nm 工艺设计，主频最高可达2.4GHz，集成 ARM Ma-li-G610,内置 3D GPU, 兼容 OpenGL ES1.1/2.0/3.2、OpenCL 2.2和Vulkan1.2;内嵌的NPU支持 INT4/INT8/INT16/FP16 混合运算，算力高达 6Tops，可以满足绝大多数终端设备的边缘计算需求; 有 4GB/8GB/16GBLPDDR4/4x 内存和 eMMC 闪存插座，可以外接 16GB/32GB/64GB/128GB/256GB eMMC 模块。Orange Pi 5Plus 支持 Orange Pi 官方研发的操作系统 Orange PiOS，同时，支持 Android12、Debian11、Ubuntu22.04 等操作系统。
   - Orange Pi 5Plus 具有丰富的接口，有2个 HDMI 输出端口,1个输入 HDMI 端口，最高可解码 8K@60P 视频，两个PCIe扩展的2.5G以太网接口，配备一个支持安装 NVMe 固态硬盘的 M.2 M-Key 插槽，一个支持 Wi-Fi6/BT 模块的 M.2 E-Key 插槽。此外，Orange Pi 5 Plus 有2个 USB 3.0、2个 USB 2.0、2个 Type-C(其中一个为电源接口)。Orange Pi 5Plus 具有广泛的用途，可以帮助嵌入式系统开发爱好者进行探索，也适用于企业开发具有多个以太网端口的迷你机器视觉系统，Orange Pi 5Plus为高端应用提供了更强性能体验，可满足不同行业的产品定制需求。

2. 采购地址

   [淘宝](https://item.taobao.com/item.htm?spm=a312a.7700824.w4004-23020891405.3.49108ec15qSG0u&id=718906164843&pisk=ga4EoOwwp426CuMKxPgy3bR6_F3Kl4WXtzMSZ7VoOvDndzwzZbc2Kemnq5ciaA9Id0wSalPYUv1Rv2ZZUR2ZKeaSR72agJrBdwHBa8P-C8T7Ozr9zSF8R63592FKy4XfhZ6b9W3-DP3cLykMSXFMqHmlfVcJm9w1hZ_b_McKP5XbPe1RlXhorXmoZNfZMjJorDYnS1loNpDoK8fN_jh2EbDk-CbiGAvntUx3shcSGUvoEvfZjAhnE40uEVfZwXH4EJLZQDfgd3qZxIssvxVnQUY2MXnHTW-WzUSotDDmj58lfPlEYxV3ushp1br0uDecCHM30owqGJ_M8-P3m8q0INTEz5q0zV4GICM3D5zrRyf5-VV0bSiU8t8IeJHuLynDZhm3DJZmbP6eWxqg_7iaX_KIFohuemU5HeH3ZSUYcq7yS2ol4gJ-s-u3ykJk4Dct_x1N_oOu5s5uhqEM23nNXfkfMsKJ2DYs_x1a13K-bsGZh62d).

3. 所需配置

   Orange Pi5 Plus（8G）主板+32G卡+Type-C 5V4A 中规电源

4. 对应价格

   900.9￥

## **第三部分 移植工作难点分析**

### **3.1 StarryOS** 内核当前支持情况

- StarryOS 目前支持的架构有 riscv64、loongarch64，rk3588 属于 aarch64 架构，当前aarch64没有完整支持，只包含EL1内核部分功能，需要实现 aarch64 架构的用户态切换功能。
- StarryOS 最小模式启动，需要实现的驱动包括 UART、GICv3(rk3588)、emmc等，目前只包含了 UART 驱动，其他驱动需要开发。
- StarryOS 目前没有支持 NPU 的驱动，需要开发 rk3588 内置 NPU 的驱动。
- 内核可以运行busybox，支持基本的文件系统操作，包含 ext4 等文件系统，需要移植RKNN等 NPU 用户态支持程序。
- StarryOS 目前没有支持 USB 驱动，需要开发 rk3588 内置 USB-host 的驱动。

### **3.2** **外设驱动开发**计划

给出你准备移植的三种外设驱动的类型，你的开发经验和驱动项目经历有哪些优势

准备移植电源驱动、时钟驱动、EMMC 驱动、USB 驱动、PCIe 驱动、NPU 驱动。

开发经验：团队长期从事Rust系统开发，熟悉各类外设驱动开发，具备丰富的驱动开发经验。曾使用Rust开发过多种操作系统无关的外设驱动，如：

- 串口驱动：pl011、ns16550
- 定时器驱动：arm generic timer
- 中断控制器驱动：GICv2、GICv3
- 存储驱动：飞腾emmc\sd卡驱动、rk3568 emmc\sd卡驱动
- 网络驱动：e1000e、igb(i210)
- PCIe驱动：x86 pcie
- USB驱动: xhci、uvc 摄像头

基于以上经验，团队有信心完成 rk3588 上的外设驱动开发任务。

### **3.3** **AI**加速单元开发计划

给出你准备开发的AI加速单元的类型，你的开发经验和驱动项目经历有哪些优势

准备开发 rk3588 内置 NPU 的驱动，团队成员有丰富的 pytorch、burn(rust) 等深度学习框架使用经验，熟悉 yolo-v8 模型的训练和部署，具备一定的模型优化能力。虽未接触过内核态的 NPU 驱动开发，但熟悉上层应用的使用方式，有助于了解驱动行为、设计驱动架构，有信心通过查阅资料和团队协作完成该任务。

## **第四部分 开发计划与里程碑**

### **4.1** **人员分工**

周睿：移植适配 rk3588 ，在 rk3588 开发板上启动 StarryOS

柏乔森：外设驱动开发，电源驱动、时钟驱动

杨金全：外设驱动开发，添加 USB 驱动、EMMC 驱动

宋志勇：外设驱动开发，添加 NPU 驱动、PCIe 驱动

赵长收：支持AI加速单元(NPU/TPU/BPU)，完成基于Yolo-v8模型进行目标识别等AI应用移植和算法优化

### **4.2** **开发环境/工具**

- linux：Ubuntu 20.04/22.04

- rustup 1.28.2 (e4f3ad6f8 2025-04-28)

- rustc 1.91.0-nightly (425a9c0a0 2025-08-17)

- make：用于编译U-Boot和内核

- aarch64-linux-gnu：用于将Rust代码编译为ARM架构的二进制文件

- vscode：安装 `rust-analyzer` 插件，提供智能补全、错误检查、调试支持

- dtc：用于编译设备树

### 4.3 开发计划

注：建议按照每周为一个里程碑，大赛组委会将会于决赛的五周内（9.9~10.12）于每周六晚组织大家进行讨论

#### 第一周（09.09~09.14）

1. 信息搜集，了解Starry-OS的组织结构和组件设计

2. 调研 RK3588 开发板硬件手册，确认：
   - NPU 物理接口和寄存器地址
   - 关键外设地址映射（如 UART、PM、CLK、EMMC）

3. 连接开发板，测试rknn等ai模块

#### 第二周（09.15~09.21）

- CPU 启动及初始化
- 异常与中断处理
- 内存管理单元（MMU）配置与页表管理
- 基本外设（UART）的初始化与驱动
- Aarch64 内核态与用户态切换机制

#### 第三周（09.22~09.28）

- 中断控制器（GIC-500）驱动适配。
- CLK 驱动适配。
- 电源管理（PMU）驱动适配。
- EMMC 驱动适配。

可加载文件系统，进入用户态命令行

#### 第四周（09.29~10.05）

- USB 驱动开发

  - 完成 RK3588 内置 USB-host 的寄存器配置与初始化
  - 实现基本的设备枚举与数据传输接口，支持 UVC 摄像头等常见设备

NPU 驱动开发

- 完成 RK3588 内置 NPU 的寄存器配置与初始化
- 实现基本的任务调度与数据传输接口，支持简单的推理任务

#### 第五周（10.06~10.12）

1. 完善优化代码，规范合理化，提高可读性
2. 输出开发指导文档和技术报告
3. 录制演示视频

## 第五部分 计划设计的 Rust 组件说明

说明计划设计的内核组件，每个组件的介绍，主要功能/API，架构等

### 5.1 内核相关组件

1. axplat-aarch64-dyn - aarch64 架构适配

### 5.2 驱动相关组件

1. power_driver

2. time_driver

3. emmc_driver

4. npu_driver

5. usb_driver

### 5.3 AI 应用相关组件

r-rknn - 用户态简易 NPU 调用库

## 第六部分 创新点与优势

说明本次参赛作品的创新点，分析团队成员完成此任务的优势等

1. 使用动态获取内存布局，将当前内存初始化由写死配置文件的方式，转化为通过设备树（arm riscv）或 acpi 动态读取的方式，物理内存地址和大小由 axconfig 定义，均为 const，调整为动态后，所有依赖项都需要由 const 转为动态。物理内存地址动态后，程序入口地址和物理内存地址的偏移 PHYS_VIRT_OFFSET 也需要改为动态，并且 kernel code 与 mmio 的 offset 是不同的，需要引入 linux 类似的内存分区，不同类型的地址需要采用不同的映射方式。
2. 开发 ostool 工具，更加方便通过 U-Boot 在 rk3588 上启动 Starry，具体来说通过一次性配置 `.peroject.toml` 文件中的编译指令，内核名称等信息，执行 `ostool run uboot` 指令，自动将编译后的系统镜像通过串口发送到 rk3568 开发板上，而避免了每次手动烧录或者通过 tftp 传送的繁琐。
3. 团队成员在 arceos 以及 axvisor 项目中具有一定技术知识储备，在平台适配和驱动开发上积累过经验

## 第七部分 预期成果

说明本次参赛作品的预期达成的成果，包括提交的代码，文档，带动的课程，项目等

本次参赛预期实现在 rk3588 开发板上启动 StarryOS，并覆盖 时钟管理驱动、电源管理驱动、EMMC驱动、 NPU 驱动、USB 驱动，支持AI加速单元，并通过硬件加速接口实现AI模型推理；完成基于 Yolo-v8 模型进行目标识别等AI应用移植和算法优化，最终完成系统集成和场景验证。

提交代码包括

输出文档
