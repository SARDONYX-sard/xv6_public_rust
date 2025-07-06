#![no_std]
#![no_main]
// #![feature(lang_items)]

// kernel entry point example
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    {
        loop {
            {}
        }
    }
}

#[cfg(not(test))]
#[panic_handler]
pub fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

// #[cfg(not(test))]
// #[lang = "eh_personality"]
// pub extern "C" fn eh_personality() {}
