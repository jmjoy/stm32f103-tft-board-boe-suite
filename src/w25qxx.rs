use embassy_stm32::{gpio::Output, mode::{self, Async, Mode}, spi::Spi};
use num_enum::TryFromPrimitive;

// Page size constants
pub const SPI_FLASH_PAGE_SIZE: usize = 256;
pub const SPI_FLASH_PER_WRITE_PAGE_SIZE: usize = 256;

// W25X Commands
pub const W25X_WRITE_ENABLE: u8 = 0x06;
pub const W25X_WRITE_DISABLE: u8 = 0x04;
pub const W25X_READ_STATUS_REG: u8 = 0x05;
pub const W25X_WRITE_STATUS_REG: u8 = 0x01;
pub const W25X_READ_DATA: u8 = 0x03;
pub const W25X_FAST_READ_DATA: u8 = 0x0B;
pub const W25X_FAST_READ_DUAL: u8 = 0x3B;
pub const W25X_PAGE_PROGRAM: u8 = 0x02;
pub const W25X_BLOCK_ERASE: u8 = 0xD8;
pub const W25X_SECTOR_ERASE: u8 = 0x20;
pub const W25X_CHIP_ERASE: u8 = 0xC7;
pub const W25X_POWER_DOWN: u8 = 0xB9;
pub const W25X_RELEASE_POWER_DOWN: u8 = 0xAB;
pub const W25X_DEVICE_ID: u8 = 0xAB;
pub const W25X_MANUFACT_DEVICE_ID: u8 = 0x90;
pub const W25X_JEDEC_DEVICE_ID: u8 = 0x9F;

// Status flags
pub const WIP_FLAG: u8 = 0x01; // Write In Progress (WIP) flag

pub const DUMMY_BYTE: u8 = 0xFF;

/// W25Q Chip IDs
#[derive(TryFromPrimitive)]
#[repr(u32)]
pub enum W25QxxID {
 W25Q16     = 0xEF4015,
 W25Q32     = 0xEF4016,
 W25Q64     = 0xEF4017,
 W25Q80     = 0xEF4014,
}

pub struct W25Qxx {
    spi: Spi<'static, Async>,
    cs: Output<'static>,
}

impl W25Qxx {
    pub fn new(spi: Spi<'static, Async>, cs: Output<'static>) -> Self {
        Self { spi, cs }
    }

    pub async fn read_device_id(&mut self) -> u8 {
        self.cs.set_low();

        let data = &mut [W25X_DEVICE_ID, DUMMY_BYTE, DUMMY_BYTE, DUMMY_BYTE, DUMMY_BYTE];
        self.spi.transfer_in_place(data).await.unwrap();
        let device_id = data[4];

        self.cs.set_high();

        device_id
    }

    pub async fn read_id(&mut self) -> u32 {
        self.cs.set_low();

        let data = &mut [W25X_JEDEC_DEVICE_ID, DUMMY_BYTE, DUMMY_BYTE, DUMMY_BYTE];
        self.spi.transfer_in_place(data).await.unwrap();
        data[0] = 0;
        let id = u32::from_be_bytes(*data);

        self.cs.set_high();

        id
    }

    pub async fn sector_erase(&mut self, sector_addr: u32) {
        self.write_enable().await;
        self.wait_for_write_end().await;

        self.cs.set_low();
        let mut data = u32::to_be_bytes(sector_addr);
        data[0] = W25X_SECTOR_ERASE;
        self.spi.write(&data).await.unwrap();
        self.cs.set_high();

        self.wait_for_write_end().await;
    }

    pub async fn write_enable(&mut self) {
        self.cs.set_low();
        self.spi.write(&[W25X_WRITE_ENABLE]).await.unwrap();
        self.cs.set_high();
    }

    pub async fn wait_for_write_end(&mut self) {
        self.cs.set_low();

        self.spi.write(&[W25X_READ_STATUS_REG]).await.unwrap();
        
        let mut data = [DUMMY_BYTE];
        loop {
            self.spi.transfer_in_place(&mut data).await.unwrap();
            if data[0] & WIP_FLAG == 0 {
                break;
            }
        }

        self.cs.set_high();
    }

    pub async fn page_write(&mut self, buffer: &[u8], write_addr: u32) {
        self.write_enable().await;

        self.cs.set_low();
        
        // Send "Write to Memory" instruction and address
        let mut cmd_addr = u32::to_be_bytes(write_addr);
        cmd_addr[0] = W25X_PAGE_PROGRAM;
        self.spi.write(&cmd_addr).await.unwrap();

        // Limit write size to page size
        let write_count = buffer.len().min(SPI_FLASH_PER_WRITE_PAGE_SIZE);
        let buffer = &buffer[..write_count];
        
        // Write the data
        self.spi.write(buffer).await.unwrap();

        self.cs.set_high();

        // Wait for the write to complete
        self.wait_for_write_end().await;
    }

    pub async fn buffer_read(&mut self, buffer: &mut [u8], read_addr: u32) {
        self.cs.set_low();

        // Send "Read from Memory" instruction and address
        let mut cmd_addr = u32::to_be_bytes(read_addr);
        cmd_addr[0] = W25X_READ_DATA;
        self.spi.write(&cmd_addr).await.unwrap();

        // Read the data
        self.spi.read(buffer).await.unwrap();

        self.cs.set_high();
    }

    pub async fn buffer_write(&mut self, buffer: &[u8], write_addr: u32) {
        if buffer.len() == 0 {
            return;
        }

        let mut current_addr = write_addr;
        let mut bytes_written = 0;
        let total_bytes = buffer.len() as usize;
        
        // Handle first unaligned page if needed
        let first_page_offset = current_addr as usize % SPI_FLASH_PAGE_SIZE;
        if first_page_offset > 0 {
            // Calculate bytes to write to align with page boundary
            let bytes_to_page_boundary = (SPI_FLASH_PAGE_SIZE - first_page_offset) as usize;
            let bytes_to_write = bytes_to_page_boundary.min(total_bytes);
            
            // Write partial first page
            self.page_write(&buffer[bytes_written..bytes_written + bytes_to_write], current_addr).await;
            
            bytes_written += bytes_to_write;
            current_addr += bytes_to_write as u32;
            
            if bytes_written >= total_bytes {
                return; // Done if first page write covered all data
            }
        }
        
        // Write full pages
        while bytes_written + SPI_FLASH_PAGE_SIZE <= total_bytes {
            self.page_write(&buffer[bytes_written..bytes_written + SPI_FLASH_PAGE_SIZE], current_addr).await;
            
            bytes_written += SPI_FLASH_PAGE_SIZE;
            current_addr += SPI_FLASH_PAGE_SIZE as u32;
        }
        
        // Write remaining bytes (last partial page)
        let remaining_bytes = total_bytes - bytes_written;
        if remaining_bytes > 0 {
            self.page_write(&buffer[bytes_written..bytes_written + remaining_bytes], current_addr).await;
        }
    }
}
