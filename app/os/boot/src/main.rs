#![cfg_attr(not(test), no_std)]
#![no_main]

pub mod arch;

#[cfg(not(test))]
#[panic_handler]
pub fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
