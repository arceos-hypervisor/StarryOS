# 基于国产AI芯片的Rust内核组件开发赛

队伍名称：泉城把子肉

队伍成员：赵长收，周睿，柏乔森，杨金全，宋志勇

## **第一部分 项目概述**

基于 Rust 编程语言，完善 StarryOS 组件，并在基于 RK3588 开发板上移植启动，最终实现基于 RK3588 内置 NPU 运行 Yolo-v8 模型，实现流程分为以下四个阶段：

1. 添加电源驱动、时钟驱动 、NPU 驱动、PCIe 驱动、USB 驱动、SPI 驱动
2. 添加必要的 StarryOS 组件
3. 移植适配 RK3588 开发板
4. 移植适配 Yolo-v8 模型

## **第二**部分 硬件平台分析

### **2.1** **芯片/SoC特性分析**

给出你所选用芯片的型号，内置外设和AI加速单元的情况

1. 型号选择：

​	Orange Pi5 Plus

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

   <img src="img/Orange Pi5 Plus.png" alt="image-20250910160912933" style="zoom: 50%;" /> 

- Orange Pi 5Plus 采用了瑞芯微 RK3588 八核64位处理器，具体为四核 A76+ 四核 A55，采用了 8nm 工艺设计，主频最高可达2.4GHz，集成 ARM Ma-li-G610,内置 3D GPU, 兼容 OpenGL ES1.1/2.0/3.2、OpenCL 2.2和Vulkan1.2;内嵌的NPU支持 INT4/INT8/INT16/FP16 混合运算，算力高达 6Tops，可以满足绝大多数终端设备的边缘计算需求; 有 4GB/8GB/16GBLPDDR4/4x 内存和 eMMC 闪存插座，可以外接 16GB/32GB/64GB/128GB/256GB eMMC 模块。Orange Pi 5Plus 支持 Orange Pi 官方研发的操作系统 Orange PiOS，同时，支持 Android12、Debian11、Ubuntu22.04 等操作系统。
- Orange Pi 5Plus 具有丰富的接口，有2个 HDMI 输出端口,1个输入 HDMI 端口，最高可解码 8K@60P 视频，两个PCIe扩展的2.5G以太网接口，配备一个支持安装 NVMe 固态硬盘的 M.2 M-Key 插槽，一个支持 Wi-Fi6/BT 模块的 M.2 E-Key 插槽。此外，Orange Pi 5 Plus 有2个 USB 3.0、2个 USB 2.0、2个 Type-C(其中一个为电源接口)。Orange Pi 5Plus 具有广泛的用途，可以帮助嵌入式系统开发爱好者进行探索，也适用于企业开发具有多个以太网端口的迷你机器视觉系统，Orange Pi 5Plus为高端应用提供了更强性能体验，可满足不同行业的产品定制需求。

2. 采购地址

   https://item.taobao.com/item.htm?spm=a312a.7700824.w4004-23020891405.3.49108ec15qSG0u&id=718906164843&pisk=ga4EoOwwp426CuMKxPgy3bR6_F3Kl4WXtzMSZ7VoOvDndzwzZbc2Kemnq5ciaA9Id0wSalPYUv1Rv2ZZUR2ZKeaSR72agJrBdwHBa8P-C8T7Ozr9zSF8R63592FKy4XfhZ6b9W3-DP3cLykMSXFMqHmlfVcJm9w1hZ_b_McKP5XbPe1RlXhorXmoZNfZMjJorDYnS1loNpDoK8fN_jh2EbDk-CbiGAvntUx3shcSGUvoEvfZjAhnE40uEVfZwXH4EJLZQDfgd3qZxIssvxVnQUY2MXnHTW-WzUSotDDmj58lfPlEYxV3ushp1br0uDecCHM30owqGJ_M8-P3m8q0INTEz5q0zV4GICM3D5zrRyf5-VV0bSiU8t8IeJHuLynDZhm3DJZmbP6eWxqg_7iaX_KIFohuemU5HeH3ZSUYcq7yS2ol4gJ-s-u3ykJk4Dct_x1N_oOu5s5uhqEM23nNXfkfMsKJ2DYs_x1a13K-bsGZh62d.

3. 所需配置

   Orange Pi5 Plus（8G）主板+32G卡+Type-C 5V4A 中规电源

4. 对应价格

   900.9￥

## **第三部分 移植工作难点分析**

### **3.1 StarryOS** 内核当前支持情况

分析当前 StarryOS 内核对该芯片的支持情况，是否可以启动，支持的外设驱动有哪些

分析 Starry-OS/axplat_crates 的 platforms，目前暂不支持 rk3588 平台的启动

分析 Starry-OS/axdriver_crates 的相关驱动，目前暂不支持 rk3588 平台的驱动



### **3.2** **外设驱动开发**计划

给出你准备移植的三种外设驱动的类型，你的开发经验和驱动项目经历有哪些优势

准备移植电源驱动、时钟驱动、NPU 驱动、PCIe 驱动、USB 驱动、SPI 驱动



### **3.3** **AI**加速单元开发计划

给出你准备开发的AI加速单元的类型，你的开发经验和驱动项目经历有哪些优势

集成瑞芯微 RK3588 内置 NPU（神经网络处理单元）的算力调度接口，实现：
1.Rust 语言层与 NPU 的通信交互，支持模型加载、推理任务提交与结果回传；
2.内核级任务调度机制优化，确保 CPU 与 NPU 的协同计算效率，降低任务切换开销。

在此项目中，我们的开发经验和以往的项目经历将带来以下优势：

1. Rust 编程语言经验：

   我们在多个项目中深度使用 Rust，尤其是在高并发、低延迟的系统开发中积累了大量经验。Rust 的内存安全性和并发性特性将极大提高与 NPU 的通信交互效率。Rust 在实现高性能任务调度和多线程处理时，相比传统的 C/C++，具有更强的安全性和更少的运行时错误。

2. 嵌入式系统开发经验：

   我们在嵌入式系统和边缘计算领域有丰富的开发经验，特别是在嵌入式硬件加速和 AI 推理任务的实现上。熟悉不同硬件平台和内存管理技巧，能有效应对有限资源环境下的优化需求。

3. 驱动开发与硬件接口经验：

   在过去的项目中，我们参与与硬件相关的底层驱动开发工作，涉及硬件加速模块（如 GPU、NPU）的驱动编写。

4. 多线程与并行计算优化经验：

   我们在多个项目中，设计和实现了基于 Rust 的并行计算框架，并通过精细的任务调度、负载均衡和线程同步优化，实现了高效的并行计算。在 AI 加速的背景下，我能够快速为项目开发出适合的任务调度机制，确保 CPU 和 NPU 资源的高效利用。

5. 内核调度与系统优化能力：

   我们在多核处理器系统下的内核级任务调度优化有丰富的经验。曾在多个嵌入式项目中，通过自定义调度算法、优化中断处理、降低上下文切换频率等手段，有效提升了任务执行效率。这些经验可以帮助我在本项目中优化任务调度机制，减少任务切换的延迟，提升协同计算的效率。

 

## **第四部分 开发计划与里程碑**

### **4.1** **人员分工**

赵长收：支持AI加速单元(NPU/TPU/BPU)，完成基于 Yolo-v8 模型进行目标识别等AI应用移植和算法优化

周睿：移植适配 rk3588 ，在 AIO-3588JD4 开发板上启动 StarryOS

柏乔森：外设驱动开发，添加电源驱动、时钟驱动

杨金全：外设驱动开发，添加 USB 驱动、SPI 驱动

宋志勇：外设驱动开发，添加 NPU 驱动、PCIe 驱动

### **4.2** **开发环境/工具**

- linux：Ubuntu 24.04

- rustup 1.28.2 (e4f3ad6f8 2025-04-28)

- rustc 1.91.0-nightly (425a9c0a0 2025-08-17)

- make：用于编译U-Boot和内核

- aarch64-linux-gnu：用于将Rust代码编译为ARM架构的二进制文件

- rust-objdump：分析和反汇编 Rust 编译后的二进制文件

- vscode：安装 `rust-analyzer` 插件，提供智能补全、错误检查、调试支持

- dtc：用于编译设备树

### **4.3** **开发计划**

注：建议按照每周为一个里程碑，大赛组委会将会于决赛的五周内（9.9~10.12）于每周六晚组织大家进行讨论

**第一周（09.09~09.14）**

1. 信息搜集，了解Starry-OS的组织结构和组件设计

2. 调研 RK3588 开发板硬件手册，确认：
   - NPU 物理接口（PCIe/USB/SPI）
   - 关键外设地址映射（如 UART、GPIO、DMA）

3. 连接开发板，测试基础通信等功能

**第二周（09.15~09.21）**

1. StarryOS 内核组件底层支撑
   - CPU 核心（四核 Cortex-A76 + 四核 Cortex-A55 异构架构）的调度与运行控制
   - 内存管理单元（MMU）的地址映射与内存保护机制

2. 驱动开发与组件适配
   - 电源驱动
   - USB 驱动
   - NPU 驱动

**第三周（09.22~09.28）**

1. StarryOS 内核组件移植适配
   - 中断控制器（GIC-500）的中断响应与优先级管理，确保硬件事件的实时处理。
   - 调试测试在 AIO-3588JD4 启动运行Starry-OS

2. 驱动开发与组件适配
   - 时钟驱动
   - SPI 驱动
   - PCIe 驱动

3. Yolo-v8 模型移植与优化
   - 模型转换，将 Yolo-v8 转换为 ONNX 或 RKNN 格式
   - 推理框架适配，在 StarryOS 中实现推理接口

**第四周（09.29~10.05）**

1. 针对 RK3588 的 NPU 架构特性，对模型进行量化与算子优化，提升推理效率
2. 系统集成与场景验证
   - 完成内核组件、外设驱动、AI 应用的全系统集成，在 RK3588 开发板上实现 StarryOS 的完整启动与功能运行
   - 通过场景化测试验证，实现从图像采集、目标识别到执行器响应的全流程闭环。

**第五周（10.06~10.12）**

1. 完善优化代码，规范合理化，提高可读性
2. 输出开发指导文档和技术报告
3. 录制演示视频

## **第五部分 计划设计的 Rust 组件说明**

说明计划设计的内核组件，每个组件的介绍，主要功能/API，架构等

### **5.1** **内核相关组件**

1. axplat-aarch64-dyn

   组件介绍：axplat-aarch64-dyn 是一个动态内核平台抽象层，旨在为不同硬件架构提供抽象接口。它支持平台的动态加载，根据提供对应平台的设备树，解析设备树的相关节点选择平台所需的支持，增强了系统的可移植性

   主要功能：
   - 动态获取内存布局，将当前内存初始化由写死配置文件的方式，转化为通过设备树（arm riscv）或 acpi 动态读取的方式。
   - 提供跨平台的硬件接口抽象，简化底层硬件操作。

   API：
   - init_early： 在 early 阶段为主核心初始化平台
   - init_early_secondary： 在 early 阶段为辅助核心初始化平台
   - init_later： 在 later 阶段为主核心初始化平台
   - init_later_secondary： 在 later 阶段为辅助核心初始化平台
   - write_bytes： 将给定字节写入控制台
   - read_bytes： 从控制台读取字节到给定的可变片
   - phys_ram_ranges： 返回平台上的所有物理内存（RAM）范围
   - reserved_phys_ram_ranges： 返回平台上所有保留的物理内存范围
   - mmio_ranges： 返回平台上的所有设备内存（MMIO）范围
   - phys_to_virt： 物理地址转换为虚拟地址
   - virt_to_phys： 虚拟地址转换为物理地址
   - cpu_boot： 用给定的初始堆栈引导给定的CPU内核（在物理地址中）
   - system_off： 关闭整个系统
   - current_ticks： 以硬件刻度返回当前时钟时间
   - ticks_to_nanos： 将硬件刻度转换为纳秒
   - nanos_to_ticks：将纳秒转换为硬件刻度
   - epochoffset_nanos：返回以纳秒为单位的历元偏移量（到单调时钟开始的壁时间偏移量）
   - set_oneshot_timer：设置一次计时器，计时器中断将在指定的单调时间截止日期（以纳秒为单位）触发


### **5.2** **驱动相关组件**

1. power_driver

   组件介绍：power_driver 是一个电源管理驱动，负责控制系统的电源管理策略，包括电源的开启、关闭、休眠和唤醒等功能。

   主要功能：电源状态管理，根据系统需求管理电源的开关。

   API：
   - power_on 开启系统电源。
   - power_off 关闭系统电源。

2. time_driver

   组件介绍：time_driver 提供精确的时间管理服务，包括时钟同步、时间延迟控制和定时任务调度等。

   主要功能：
   - 时钟同步：确保系统时间与硬件时钟同步。
   - 定时器：实现定时任务调度和时间延迟功能。
   - 时间获取：提供系统时间接口，支持获取当前的时间戳。

   API：
   - set_system_time：设置系统时间
   - get_system_time：获取当前系统时间

3. npu_driver

   组件介绍：npu_driver 是一个专门针对神经网络处理单元（NPU）的驱动，负责与硬件交互，提供加速计算能力。

   主要功能：
   - 神经网络加速：利用 NPU 提供的硬件加速来加速深度学习计算
   - 内存管理：为 NPU 分配和释放内存，确保高效的数据传输
   API：
   - initialize_npu：初始化 NPU 硬件
   - query_status：查询 NPU 状态

4. pcie_driver

   组件介绍：pcie_driver 是一个用于管理 PCIe 总线设备的驱动，支持与各种 PCIe 设备的交互，如网络适配器、存储设备等

   主要功能：
   - 设备识别：识别并初始化连接的 PCIe 设备
   - 数据传输：提供高效的 PCIe 数据传输接口
   - 错误处理：处理 PCIe 设备的错误和异常。

   API：
   - alloc_memory32：在低 4 GB 里分配 32 位地址
   - alloc_memory64：在 64 位空间里分配 64 位地址
   - read：用‘ offset ’在‘ address ’处执行PCI读取操作
   - write：用‘ offset ’在‘ address ’处执行PCI写入操作
   - fmt：格式化内容
   - read_bar：读取与某个 PCI 设备相关的 BAR 地址
   - address：获取 PCI 设备的地址信息
   - header_type：返回 PCI 设备的头部类型

5. usb_driver

   组件介绍：usb_driver 提供对 USB 设备的支持

   主要功能：
   - 设备识别：自动识别和初始化连接的 USB 设备。
   - 数据传输：提供 USB 设备的数据传输接口

   API：
   - initialize_usb_device：初始化 USB 设备
   - send_data：向 USB 设备发送数据
   - get_device_status：查询 USB 设备的状态


6. spi_driver

   组件介绍：spi_driver 是一个用于与 SPI（串行外设接口）设备交互的驱动

   主要功能：
   - 设备初始化：初始化 SPI 设备和配置通信参数
   - 数据交换：与 SPI 设备进行数据读写操作。

   API：
   - initialize_spi_device：初始化 SPI 设备并设置通信参数
   - send_spi_data：向 SPI 设备发送数据
   - read_spi_data：从 SPI 设备读取数据


### **5.3** **AI 应用相关组件**

 1.ai_acc
 
   组件介绍：ai_acc 一个与 AI 计算加速相关的模块，旨在为ai应用提供高效的硬件加速支持

   主要功能：
   - 硬件加速：利用 GPU、TPU、NPU 等硬件进行 AI 模型的训练和推理加速
   - 支持 Yolo-v8 模型
   - 管理和优化用于 AI 运算的内存资源，减少内存占用，提高数据传输效率

   API：
   - init_accelerator：初始化 AI 加速硬件设备，如 GPU 或 NPU
   - accelerate_model：将指定的 AI 模型传递给硬件加速器进行加速推理
   - get_available_devices：获取可用的加速硬件设备列表
   - get_error_logs：获取硬件加速过程中出现的错误日志
   - clear_error_logs ：清除错误日志

## **第六部分 创新点与优势**

说明本次参赛作品的创新点，分析团队成员完成此任务的优势等

1. 使用动态获取内存布局，将当前内存初始化由写死配置文件的方式，转化为通过设备树（arm riscv）或 acpi 动态读取的方式，物理内存地址和大小由 axconfig 定义，均为 const，调整为动态后，所有依赖项都需要由 const 转为动态。物理内存地址动态后，程序入口地址和物理内存地址的偏移 PHYS_VIRT_OFFSET 也需要改为动态，并且 kernel code 与 mmio 的 offset 是不同的，需要引入 linux 类似的内存分区，不同类型的地址需要采用不同的映射方式。
2. 开发 ostool 工具，更加方便通过 U-Boot 在 rk3588 上启动 Starry，具体来说通过一次性配置 `.peroject.toml` 文件中的编译指令，内核名称等信息，执行 `ostool run uboot` 指令，自动将编译后的系统镜像通过串口发送到 rk3568 开发板上，而避免了每次手动烧录或者通过 tftp 传送的繁琐。
3. 团队成员在 arceos 以及 axvisor 项目中具有一定技术知识储备，在平台适配和驱动开发上积累过经验

##  

## **第七部分 预期成果**

说明本次参赛作品的预期达成的成果，包括提交的代码，文档，带动的课程，项目等

本次参赛预期实现
- 在 aio-rk3568jd 开发板上启动 StarryOS
- 并覆盖电源驱动、时钟驱动、NPU 驱动、PCIe 驱动、USB 驱动、SPI 驱动
- 支持AI加速单元，并通过硬件加速接口实现AI模型推理
- 完成基于 Yolo-v8 模型进行目标识别等AI应用移植和算法优化，最终完成系统集成和场景验证

提交代码包括 
- 适配rk3588的动态平台代码
- 电源驱动、时钟驱动、NPU 驱动、PCIe 驱动、USB 驱动、SPI 驱动的驱动代码
- AI加速单元的实现代码
- 适配 Yolo-v8 模型应用移植的系统调用或相关支持代码

输出文档
- 所有代码仓库中的 README
- 实现本次参赛作品成果的总体说明文档，包括实现的功能说明，代码仓库链接，实现效果展示等