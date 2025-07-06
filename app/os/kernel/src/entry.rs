use core::arch::global_asm;

use crate::params::K_STACK_SIZE;

global_asm!(
    r#"
.section .text

.align 4
.global multiboot_header
multiboot_header:
    .long 0x1BADB002
    .long 0
    .long -(0x1BADB002 + 0)

.global _start
_start:
    jmp entry_phys

.global entry
entry:
    mov eax, cr4
    or eax, 0x10          # CR4_PSE
    mov cr4, eax

    mov eax, offset ENTRY_PG_DIR
    sub eax, 0x80000000   # V2P_WO(ENTRY_PG_DIR)
    mov cr3, eax

    mov eax, cr0
    or eax, 0x80010000    # CR0_PG | CR0_WP
    mov cr0, eax

    mov esp, offset STACK_TOP

    mov eax, offset main
    jmp eax

entry_phys:
    jmp entry
"#
);

// Dummy symbol definitions for linking
#[unsafe(no_mangle)]
pub static mut ENTRY_PG_DIR: [u32; 1024] = [0; 1024];

#[unsafe(no_mangle)]
#[unsafe(link_section = ".bss.stack")]
static mut STACK: [u8; K_STACK_SIZE] = [0; K_STACK_SIZE];

#[unsafe(no_mangle)]
pub static STACK_TOP: u32 = 0x0; // patched later if needed
