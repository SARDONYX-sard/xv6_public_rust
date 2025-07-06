// SPDX-License-Identifier: MIT
// Copyright (c) 2006-2018 Frans Kaashoek, Robert Morris, Russ Cox, Massachusetts Institute of Technology
//! boot.asm - x86 Boot Loader (Entry & Mode Switch)
//!
//! This file is the very first code run by CPU after BIOS loads the boot sector.
//! It performs these key tasks sequentially:
//!
//! - 1/12: Start in 16-bit Real Mode and define entry point.
//! - 2/12: Disable interrupts.
//! - 3/12: Clear segment registers in real mode.
//! - 4/12: Enable the A20 line to access memory above 1MB.
//! - 5/12: Load and configure the Global Descriptor Table (GDT).
//! - 6/12: Enable Protected Mode via CR0 PE bit.
//! - 7/12: Far jump to flush pipeline and enter Protected Mode.
//! - 8/12: Switch assembler to 32-bit mode.
//! - 9/12: Setup data segment registers for Protected Mode.
//! - 10/12: Setup temporary stack and call Rust entry point bootmain.
//! - 11/12: If bootmain returns, halt CPU in an infinite loop.
//! - 12/12: Define GDT table and descriptors using macros.

core::arch::global_asm!(
    r#"
.macro SEG_NULLASM
    .long 0
    .long 0
.endm

.macro SEG_ASM access, base, limit
    .word ((\limit >> 12) & 0xFFFF)
    .word ((\base) & 0xFFFF)
    .byte (((\base) >> 16) & 0xFF)
    .byte (0x90 | ((\access) & 0xF))
    .byte (0xC0 | (((\limit) >> 28) & 0xF))
    .byte (((\base) >> 24) & 0xFF)
.endm

.global start
.extern bootmain

# 1/12: Real Mode, entry point setup
.code16
start:
    # 2/12: Disable interrupts
    cli

    # 3/12: Clear segment registers (required in real mode)
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax

    # 4/12: Enable A20 line to allow addressing >1MB memory
seta20_1:
    in al, 0x64
    test al, 0x2
    jnz seta20_1

    mov al, 0xd1
    out 0x64, al

seta20_2:
    in al, 0x64
    test al, 0x2
    jnz seta20_2

    mov al, 0xdf
    out 0x60, al

    # 5/12: Load Global Descriptor Table (GDT)
    lgdt [gdtdesc]

    # 6/12: Enable Protected Mode (set PE bit in CR0)
    mov eax, cr0
    or eax, 0x1
    mov cr0, eax

    # 7/12: Far jump to flush pipeline and reload CS for Protected Mode
    jmp [start32 + 0x08]

.code32
start32:
    # 9/12: Setup segment registers for protected mode
    mov ax, 0x10  # (2 << 3)
    mov ds, ax
    mov es, ax
    mov ss, ax

    xor ax, ax
    mov fs, ax
    mov gs, ax

    # 10/12: Setup temporary stack and call Rust bootmain
    mov esp, start
    call bootmain

    # 11/12: If bootmain returns (should not), halt CPU in infinite loop
    mov ax, 0x8a00
    mov dx, ax
    out dx, ax
    mov ax, 0x8ae0
    out dx, ax

    spin:
        jmp spin

.align 4
gdt:
    .long 0 # SEG_NULL (It seems that we need to enter null at first.)
    .long 0 # SEG_NULL
    SEG_ASM 0xA | 0x2, 0x00000000, 0xFFFFFFFF  # Code Segment
    SEG_ASM 0x2, 0x00000000, 0xFFFFFFFF        # Data Segment

gdtdesc:
    .word gdtdesc - gdt - 1
    .long gdt
"#
);
