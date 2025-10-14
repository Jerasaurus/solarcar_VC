use defmt::info;
use embassy_stm32::gpio::Output;
use embassy_stm32::mode::Async;
use embassy_stm32::spi::Spi;
use embassy_time::Timer;
use embedded_graphics::pixelcolor::Gray4;
use embedded_graphics::prelude::*;

// Display dimensions
pub const DISPLAY_WIDTH: usize = 256;
pub const DISPLAY_HEIGHT: usize = 64;
const DISPLAY_BUFFER_SIZE: usize = DISPLAY_WIDTH * DISPLAY_HEIGHT;

// SSD1322 Commands
const CMD_SET_COMMAND_LOCK: u8 = 0xFD;
const CMD_DISPLAY_OFF: u8 = 0xAE;
const CMD_DISPLAY_ON: u8 = 0xAF;
const CMD_SET_CLOCK_DIVIDER: u8 = 0xB3;
const CMD_SET_MUX_RATIO: u8 = 0xCA;
const CMD_SET_DISPLAY_OFFSET: u8 = 0xA2;
const CMD_SET_START_LINE: u8 = 0xA1;
const CMD_SET_REMAP: u8 = 0xA0;
const CMD_SET_GPIO: u8 = 0xB5;
const CMD_FUNCTION_SELECT: u8 = 0xAB;
const CMD_DISPLAY_ENHANCE: u8 = 0xB2;
const CMD_SET_CONTRAST_CURRENT: u8 = 0xC1;
const CMD_MASTER_CURRENT_CONTROL: u8 = 0xC7;
const CMD_SELECT_DEFAULT_GRAYSCALE: u8 = 0xB9;
const CMD_SET_PHASE_LENGTH: u8 = 0xB1;
const CMD_DISPLAY_ENHANCE_B: u8 = 0xD1;
const CMD_SET_PRECHARGE_VOLTAGE: u8 = 0xBB;
const CMD_SET_SECOND_PRECHARGE_PERIOD: u8 = 0xB6;
const CMD_SET_VCOMH: u8 = 0xBE;
const CMD_NORMAL_DISPLAY: u8 = 0xA6;
const CMD_EXIT_PARTIAL_DISPLAY: u8 = 0xA9;
const CMD_SET_COLUMN_ADDR: u8 = 0x15;
const CMD_SET_ROW_ADDR: u8 = 0x75;
const CMD_WRITE_RAM: u8 = 0x5C;

const MIN_SEG: u8 = 0x1C;
const MAX_SEG: u8 = 0x5B;

pub struct Ssd1322Display<'a> {
    spi: Spi<'a, Async>,
    dc: Output<'a>,
    cs: Output<'a>,
    rst: Output<'a>,
    framebuffer: [u8; DISPLAY_BUFFER_SIZE],
}

impl<'a> Ssd1322Display<'a> {
    pub async fn new(
        spi: Spi<'a, Async>,
        dc: Output<'a>,
        cs: Output<'a>,
        rst: Output<'a>,
    ) -> Self {
        let mut display = Self {
            spi,
            dc,
            cs,
            rst,
            framebuffer: [0; DISPLAY_BUFFER_SIZE],
        };

        display.init().await;
        display
    }

    async fn init(&mut self) {
        // Hardware reset
        self.cs.set_high();
        self.rst.set_low();
        Timer::after_millis(1).await;
        self.rst.set_high();
        self.dc.set_high();
        Timer::after_millis(10).await;

        // Initialization sequence based on datasheet
        self.send_command(CMD_SET_COMMAND_LOCK).await;
        self.send_data(&[0x12]).await; // Unlock

        self.send_command(CMD_DISPLAY_OFF).await;

        self.send_command(CMD_SET_CLOCK_DIVIDER).await;
        self.send_data(&[0x91]).await;

        self.send_command(CMD_SET_MUX_RATIO).await;
        self.send_data(&[0x3F]).await; // 64 MUX

        self.send_command(CMD_SET_DISPLAY_OFFSET).await;
        self.send_data(&[0x00]).await;

        self.send_command(CMD_SET_START_LINE).await;
        self.send_data(&[0x00]).await;

        self.send_command(CMD_SET_REMAP).await;
        self.send_data(&[0x14, 0x11]).await;

        self.send_command(CMD_SET_GPIO).await;
        self.send_data(&[0x00]).await;

        self.send_command(CMD_FUNCTION_SELECT).await;
        self.send_data(&[0x01]).await; // Enable internal VDD regulator

        self.send_command(CMD_DISPLAY_ENHANCE).await;
        self.send_data(&[0xA0, 0xFD]).await;

        self.send_command(CMD_SET_CONTRAST_CURRENT).await;
        self.send_data(&[0xFF]).await;

        self.send_command(CMD_MASTER_CURRENT_CONTROL).await;
        self.send_data(&[0x0F]).await;

        self.send_command(CMD_SELECT_DEFAULT_GRAYSCALE).await;

        self.send_command(CMD_SET_PHASE_LENGTH).await;
        self.send_data(&[0xE2]).await;

        self.send_command(CMD_DISPLAY_ENHANCE_B).await;
        self.send_data(&[0x82, 0x20]).await;

        self.send_command(CMD_SET_PRECHARGE_VOLTAGE).await;
        self.send_data(&[0x1F]).await;

        self.send_command(CMD_SET_SECOND_PRECHARGE_PERIOD).await;
        self.send_data(&[0x08]).await;

        self.send_command(CMD_SET_VCOMH).await;
        self.send_data(&[0x07]).await;

        self.send_command(CMD_NORMAL_DISPLAY).await;
        self.send_command(CMD_EXIT_PARTIAL_DISPLAY).await;

        Timer::after_millis(10).await;

        self.send_command(CMD_DISPLAY_ON).await;
        Timer::after_millis(50).await;

        info!("SSD1322 initialized");
    }

    async fn send_command(&mut self, cmd: u8) {
        self.dc.set_low(); // Command mode
        self.cs.set_low();
        self.spi.write(&[cmd]).await.ok();
        self.cs.set_high();
    }

    async fn send_data(&mut self, data: &[u8]) {
        self.dc.set_high(); // Data mode
        self.cs.set_low();
        self.spi.write(data).await.ok();
        self.cs.set_high();
    }

    pub fn clear(&mut self) {
        self.framebuffer.fill(0);
    }

    pub async fn flush(&mut self) {
        // Set column address
        self.send_command(CMD_SET_COLUMN_ADDR).await;
        self.send_data(&[MIN_SEG, MAX_SEG]).await;

        // Set row address
        self.send_command(CMD_SET_ROW_ADDR).await;
        self.send_data(&[0, 63]).await;

        // Write RAM command
        self.send_command(CMD_WRITE_RAM).await;

        // Pack and send framebuffer data (2 pixels per byte, 4-bit each)
        self.dc.set_high();
        self.cs.set_low();

        // We need to pack the data on the fly to avoid stack allocation
        // Send in chunks to avoid large stack allocation
        const CHUNK_SIZE: usize = 256;
        let mut packed_chunk = [0u8; CHUNK_SIZE / 2];

        for chunk_start in (0..DISPLAY_BUFFER_SIZE).step_by(CHUNK_SIZE) {
            let chunk_end = (chunk_start + CHUNK_SIZE).min(DISPLAY_BUFFER_SIZE);
            let chunk_len = chunk_end - chunk_start;

            for i in (0..chunk_len).step_by(2) {
                let idx = chunk_start + i;
                packed_chunk[i / 2] = (self.framebuffer[idx] << 4) | self.framebuffer[idx + 1];
            }

            self.spi.write(&packed_chunk[..chunk_len / 2]).await.ok();
        }

        self.cs.set_high();
    }
}

impl<'a> DrawTarget for Ssd1322Display<'a> {
    type Color = Gray4;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels.into_iter() {
            if coord.x >= 0
                && coord.x < DISPLAY_WIDTH as i32
                && coord.y >= 0
                && coord.y < DISPLAY_HEIGHT as i32
            {
                let x = coord.x as usize;
                let y = coord.y as usize;
                self.framebuffer[x + y * DISPLAY_WIDTH] = color.luma();
            }
        }
        Ok(())
    }
}

impl<'a> OriginDimensions for Ssd1322Display<'a> {
    fn size(&self) -> Size {
        Size::new(DISPLAY_WIDTH as u32, DISPLAY_HEIGHT as u32)
    }
}