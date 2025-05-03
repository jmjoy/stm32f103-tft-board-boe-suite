pub mod font;
pub mod pic;

use as_what::{AsU16, AsUsize};
use core::cmp::{Ordering, max};
use embassy_stm32::{gpio::Output, mode::Async, spi::Spi};
use embassy_time::Timer;
use font::{ASCII_1206, ASCII_1608, ASCII_2412, ASCII_3216, ChineseFontSize, FontSize};

pub const DIRECTION: Direction = Direction::Horizontal0;

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

#[derive(Clone, Copy)]
pub enum CharMode {
    NonOverlay,
    Overlay,
}

pub struct LCD {
    spi: Spi<'static, Async>,
    cs: Output<'static>,
    res: Output<'static>,
    blk: Output<'static>,
    dc: Output<'static>,
}

impl LCD {
    pub fn new(
        spi: Spi<'static, Async>, cs: Output<'static>, res: Output<'static>, blk: Output<'static>,
        dc: Output<'static>,
    ) -> Self {
        Self {
            spi,
            cs,
            res,
            blk,
            dc,
        }
    }

    pub async fn init(&mut self) {
        self.res.set_low();
        Timer::after_millis(100).await;
        self.res.set_high();
        Timer::after_millis(100).await;

        // 打开背光
        self.blk.set_high();
        Timer::after_millis(100).await;

        // Sleep exit
        self.write_reg(&[0x11]).await;
        Timer::after_millis(120).await;
        self.write_reg(&[0xb1]).await;
        self.write_data8(&[0x05, 0x3c, 0x3c]).await;

        self.write_reg(&[0xb2]).await;
        self.write_data8(&[0x05, 0x3c, 0x3c]).await;

        self.write_reg(&[0xb3]).await;
        self.write_data8(&[0x05, 0x3c, 0x3c, 0x05, 0x3c, 0x3c])
            .await;

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
            0x0b, 0x17, 0x0a, 0x0d, 0x1a, 0x19, 0x16, 0x1d, 0x21, 0x26, 0x37, 0x3c, 0x00, 0x09,
            0x05, 0x10,
        ])
        .await;

        self.write_reg(&[0xe1]).await;
        self.write_data8(&[
            0x0c, 0x19, 0x09, 0x0d, 0x1b, 0x19, 0x15, 0x1d, 0x21, 0x26, 0x39, 0x3e, 0x00, 0x09,
            0x05, 0x10,
        ])
        .await;

        Timer::after_millis(120).await;
        self.write_reg(&[0x29]).await; // Display on
    }

    pub async fn fill(&mut self, x_start: u16, y_start: u16, x_end: u16, y_end: u16, color: u16) {
        // Set the display address range
        self.set_address(x_start, y_start, x_end - 1, y_end - 1)
            .await;

        // Fill the screen pixel by pixel
        for _ in y_start..y_end {
            for _ in x_start..x_end {
                // For each pixel, write the color data
                self.write_data(&[color]).await;
            }
        }
    }

    pub async fn draw_point(&mut self, x: u16, y: u16, color: u16) {
        self.set_address(x, y, x, y).await;
        self.write_data(&[color]).await;
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
            }
            Direction::Horizontal0 | Direction::Horizontal1 => {
                // Column address set
                self.write_reg(&[0x2a]).await;
                self.write_data(&[x1, x2]).await;

                // Row address set
                self.write_reg(&[0x2b]).await;
                self.write_data(&[y1 + 24, y2 + 24]).await;
            }
        }

        // Memory write
        self.write_reg(&[0x2c]).await;
    }

    pub async fn draw_line(&mut self, x1: u16, y1: u16, x2: u16, y2: u16, color: u16) {
        let mut xerr: i32 = 0;
        let mut yerr: i32 = 0;
        let mut delta_x = i32::from(x2) - i32::from(x1);
        let mut delta_y = i32::from(y2) - i32::from(y1);

        let incx: i32;
        let incy: i32;

        let mut row = i32::from(x1);
        let mut col = i32::from(y1);

        match delta_x.cmp(&0) {
            Ordering::Greater => {
                incx = 1;
            }
            Ordering::Equal => {
                incx = 0;
            }
            Ordering::Less => {
                incx = -1;
                delta_x = -delta_x;
            }
        }

        match delta_y.cmp(&0) {
            Ordering::Greater => {
                incy = 1;
            }
            Ordering::Equal => {
                incy = 0;
            }
            Ordering::Less => {
                incy = -1;
                delta_y = -delta_y;
            }
        }

        let distance = max(delta_x, delta_y);

        for _ in 0..=distance as usize {
            self.draw_point(row as u16, col as u16, color).await;
            xerr += delta_x;
            yerr += delta_y;

            if xerr > distance {
                xerr -= distance;
                row += incx;
            }

            if yerr > distance {
                yerr -= distance;
                col += incy;
            }
        }
    }

    pub async fn draw_rectangle(&mut self, x1: u16, y1: u16, x2: u16, y2: u16, color: u16) {
        self.draw_line(x1, y1, x2, y1, color).await;
        self.draw_line(x1, y1, x1, y2, color).await;
        self.draw_line(x1, y2, x2, y2, color).await;
        self.draw_line(x2, y1, x2, y2, color).await;
    }

    pub async fn draw_circle(&mut self, x0: u16, y0: u16, r: u8, color: u16) {
        let mut a: i32 = 0;
        let mut b: i32 = r as i32;

        while a <= b {
            self.draw_point(x0.wrapping_sub(b as u16), y0.wrapping_sub(a as u16), color)
                .await;
            self.draw_point(x0.wrapping_add(b as u16), y0.wrapping_sub(a as u16), color)
                .await;
            self.draw_point(x0.wrapping_sub(a as u16), y0.wrapping_add(b as u16), color)
                .await;
            self.draw_point(x0.wrapping_sub(a as u16), y0.wrapping_sub(b as u16), color)
                .await;
            self.draw_point(x0.wrapping_add(b as u16), y0.wrapping_add(a as u16), color)
                .await;
            self.draw_point(x0.wrapping_add(a as u16), y0.wrapping_sub(b as u16), color)
                .await;
            self.draw_point(x0.wrapping_add(a as u16), y0.wrapping_add(b as u16), color)
                .await;
            self.draw_point(x0.wrapping_sub(b as u16), y0.wrapping_add(a as u16), color)
                .await;

            a += 1;
            if (a * a + b * b) > (r as i32 * r as i32) {
                b -= 1;
            }
        }
    }

    pub async fn show_char(
        &mut self, (mut x, mut y): (u16, u16), ch: char, fc: u16, bc: u16, size: FontSize,
        mode: CharMode,
    ) {
        let size_y = size.y();
        let size_x = size.x();
        let typeface_num = (size_x / 8 + if size_x % 8 != 0 { 1 } else { 0 }) * size_y;
        let num = ch.as_usize() - ' '.as_usize();

        let x0 = x;
        let mut m = 0;

        // Set address range for this character
        self.set_address(x, y, x + size_x.as_u16() - 1, y + size_y.as_u16() - 1)
            .await;

        // Select appropriate font based on size
        for i in 0..typeface_num.as_usize() {
            // In a real implementation, we would access font data here
            // For this example, we'll just use a placeholder
            let temp = match size {
                FontSize::_6x12 => ASCII_1206[num][i],
                FontSize::_8x16 => ASCII_1608[num][i],
                FontSize::_12x24 => ASCII_2412[num][i],
                FontSize::_16x32 => ASCII_3216[num][i],
            };

            for t in 0..8 {
                match mode {
                    CharMode::NonOverlay => {
                        if (temp & (0x01 << t)) != 0 {
                            self.write_data(&[fc]).await;
                        } else {
                            self.write_data(&[bc]).await;
                        }

                        m += 1;
                        if m % size_x == 0 {
                            m = 0;
                            break;
                        }
                    }
                    CharMode::Overlay => {
                        if (temp & (0x01 << t)) != 0 {
                            self.draw_point(x, y, fc).await;
                        }

                        x += 1;
                        if (x - x0) == size_x as u16 {
                            x = x0;
                            y += 1;
                            break;
                        }
                    }
                }
            }
        }
    }

    pub async fn show_string(
        &mut self, (mut x, y): (u16, u16), s: &str, fc: u16, bc: u16, size: FontSize,
        mode: CharMode,
    ) {
        let size_y = size.y();
        for c in s.chars() {
            self.show_char((x, y), c, fc, bc, size, mode).await;
            x += size_y.as_u16() / 2;
        }
    }

    pub async fn show_int_num(
        &mut self, (x, y): (u16, u16), num: u16, len: u8, fc: u16, bc: u16, size: FontSize,
    ) {
        let mut enshow = false;
        let size_x = size.x();

        for t in 0..len {
            let temp = ((num / 10u16.pow((len - t - 1) as u32)) % 10) as u8;

            if !enshow && t < (len - 1) {
                if temp == 0 {
                    self.show_char(
                        (x + t as u16 * size_x as u16, y),
                        ' ',
                        fc,
                        bc,
                        size,
                        CharMode::NonOverlay,
                    )
                    .await;
                    continue;
                } else {
                    enshow = true;
                }
            }

            self.show_char(
                (x + t as u16 * size_x as u16, y),
                (temp + 48) as char,
                fc,
                bc,
                size,
                CharMode::NonOverlay,
            )
            .await;
        }
    }

    pub async fn show_float_num(
        &mut self, (x, y): (u16, u16), num: f32, mut len: u8, fc: u16, bc: u16, size: FontSize,
    ) {
        let size_x = size.x();
        let num1 = (num * 100.0) as u16;

        let mut t = 0;

        loop {
            let temp = ((num1 / 10u16.pow((len - t - 1) as u32)) % 10) as u8;

            if t == (len - 2) {
                self.show_char(
                    (x + (len - 2) as u16 * size_x as u16, y),
                    '.',
                    fc,
                    bc,
                    size,
                    CharMode::NonOverlay,
                )
                .await;
                t += 1;
                len += 1;
            }

            self.show_char(
                (x + t as u16 * size_x as u16, y),
                (temp + 48) as char,
                fc,
                bc,
                size,
                CharMode::NonOverlay,
            )
            .await;

            if t >= len {
                break;
            }

            t += 1;
        }
    }

    pub async fn show_chinese(
        &mut self, (mut x, y): (u16, u16), s: &str, fc: u16, bc: u16, size: ChineseFontSize,
        mode: CharMode,
    ) {
        for ch in s.chars() {
            self.show_chinese_char((x, y), ch, fc, bc, size, mode).await;
            x += size.y() as u16;
        }
    }

    async fn show_chinese_char(
        &mut self, (mut x, mut y): (u16, u16), ch: char, fc: u16, bc: u16, size: ChineseFontSize,
        mode: CharMode,
    ) {
        let size_y = size.y() as usize;
        let size_x = size.x() as usize;
        let typeface_num = (size_x / 8 + if size_x % 8 != 0 { 1 } else { 0 }) * size_y;

        let x0 = x;
        let mut m = 0;

        self.set_address(x, y, x + size_y as u16 - 1, y + size_y as u16 - 1)
            .await;

        match size {
            ChineseFontSize::_12x12 => {
                // Search for character in TFONT12
                for font in font::TFONT12.iter() {
                    if font.index == ch {
                        for i in 0..typeface_num {
                            for j in 0..8 {
                                match mode {
                                    CharMode::NonOverlay => {
                                        if i < font.msk.len() && font.msk[i] & (0x01 << j) != 0 {
                                            self.write_data(&[fc]).await;
                                        } else {
                                            self.write_data(&[bc]).await;
                                        }

                                        m += 1;
                                        if m % size_y == 0 {
                                            m = 0;
                                            break;
                                        }
                                    }
                                    CharMode::Overlay => {
                                        if i < font.msk.len() && font.msk[i] & (0x01 << j) != 0 {
                                            self.draw_point(x, y, fc).await;
                                        }

                                        x += 1;
                                        if (x - x0) == size_y as u16 {
                                            x = x0;
                                            y += 1;
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                        return; // Found the character, exit the function
                    }
                }
            }
            ChineseFontSize::_16x16 => {
                // Search for character in TFONT16
                for font in font::TFONT16.iter() {
                    if font.index == ch {
                        for i in 0..typeface_num {
                            for j in 0..8 {
                                match mode {
                                    CharMode::NonOverlay => {
                                        if i < font.msk.len() && font.msk[i] & (0x01 << j) != 0 {
                                            self.write_data(&[fc]).await;
                                        } else {
                                            self.write_data(&[bc]).await;
                                        }

                                        m += 1;
                                        if m % size_y == 0 {
                                            m = 0;
                                            break;
                                        }
                                    }
                                    CharMode::Overlay => {
                                        if i < font.msk.len() && font.msk[i] & (0x01 << j) != 0 {
                                            self.draw_point(x, y, fc).await;
                                        }

                                        x += 1;
                                        if (x - x0) == size_y as u16 {
                                            x = x0;
                                            y += 1;
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                        return; // Found the character, exit the function
                    }
                }
            }
            ChineseFontSize::_24x24 => {
                // Search for character in TFONT24
                for font in font::TFONT24.iter() {
                    if font.index == ch {
                        for i in 0..typeface_num {
                            for j in 0..8 {
                                match mode {
                                    CharMode::NonOverlay => {
                                        if i < font.msk.len() && font.msk[i] & (0x01 << j) != 0 {
                                            self.write_data(&[fc]).await;
                                        } else {
                                            self.write_data(&[bc]).await;
                                        }

                                        m += 1;
                                        if m % size_y == 0 {
                                            m = 0;
                                            break;
                                        }
                                    }
                                    CharMode::Overlay => {
                                        if i < font.msk.len() && font.msk[i] & (0x01 << j) != 0 {
                                            self.draw_point(x, y, fc).await;
                                        }

                                        x += 1;
                                        if (x - x0) == size_y as u16 {
                                            x = x0;
                                            y += 1;
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                        return; // Found the character, exit the function
                    }
                }
            }
            ChineseFontSize::_32x32 => {
                // Search for character in TFONT32
                for font in font::TFONT32.iter() {
                    if font.index == ch {
                        for i in 0..typeface_num {
                            for j in 0..8 {
                                match mode {
                                    CharMode::NonOverlay => {
                                        if i < font.msk.len() && font.msk[i] & (0x01 << j) != 0 {
                                            self.write_data(&[fc]).await;
                                        } else {
                                            self.write_data(&[bc]).await;
                                        }

                                        m += 1;
                                        if m % size_y == 0 {
                                            m = 0;
                                            break;
                                        }
                                    }
                                    CharMode::Overlay => {
                                        if i < font.msk.len() && font.msk[i] & (0x01 << j) != 0 {
                                            self.draw_point(x, y, fc).await;
                                        }

                                        x += 1;
                                        if (x - x0) == size_y as u16 {
                                            x = x0;
                                            y += 1;
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                        return; // Found the character, exit the function
                    }
                }
            }
        }
    }

    pub async fn show_picture(
        &mut self, (x, y): (u16, u16), (length, width): (u16, u16), pic: &[u8],
    ) {
        self.set_address(x, y, x + length - 1, y + width - 1).await;

        let mut k = 0usize;

        for _ in 0..length {
            for _ in 0..width {
                self.write_data8(&[pic[k * 2], pic[k * 2 + 1]]).await;
                k += 1;
            }
        }
    }
}
