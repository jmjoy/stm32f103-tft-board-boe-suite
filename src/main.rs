#![no_std]
#![no_main]

mod lcd;
mod w25qxx;

use {defmt_rtt as _, panic_probe as _};
use embassy_stm32::{
    bind_interrupts, flash, gpio::{Level, Output, Speed}, pac::{self}, peripherals, rcc::{
        AHBPrescaler, APBPrescaler, Hse, HseMode, LsConfig, Pll, PllMul, PllPreDiv, PllSource,
        Sysclk
    }, spi::{self, Spi}, time::Hertz, usart::{self, Uart}, Config
};
use embassy_executor::Spawner;
use defmt::{error, info, warn};
use embassy_time::Timer;
use lcd::LCD;
use w25qxx::{W25QxxID, W25Qxx};
use num_enum::TryFromPrimitive;

bind_interrupts!(struct Irqs {
    USART1 => usart::InterruptHandler<peripherals::USART1>;
});

fn make_config() -> Config {
    let mut config = Config::default();
    config.rcc.hsi = true;
    config.rcc.hse = Some(Hse {
        freq: Hertz::mhz(8),
        mode: HseMode::Oscillator,
    });
    config.rcc.pll = Some(Pll {
        src: PllSource::HSE,
        prediv: PllPreDiv::DIV1,
        mul: PllMul::MUL9,
    });
    config.rcc.sys = Sysclk::PLL1_P;
    config.rcc.ahb_pre = AHBPrescaler::DIV1;
    config.rcc.apb1_pre = APBPrescaler::DIV2;
    config.rcc.apb2_pre = APBPrescaler::DIV1;
    config.rcc.ls = LsConfig::default_lse();
    config
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(make_config());

    pac::AFIO.mapr().modify(|w| {
        w.set_swj_cfg(0b0000_0010); // this is equal to __HAL_AFIO_REMAP_SWJ_NOJTAG() in C
        w.set_spi1_remap(true);
    });

    info!("stm32f103-tft-board-boe-suite started!");

    let led = Output::new(p.PC13, Level::High, Speed::VeryHigh);
    spawner.spawn(handle_led(led)).unwrap();

    let mut uart1 = Uart::new_blocking(p.USART1, p.PA10, p.PA9, Default::default()).unwrap();
    uart1.blocking_write(b"stm32f103-tft-board-boe-suite started!\r\n").unwrap();

    // LCD
    let lcd_spi_config: spi::Config = Default::default();
    let lcd_spi = Spi::new_txonly(p.SPI1, p.PB3, p.PB5, p.DMA1_CH3, lcd_spi_config);
    let lcd_cs = Output::new(p.PB7, Level::Low, Speed::VeryHigh);
    let lcd_res = Output::new(p.PB6, Level::Low, Speed::VeryHigh);
    let lcd_blk = Output::new(p.PB8, Level::Low, Speed::VeryHigh);
    let lcd_dc = Output::new(p.PB4, Level::Low, Speed::VeryHigh);

    let mut lcd = LCD::new(lcd_spi, lcd_cs, lcd_res, lcd_blk, lcd_dc);
    lcd.init().await;
    lcd.fill(0, 0, lcd::WIDTH, lcd::HEIGHT, lcd::Color::White as u16).await;

    // SPI Flash
    let mut w25qxx_spi_config: spi::Config = Default::default();
    w25qxx_spi_config.mode = spi::MODE_3;
    let w25qxx_spi = Spi::new(p.SPI2, p.PB13, p.PB15, p.PB14, p.DMA1_CH5, p.DMA1_CH4, w25qxx_spi_config);
    let w25qxx_cs = Output::new(p.PB12, Level::High, Speed::VeryHigh);

    let mut w25qxx = W25Qxx::new(w25qxx_spi, w25qxx_cs);
    let device_id = w25qxx.read_device_id().await;
    let flash_id = w25qxx.read_id().await;
    info!("FlashID is 0x{:x}, Device ID is 0x{:x}", flash_id, device_id);

    let mut flash_size = 0u8;

    match W25QxxID::try_from_primitive(flash_id) {
        Ok(flash_id) => {
            match flash_id {
                W25QxxID::W25Q16 => {
                    info!("flash芯片型号为W25Q16!");
                    flash_size = 2;
                }
                W25QxxID::W25Q32 => {
                    info!("flash芯片型号为W25Q32!");
                    flash_size = 4;
                }
                W25QxxID::W25Q64 => {
                    info!("flash芯片型号为W25Q64!");
                    flash_size = 4;
                },
                W25QxxID::W25Q80 => {
                    info!("flash芯片型号为W25Q80!");
                    flash_size = 1;
                },
            }

            const FLASH_ADDRESS: u32 = 0x00000;
            const TX_BUFFER: &[u8] = b"123456";

            w25qxx.sector_erase(FLASH_ADDRESS).await;
            w25qxx.buffer_write(TX_BUFFER, FLASH_ADDRESS).await;
            info!("写入的数据为: {}", TX_BUFFER);

            let rx_buffer = &mut [0u8; 6];
            w25qxx.buffer_read(rx_buffer, FLASH_ADDRESS).await;
            info!("读出的数据为: {}", rx_buffer);

            if TX_BUFFER == rx_buffer {
                info!("串行flash测试成功!");
                uart1.blocking_write(b"serial flash test success!\r\n").unwrap();
            } else {
                error!("flash测试失败!");
            }
        }
        Err(_) => {
            warn!("获取不到 W25Qxx ID");
        }
    }

    loop {
        Timer::after_millis(500).await;
    }
}

#[embassy_executor::task]
async fn handle_led(mut led: Output<'static>) {
    loop {
        led.set_high();
        Timer::after_millis(500).await;
        led.set_low();
        Timer::after_millis(500).await;
    }
}
