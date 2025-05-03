# 移植《STM32F103 TFT开发板综合测试程序-京东方玻璃》到`embassy-stm32`

## 项目概述

本项目是将《STM32F103 TFT开发板综合测试程序-京东方玻璃》移植到基于`Rust`的`embassy-stm32`框架上的实现。该项目使用`Rust`语言重写了原有的C语言程序，展示了如何在嵌入式`Rust`中操作`LCD`屏幕和`SPI FLASH`。

## 开发板介绍

`STM32F103 TFT`开发板兼容`stm32f103c8t6`小蓝板，但额外扩展了以下功能：

- `LCD`屏幕（京东方玻璃）
- `SPI FLASH`存储器
- 串口芯片

开发板资料链接：[STM32F103 TFT开发板资料](https://product.abrobot.club/ABrobot%E4%BA%A7%E5%93%81%E8%B5%84%E6%96%99%E4%B8%AD%E5%BF%83/STM32F103%20TFT%E5%BC%80%E5%8F%91%E6%9D%BF%E8%B5%84%E6%96%99)

## 硬件连接

### LCD屏幕引脚连接表

| 功能 | STM32引脚 | 说明 |
|------|----------|------|
| `CS`   | `PB7`      | 片选信号线 |
| `RES`  | `PB6`      | 复位信号线 |
| `BLK`  | `PB8`      | 背光控制 |
| `DC`   | `PB4`      | 数据/命令选择信号线 |
| `SCK`  | `PB3`      | SPI时钟线 |
| `MOSI` | `PB5`      | SPI主机输出 |

`LCD`屏幕采用`SPI`接口进行通信，屏幕分辨率为`160×80`像素（横向模式）。

### SPI FLASH引脚连接表

| 功能 | STM32引脚 | 说明 |
|------|----------|------|
| `CS`   | `PB12`     | 片选信号线 |
| `SCK`  | `PB13`     | SPI时钟线 |
| `MOSI` | `PB15`     | SPI主机输出 |
| `MISO` | `PB14`     | SPI主机输入 |

`SPI FLASH`支持`W25Q80`、`W25Q16`、`W25Q32`、`W25Q64`等多种型号，通过`SPI`接口进行通信。

## 功能特性

1. **LCD显示功能**：
   - 文本显示（`ASCII`和中文）
   - 图形绘制（线条、矩形、圆形）
   - 图片显示
   - 多种颜色支持

2. **SPI FLASH操作**：
   - 读取芯片`ID`
   - 擦除扇区
   - 读写数据

3. **系统功能**：
   - `LED`闪烁
   - 串口通信

## 代码结构

- `src/main.rs` - 程序入口，初始化设备并测试功能
- `src/lcd.rs` - `LCD`驱动实现
- `src/lcd/font.rs` - 字体数据
- `src/lcd/pic.rs` - 图片数据
- `src/w25qxx.rs` - `SPI FLASH`驱动实现

## 使用`embassy-stm32`的优势

1. 安全的`Rust`语言实现，避免常见的内存安全问题
2. 基于`async/await`的异步架构，提高代码可读性和资源利用
3. 类型安全的外设访问，减少配置错误
4. 使用`embassy`生态系统，简化嵌入式开发

## 示例功能

本项目实现了以下测试功能：

1. 初始化并测试`LCD`显示
2. 读取`FLASH ID`并显示
3. 测试`FLASH`读写操作
4. `LED`指示灯闪烁

## 构建与烧录

本项目使用`Rust`和`Cargo`构建系统。确保您已安装`Rust`工具链和对应的目标平台支持：

```bash
# 安装rustup（如果尚未安装）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 添加Cortex-M3目标
rustup target add thumbv7m-none-eabi

# 构建项目
cargo build --release

# 烧录并运行
cargo install probe-rs
cargo run --release
```

## 许可证

MulanPSL-2.0
