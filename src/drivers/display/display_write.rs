// Display write functions - high-level drawing functions for specific UI elements
// Ported from the C steering wheel codebase

use super::ssd1322::*;
use super::font16::{FONT_WIDTH, FONT_HEIGHT};
use core::fmt::Write;
use heapless::String;

/// Drive states matching the C enum
#[derive(Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum DriveState {
    Drive = 0,
    Reverse = 1,
    Cruise = 2,
    Neutral = 3,
}

impl<'a> Ssd1322Display<'a> {
    /// Write the drive state indicator (D/R/C/N)
    pub fn write_drive_state(&mut self, drive_state: DriveState) {
        let x = 17 * FONT_WIDTH;
        let y = 3 * FONT_HEIGHT;
        let drive_states = "DRCN";
        
        for (i, ch) in drive_states.chars().enumerate() {
            let fg_shade = if i == drive_state as usize {
                DISPLAY_BLACK
            } else {
                DISPLAY_LOW_SHADE
            };
            let bg_shade = if i == drive_state as usize {
                DISPLAY_MID_SHADE
            } else {
                DISPLAY_BLACK
            };
            self.draw_char(x + (i * FONT_WIDTH), y, fg_shade, bg_shade, ch);
        }
    }

    /// Write the vehicle speed
    pub fn write_speed(&mut self, speed: f32) {
        let x = 5 * FONT_WIDTH;
        let y = 0 * FONT_HEIGHT;
        
        let speed = speed.abs();
        
        let mut buf: String<16> = String::new();
        write!(&mut buf, "{:2.0}", speed).ok();
        
        self.draw_string_large(x, y, DISPLAY_WHITE, DISPLAY_BLACK, &buf);
    }

    /// Write the cruise speed
    pub fn write_cruise_speed(&mut self, engaged: bool, speed: f32) {
        let x = 14 * FONT_WIDTH;
        let y = 3 * FONT_HEIGHT;
        
        let speed = speed.abs();
        
        let mut buf: String<16> = String::new();
        write!(&mut buf, "{:2.0}", speed).ok();
        
        let shade = if engaged {
            DISPLAY_WHITE
        } else {
            DISPLAY_LOW_SHADE
        };
        
        self.draw_string(x, y, shade, DISPLAY_BLACK, &buf);
    }

    /// Write the battery current
    pub fn write_current(&mut self, current: f32) {
        let x = 13 * FONT_WIDTH;
        let y = 0 * FONT_HEIGHT;
        
        let current = current.abs();
        
        let mut buf: String<16> = String::new();
        write!(&mut buf, "{:5.1}A", current).ok();
        
        self.draw_string(x, y, DISPLAY_WHITE, DISPLAY_BLACK, &buf);
    }

    /// Write the high voltage (battery voltage)
    pub fn write_high_voltage(&mut self, voltage: f32) {
        let x = 13 * FONT_WIDTH;
        let y = 1 * FONT_HEIGHT;
        
        let mut buf: String<16> = String::new();
        write!(&mut buf, "{:5.1}v", voltage).ok();
        
        self.draw_string(x, y, DISPLAY_WHITE, DISPLAY_BLACK, &buf);
    }

    /// Write the low voltage
    pub fn write_low_voltage(&mut self, voltage: f32) {
        let x = 13 * FONT_WIDTH;
        let y = 2 * FONT_HEIGHT;
        
        let mut buf: String<16> = String::new();
        write!(&mut buf, "{:5.1}v", voltage).ok();
        
        self.draw_string(x, y, DISPLAY_WHITE, DISPLAY_BLACK, &buf);
    }

    /// Write regen indicator
    pub fn write_regen(&mut self, enabled: bool, engaged: bool, hwbit: bool) {
        let x = 0 * FONT_WIDTH;
        let y = 1 * FONT_HEIGHT;
        
        let shade = if !enabled {
            DISPLAY_VLOW_SHADE  // Barely visible if disabled
        } else if engaged {
            DISPLAY_WHITE  // Bright white if enabled and in-use
        } else {
            DISPLAY_MID_SHADE  // Mid-grey if enabled and not in-use
        };
        
        self.draw_char(x, y, shade, DISPLAY_BLACK, 'R');
        
        // Draw hardware bit indicator pixel
        if y > 0 {
            self.draw_pixel(x + 1, y - 1, if hwbit { DISPLAY_WHITE } else { DISPLAY_BLACK });
        }
    }

    /// Write throttle indicator
    pub fn write_throttle(&mut self, enabled: bool, engaged: bool, hwbit: bool) {
        let x = 1 * FONT_WIDTH;
        let y = 1 * FONT_HEIGHT;
        
        let shade = if !enabled {
            DISPLAY_VLOW_SHADE  // Barely visible if disabled
        } else if engaged {
            DISPLAY_WHITE  // Bright white if enabled and in-use
        } else {
            DISPLAY_MID_SHADE  // Mid-grey if enabled and not in-use
        };
        
        self.draw_char(x, y, shade, DISPLAY_BLACK, 'T');
        
        // Draw hardware bit indicator pixel
        if x > 0 && y > 0 {
            self.draw_pixel(x - 1, y - 1, if hwbit { DISPLAY_WHITE } else { DISPLAY_BLACK });
        }
    }

    /// Draw lock indicator
    pub fn write_lock(&mut self, engaged: bool) {
        let x = 0 * FONT_WIDTH;
        let y = 3 * FONT_HEIGHT;
        
        let shade = if engaged {
            DISPLAY_WHITE
        } else {
            DISPLAY_VLOW_SHADE
        };
        
        // Draw lock body (rectangle)
        for i in 0..FONT_WIDTH {
            self.draw_pixel(x + i, y, shade);
            self.draw_pixel(x + i, y + 1, shade);
            self.draw_pixel(x + i, y + 8, shade);
            self.draw_pixel(x + i, y + 9, shade);
        }
        
        for i in 0..8 {
            self.draw_pixel(x, y + i, shade);
            self.draw_pixel(x + 1, y + i, shade);
            self.draw_pixel(x + FONT_WIDTH - 2, y + i, shade);
            self.draw_pixel(x + FONT_WIDTH - 1, y + i, shade);
        }
        
        // Draw shackle (top arc)
        if y >= 5 {
            for i in 0..4 {
                self.draw_pixel(x + 2, y - i, shade);
                self.draw_pixel(x + 3, y - i, shade);
                self.draw_pixel(x + FONT_WIDTH - 4, y - i, shade);
                self.draw_pixel(x + FONT_WIDTH - 3, y - i, shade);
            }
            
            for i in 0..(FONT_WIDTH - 8) {
                self.draw_pixel(x + 4 + i, y - 4, shade);
                self.draw_pixel(x + 4 + i, y - 5, shade);
            }
        }
    }

    /// Draw left turn signal
    pub fn write_left_signal(&mut self, on: bool) {
        let shade = if on {
            DISPLAY_WHITE
        } else {
            DISPLAY_LOW_SHADE
        };
        
        // Draw arrow body
        for i in 5..9 {
            for j in 4..=8 {
                self.draw_pixel(i, j, shade);
            }
        }
        
        // Draw arrow head
        for i in 0..=4 {
            for j in (6 - i)..=(6 + i) {
                self.draw_pixel(i, j, shade);
            }
        }
    }

    /// Draw right turn signal
    pub fn write_right_signal(&mut self, on: bool) {
        let shade = if on {
            DISPLAY_WHITE
        } else {
            DISPLAY_LOW_SHADE
        };
        
        // Draw arrow body
        for i in 5..9 {
            for j in 4..=8 {
                self.draw_pixel(DISPLAY_WIDTH - 4 - i, j, shade);
            }
        }
        
        // Draw arrow head
        for i in 0..=4 {
            for j in (6 - i)..=(6 + i) {
                self.draw_pixel(DISPLAY_WIDTH - 4 - i, j, shade);
            }
        }
    }

    /// Draw turn signal state with blinking
    pub fn write_turn_signal_state(&mut self, left_state: &mut bool, right_state: &mut bool, last_blink: &mut u32, current_time: u32) {
        // Blink at ~2Hz (500ms period)
        if current_time - *last_blink > 500 {
            *left_state = !*left_state;
            *right_state = !*right_state;
            *last_blink = current_time;
        }
        
        // TODO: Get actual turn signal state from vehicle
        // For now, draw both off
        self.write_left_signal(false);
        self.write_right_signal(false);
    }

    /// Draw a timeout indicator box with VC label
    pub fn write_timeout(&mut self, time_since: u32) {
        let x = 5 * FONT_WIDTH;
        let y = 3 * FONT_HEIGHT + 4;
        const VC_TIMEOUT: u32 = 300;
        let dead = time_since >= VC_TIMEOUT;
        let ratio = if dead {
            1.0
        } else {
            time_since as f32 / VC_TIMEOUT as f32
        };
        
        // Fill bar
        for i in (x + 1)..(x + (30.0 * ratio) as usize) {
            for j in (y + 1)..(y + 9) {
                let shade = if dead {
                    DISPLAY_VLOW_SHADE
                } else {
                    DISPLAY_MID_SHADE
                };
                self.draw_pixel(i, j, shade);
            }
        }
        
        // Draw X if dead
        if dead {
            for i in 0..30 {
                let y1 = (i * 10) / 30;
                let y2 = ((29 - i) * 10) / 30;
                self.draw_pixel(x + i, y + y1, DISPLAY_MID_SHADE);
                self.draw_pixel(x + i, y + y2, DISPLAY_MID_SHADE);
            }
        }
        
        // Draw box outline
        self.draw_box_outline(x, y, 30, 10);
        
        // Draw "VC" label
        let color = DISPLAY_WHITE;
        // V
        self.draw_pixel(x + 13, y + 3, color);
        self.draw_pixel(x + 13, y + 4, color);
        self.draw_pixel(x + 13, y + 5, color);
        self.draw_pixel(x + 14, y + 6, color);
        self.draw_pixel(x + 15, y + 5, color);
        self.draw_pixel(x + 15, y + 3, color);
        self.draw_pixel(x + 15, y + 4, color);
        
        // C
        self.draw_pixel(x + 18, y + 4, color);
        self.draw_pixel(x + 18, y + 5, color);
        self.draw_pixel(x + 19, y + 3, color);
        self.draw_pixel(x + 19, y + 6, color);
        self.draw_pixel(x + 20, y + 3, color);
        self.draw_pixel(x + 20, y + 6, color);
    }

    /// Draw a timeout indicator box with BMS label
    pub fn write_bms_timeout(&mut self, time_since: u32) {
        let x = 8 * FONT_WIDTH;
        let y = 3 * FONT_HEIGHT + 4;
        const TIMEOUT: u32 = 1000;
        
        let dead = time_since >= TIMEOUT;
        let ratio = if dead {
            1.0
        } else {
            time_since as f32 / TIMEOUT as f32
        };
        
        // Fill bar
        for i in (x + 1)..(x + (30.0 * ratio) as usize) {
            for j in (y + 1)..(y + 9) {
                let shade = if dead {
                    DISPLAY_VLOW_SHADE
                } else {
                    DISPLAY_MID_SHADE
                };
                self.draw_pixel(i, j, shade);
            }
        }
        
        // Draw X if dead
        if dead {
            for i in 0..30 {
                let y1 = (i * 10) / 30;
                let y2 = ((29 - i) * 10) / 30;
                self.draw_pixel(x + i, y + y1, DISPLAY_MID_SHADE);
                self.draw_pixel(x + i, y + y2, DISPLAY_MID_SHADE);
            }
        }
        
        // Draw box outline
        self.draw_box_outline(x, y, 30, 10);
        
        // Draw "BMS" label (simplified pixel art)
        let color = DISPLAY_WHITE;
        // B
        self.draw_pixel(x + 10, y + 2, color);
        self.draw_pixel(x + 10, y + 3, color);
        self.draw_pixel(x + 10, y + 4, color);
        self.draw_pixel(x + 10, y + 5, color);
        self.draw_pixel(x + 10, y + 6, color);
        self.draw_pixel(x + 11, y + 2, color);
        self.draw_pixel(x + 11, y + 4, color);
        self.draw_pixel(x + 11, y + 6, color);
        self.draw_pixel(x + 12, y + 3, color);
        self.draw_pixel(x + 12, y + 5, color);
        
        // M
        self.draw_pixel(x + 14, y + 2, color);
        self.draw_pixel(x + 14, y + 3, color);
        self.draw_pixel(x + 14, y + 4, color);
        self.draw_pixel(x + 14, y + 5, color);
        self.draw_pixel(x + 14, y + 6, color);
        self.draw_pixel(x + 15, y + 3, color);
        self.draw_pixel(x + 16, y + 2, color);
        self.draw_pixel(x + 16, y + 3, color);
        self.draw_pixel(x + 16, y + 4, color);
        self.draw_pixel(x + 16, y + 5, color);
        self.draw_pixel(x + 16, y + 6, color);
        
        // S
        self.draw_pixel(x + 18, y + 2, color);
        self.draw_pixel(x + 19, y + 2, color);
        self.draw_pixel(x + 18, y + 3, color);
        self.draw_pixel(x + 18, y + 4, color);
        self.draw_pixel(x + 19, y + 4, color);
        self.draw_pixel(x + 19, y + 5, color);
        self.draw_pixel(x + 18, y + 6, color);
        self.draw_pixel(x + 19, y + 6, color);
    }

    /// Helper function to draw a box outline
    fn draw_box_outline(&mut self, x0: usize, y0: usize, width: usize, height: usize) {
        // Draw top and bottom edges
        for x in x0..(x0 + width) {
            self.draw_pixel(x, y0, DISPLAY_WHITE);
            self.draw_pixel(x, y0 + height - 1, DISPLAY_WHITE);
        }
        
        // Draw left and right edges
        for y in y0..(y0 + height) {
            self.draw_pixel(x0, y, DISPLAY_WHITE);
            self.draw_pixel(x0 + width - 1, y, DISPLAY_WHITE);
        }
    }

    /// Draw BMS flash indicator
    pub fn write_bms_flash(&mut self, _bps_strobe: bool, _flash: &mut bool, _last_flash: &mut u32, _current_time: u32) {
        // TODO: Implement BMS flash indicator
        // This would flash an indicator when the BPS (Battery Protection System) is active
    }

    /// Write debug screen header
    pub fn write_debug(&mut self) {
        let x = 0 * FONT_WIDTH;
        let y = 0 * FONT_HEIGHT;
        
        self.draw_string(x, y, DISPLAY_WHITE, DISPLAY_BLACK, "Debug");
    }

    /// Write throttle debug info
    pub fn write_throttle_debug(&mut self, throttle: f32, raw_throttle: u16) {
        let x = 10 * FONT_WIDTH;
        let y = 2 * FONT_HEIGHT;
        
        let mut buf1: String<16> = String::new();
        write!(&mut buf1, "Thr:{:0.3}", throttle).ok();
        self.draw_string(x.saturating_sub(9 * FONT_WIDTH), y, DISPLAY_WHITE, DISPLAY_BLACK, &buf1);
        
        let mut buf2: String<16> = String::new();
        write!(&mut buf2, "RAW:{}", raw_throttle).ok();
        self.draw_string(x, y, DISPLAY_WHITE, DISPLAY_BLACK, &buf2);
    }

    /// Write regen debug info
    pub fn write_regen_debug(&mut self, regen: f32, raw_regen: u32) {
        let x = 10 * FONT_WIDTH;
        let y = 3 * FONT_HEIGHT;
        
        let mut buf1: String<16> = String::new();
        write!(&mut buf1, "Reg:{:0.3}", regen).ok();
        self.draw_string(x.saturating_sub(9 * FONT_WIDTH), y, DISPLAY_WHITE, DISPLAY_BLACK, &buf1);
        
        let mut buf2: String<16> = String::new();
        write!(&mut buf2, "RAW:{}", raw_regen).ok();
        self.draw_string(x, y, DISPLAY_WHITE, DISPLAY_BLACK, &buf2);
    }

    /// Write pedal value debug info
    pub fn write_pedal_value(&mut self, pedal: f32, raw_pedal: u32) {
        let x = 8 * FONT_WIDTH;
        let y = 0 * FONT_HEIGHT;
        
        let mut buf1: String<16> = String::new();
        write!(&mut buf1, "Ped:{:5.1}", pedal).ok();
        self.draw_string(x, y, DISPLAY_WHITE, DISPLAY_BLACK, &buf1);
        
        let x2 = 4 * FONT_WIDTH;
        let y2 = 1 * FONT_HEIGHT;
        
        let mut buf2: String<16> = String::new();
        write!(&mut buf2, "RAW Ped:{:5}", raw_pedal).ok();
        self.draw_string(x2, y2, DISPLAY_WHITE, DISPLAY_BLACK, &buf2);
    }
}

