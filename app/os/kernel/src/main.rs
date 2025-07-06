#![no_std]
#![no_main]
// #![feature(lang_items)]

mod entry;
mod params;

use core::fmt::{self, Write};

struct SerialPort;

impl Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            unsafe {
                // Data register (0x3F8) of QEMU's standard COM1 serial port
                let com1: *mut u8 = core::ptr::without_provenance_mut(0x3F8);
                core::ptr::write_volatile(com1, byte);
            }
        }
        Ok(())
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    let mut serial = SerialPort;
    writeln!(serial, "Hello, xv6 Rust!").unwrap();

    qemu_exit(0);
}

fn qemu_exit(code: u8) -> ! {
    unsafe {
        core::arch::asm!("out dx, al", in("dx") 0x501u16, in("al") code);
    }
    loop {}
}

#[cfg(not(test))]
#[panic_handler]
pub fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

// #[cfg(not(test))]
// #[lang = "eh_personality"]
// pub extern "C" fn eh_personality() {}
