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
_start = [entry - {kernel_base}]   # addr - KERNEL_BASE: Virt to Phys addr

.global entry
entry:
    mov eax, cr4
    or eax, 0x10          # CR4_PSE
    mov cr4, eax

    # Set page dir
    mov eax, [ENTRY_PG_DIR - {kernel_base}] # eax = virt_to_phys_addr(ENTRY_PG_DIR)
    mov cr3, eax

    # Turn on paging
    mov eax, cr0
    or eax, 0x80010000    # CR0_PG | CR0_WP
    mov cr0, eax

    mov esp, [STACK - {k_stack_size}]

    mov eax, offset main
    jmp eax
"#,
    kernel_base = const KERNEL_BASE,
    k_stack_size = const K_STACK_SIZE,
);

#[unsafe(no_mangle)]
static ENTRY_PG_DIR: AlignedPdeArray = make_entry_pg_dir();

const N_PD_ENTRIES: usize = 1024;
const PTE_P: u32 = 0x001; // Present
const PTE_W: u32 = 0x002; // Writable
const PTE_PS: u32 = 0x080; // Page size (4MB)
const KERNEL_BASE: usize = 0x8000_0000; // 例
/// 4MiB shift
const PDX_SHIFT: usize = 22;

#[allow(unused)]
#[repr(align(4096))]
struct AlignedPdeArray([u32; N_PD_ENTRIES]);

const fn make_entry_pg_dir() -> AlignedPdeArray {
    let mut arr = [0u32; N_PD_ENTRIES];
    arr[0] = (0) | PTE_P | PTE_W | PTE_PS;
    arr[KERNEL_BASE >> PDX_SHIFT] = (0) | PTE_P | PTE_W | PTE_PS;
    AlignedPdeArray(arr)
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".bss.stack")]
static mut STACK: [u8; K_STACK_SIZE] = [0; K_STACK_SIZE];
