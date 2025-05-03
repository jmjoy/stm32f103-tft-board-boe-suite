use embassy_stm32::{gpio::{Level, Output}, mode, spi::Spi};
use embassy_time::Timer;

pub struct LCD {
    spi: Spi<'static, mode::Async>,
    cs: Output<'static>,
    res: Output<'static>,
    blk: Output<'static>,
    dc: Output<'static>,
}

impl LCD {
    pub fn new(spi: Spi<'static, mode::Async>, cs: Output<'static>, res: Output<'static>, blk: Output<'static>, dc: Output<'static>) -> Self {
        Self { spi, cs, res, blk, dc }
    }

    pub async fn init(&mut self) {
        self.res.set_low();
        Timer::after_millis(100).await;
        self.res.set_high();
        Timer::after_millis(100).await;

        //打开背光
        self.blk.set_high();
        Timer::after_millis(100).await;

        //Sleep exit
        self.write_reg(&[0x11]).await; 
        Timer::after_millis(120).await;
        self.write_reg(&[0xb1]).await;
        self.write_data(&[0x05, 0x3c, 0x3c]).await;

        self.write_reg(&[0xb2]).await;
        self.write_data(&[0x05, 0x3c, 0x3c]).await;

        self.write_reg(&[0xb3]).await;
        self.write_data(&[0x05, 0x3c, 0x3c, 0x05, 0x3c, 0x3c]).await;

        self.write_reg(&[0xb4]).await; // Dot inversion
        self.write_data(&[0x03]).await;

        self.write_reg(&[0xc0]).await;
        self.write_data(&[0x0e, 0x0e, 0x04]).await;

        self.write_reg(&[0xc1]).await;
        self.write_data(&[0xc5]).await;

        self.write_reg(&[0xc2]).await;
        self.write_data(&[0x0d, 0x00]).await;

        self.write_reg(&[0xc3]).await;
        self.write_data(&[0x8d, 0x2a]).await;

        self.write_reg(&[0xc4]).await;
        self.write_data(&[0x8d, 0xee]).await;

        self.write_reg(&[0xc5]).await; // VCOM
        self.write_data(&[0x06]).await;

        self.write_reg(&[0x36]).await; // MX, MY, RGB mode
        // Note: The USE_HORIZONTAL value needs to be defined or passed as a parameter
        // For now, let's default to option 0
        self.write_data(&[0x08]).await;
        
        self.write_reg(&[0x3a]).await;
        self.write_data(&[0x55]).await;

        self.write_reg(&[0xe0]).await;
        self.write_data(&[
            0x0b, 0x17, 0x0a, 0x0d, 0x1a, 0x19, 0x16, 0x1d,
            0x21, 0x26, 0x37, 0x3c, 0x00, 0x09, 0x05, 0x10,
        ]).await;

        self.write_reg(&[0xe1]).await;
        self.write_data(&[
            0x0c, 0x19, 0x09, 0x0d, 0x1b, 0x19, 0x15, 0x1d,
            0x21, 0x26, 0x39, 0x3e, 0x00, 0x09, 0x05, 0x10,
        ]).await;

        Timer::after_millis(120).await;
        self.write_reg(&[0x29]).await; // Display on
    }

    async fn write_reg(&mut self, data: &[u8]) {
        self.dc.set_low(); // write command
        self.write_bus(data).await;
        self.dc.set_high(); // write data
    }

    async fn write_data(&mut self, data: &[u8]) {
        self.write_bus(data).await;
    }

    async fn write_bus(&mut self, data: &[u8]) {
        self.cs.set_low();
        self.spi.write(data).await.unwrap();
        self.cs.set_high();
    }
}
