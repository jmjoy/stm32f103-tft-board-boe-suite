use embassy_stm32::{gpio::{Level, Output}, mode, spi::Spi};
use embassy_time::Timer;

pub const DIRECTION: Direction = Direction::Horizontal1;

pub const WIDTH: u16 = match DIRECTION {
    Direction::Vertical0 | Direction::Vertical1 => 80,
    Direction::Horizontal0 | Direction::Horizontal1 => 160,
};

pub const HEIGHT: u16 = match DIRECTION {
    Direction::Vertical0 | Direction::Vertical1 => 160,
    Direction::Horizontal0 | Direction::Horizontal1 => 80,
};

#[repr(u8)]
pub enum Direction {
    Vertical0 = 0,
    Vertical1 = 1,
    Horizontal0 = 2,
    Horizontal1 = 3,
}

#[repr(u16)]
pub enum Color {
    White = 0xFFFF,
    Black = 0x0000,
    Blue = 0x001F,
    Gblue = 0x07FF,
    Red = 0xF800,
    Magenta = 0xF81F,
    Green = 0x07E0,
    Cyan = 0x7FFF,
    Yellow = 0xFFE0,
    Brown = 0xBC40,
    Brred = 0xFC07,
    Gray = 0x8430,
    Darkblue = 0x01CF,
    Lightblue = 0x7D7C,
    Grayblue = 0x5458,
    Lightgreen = 0x841F,
    Lgray = 0xC618,
    Lgrayblue = 0xA651,
    Lbblue = 0x2B12,
}

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
        self.write_data8(&[0x05, 0x3c, 0x3c]).await;

        self.write_reg(&[0xb2]).await;
        self.write_data8(&[0x05, 0x3c, 0x3c]).await;

        self.write_reg(&[0xb3]).await;
        self.write_data8(&[0x05, 0x3c, 0x3c, 0x05, 0x3c, 0x3c]).await;

        self.write_reg(&[0xb4]).await; // Dot inversion
        self.write_data8(&[0x03]).await;

        self.write_reg(&[0xc0]).await;
        self.write_data8(&[0x0e, 0x0e, 0x04]).await;

        self.write_reg(&[0xc1]).await;
        self.write_data8(&[0xc5]).await;

        self.write_reg(&[0xc2]).await;
        self.write_data8(&[0x0d, 0x00]).await;

        self.write_reg(&[0xc3]).await;
        self.write_data8(&[0x8d, 0x2a]).await;

        self.write_reg(&[0xc4]).await;
        self.write_data8(&[0x8d, 0xee]).await;

        self.write_reg(&[0xc5]).await; // VCOM
        self.write_data8(&[0x06]).await;

        self.write_reg(&[0x36]).await; // MX, MY, RGB mode
        match DIRECTION {
            Direction::Vertical0 => self.write_data8(&[0x08]).await,
            Direction::Vertical1 => self.write_data8(&[0xc8]).await,
            Direction::Horizontal0 => self.write_data8(&[0x78]).await,
            Direction::Horizontal1 => self.write_data8(&[0xa8]).await,
        }
        
        self.write_reg(&[0x3a]).await;
        self.write_data8(&[0x55]).await;

        self.write_reg(&[0xe0]).await;
        self.write_data8(&[
            0x0b, 0x17, 0x0a, 0x0d, 0x1a, 0x19, 0x16, 0x1d,
            0x21, 0x26, 0x37, 0x3c, 0x00, 0x09, 0x05, 0x10,
        ]).await;

        self.write_reg(&[0xe1]).await;
        self.write_data8(&[
            0x0c, 0x19, 0x09, 0x0d, 0x1b, 0x19, 0x15, 0x1d,
            0x21, 0x26, 0x39, 0x3e, 0x00, 0x09, 0x05, 0x10,
        ]).await;

        Timer::after_millis(120).await;
        self.write_reg(&[0x29]).await; // Display on
    }

    pub async fn fill(&mut self, x_start: u16, y_start: u16, x_end: u16, y_end: u16, color: u16) {
        // Set the display address range
        self.set_address(x_start, y_start, x_end - 1, y_end - 1).await;
        
        // Fill the screen pixel by pixel
        for _ in y_start..y_end {
            for _ in x_start..x_end {
                // For each pixel, write the color data
                self.write_data(&[color]).await;
            }
        }
    }

    async fn write_reg(&mut self, data: &[u8]) {
        self.dc.set_low(); // write command
        self.write_bus(data).await;
        self.dc.set_high(); // write data
    }

    async fn write_data8(&mut self, data: &[u8]) {
        self.write_bus(data).await;
    }

    async fn write_bus(&mut self, data: &[u8]) {
        self.cs.set_low();
        self.spi.write(data).await.unwrap();
        self.cs.set_high();
    }

    async fn write_data(&mut self, data: &[u16]) {
        self.cs.set_low();
        self.spi.write(data).await.unwrap();
        self.cs.set_high();
    }

    async fn set_address(&mut self, x1: u16, y1: u16, x2: u16, y2: u16) {
        match DIRECTION {
            Direction::Vertical0 | Direction::Vertical1 => {
                // Column address set
                self.write_reg(&[0x2a]).await;
                self.write_data(&[x1 + 24, x2 + 24]).await;
                
                // Row address set
                self.write_reg(&[0x2b]).await;
                self.write_data(&[y1, y2]).await;
            },
            Direction::Horizontal0 | Direction::Horizontal1 => {
                // Column address set
                self.write_reg(&[0x2a]).await;
                self.write_data(&[x1, x2]).await;
                
                // Row address set
                self.write_reg(&[0x2b]).await;
                self.write_data(&[y1 + 24, y2 + 24]).await;
            },
        }
        
        // Memory write
        self.write_reg(&[0x2c]).await;
    }

}
