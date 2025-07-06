#[doc = include_str!("./boot.asm_description.md")]
mod boot;

mod consts;

use self::consts::*;
use core::ptr::{read_volatile, write_volatile};

use elf::arch::x86::{ELF_MAGIC, ElfHeader, ProgramHeader};

// core::arch::global_asm!(include_str!("./boot.S"));

/// Bootloader main function.
///
/// - Loads ELF binary from disk (starting at sector 1) into memory.
/// - Parses and loads each segment into its physical location.
/// - Fills uninitialized segment memory with zero.
/// - Jumps to the kernel's entry point.
#[unsafe(no_mangle)]
pub extern "C" fn bootmain() {
    let elf_ptr: *mut ElfHeader = core::ptr::without_provenance_mut(0x10000);
    unsafe {
        read_segment(elf_ptr.cast::<u8>(), 4096, 0);

        if read_volatile(&(*elf_ptr).magic) != ELF_MAGIC {
            return; // fallback to spin loop
        }

        let program_header: *const ProgramHeader = {
            let program_header_offset = (*elf_ptr).program_header_offset as usize;
            elf_ptr.byte_add(program_header_offset).cast()
        };
        let header_count = read_volatile(&(*elf_ptr).program_header_entry_size) as usize;

        for n in 0..header_count {
            let segment = program_header.add(n);

            let dest = read_volatile(&(*segment).physical_address) as *mut u8;
            let file_size = read_volatile(&(*segment).file_size) as usize;
            let mem_size = read_volatile(&(*segment).memory_size) as usize;
            let file_offset = read_volatile(&(*segment).offset) as usize;

            read_segment(dest, file_size, file_offset);

            if mem_size > file_size {
                stosb(dest.add(file_size), 0, mem_size - file_size);
            }
        }

        let entry = read_volatile(&(*elf_ptr).entry) as usize;
        let entry_fn: extern "C" fn() = core::mem::transmute(entry);
        entry_fn();
    }
}

/// Reads `count` bytes from disk starting at `file_offset` into physical memory `phys_addr`.
///
/// - The disk is read sector-by-sector (512 bytes).
/// - The memory destination is aligned backward to the sector boundary if `file_offset` is not aligned.
/// - The actual data loaded may be more than `count`, which is acceptable during boot.
#[inline(never)] // NOTE: Since inline expansion by the compiler would exceed the 510-byte limit, prevent inline expansion.
unsafe fn read_segment(phys_addr: *mut u8, bytes_count: usize, file_offset: usize) {
    // Byte misalignment from the beginning of a sector
    let offset_into_sector = file_offset % SECTOR_SIZE;

    // Adjust destination pointer backward to align with sector start
    let aligned_dest = unsafe { phys_addr.offset(-(offset_into_sector as isize)) };

    // Starting sector index
    let start_sector = file_offset / SECTOR_SIZE;

    // Calculate the number of sectors needed to cover the range (rounded up)
    let end_offset = file_offset + bytes_count;
    let total_sectors = (end_offset + SECTOR_SIZE - 1) / SECTOR_SIZE - start_sector;

    // Read each sector into memory
    for i in 0..total_sectors {
        let target = unsafe { aligned_dest.add(i * SECTOR_SIZE) };
        let sector_number = start_sector + i;
        unsafe { read_sector(target, sector_number) };
    }
}

/// Reads a single 512-byte sector from the disk into the given memory address `dst`.
///
/// Uses **ATA PIO mode with 28-bit Logical Block Addressing (LBA28)**, which is a legacy method
/// for interacting with disks. This is intentionally chosen for **educational simplicity**.
///
/// # Why this "slow" method is used:
///
/// - ✅ **Simplicity over Speed**: PIO mode is slow but easy to implement.
///   - Direct access to I/O ports (e.g., `outb`, `inb`)—no complex controller initialization needed.
/// - ✅ **No hardware discovery or drivers required**:
///   - AHCI or NVMe require enumerating PCI devices, handling BARs, and memory-mapped I/O.
/// - ✅ **Works in QEMU and Bochs**:
///   - Emulators fully support legacy ATA PIO mode with LBA28, which ensures reproducibility.
/// - ✅ **Suitable for 16-bit real mode and early boot**:
///   - No paging or virtual memory is needed; segment+offset flat memory works fine.
///
/// # Limitations:
/// - ❌ Slow (uses CPU for data transfer instead of DMA).
/// - ❌ Limited to 128GiB drives (28-bit LBA address space).
/// - ❌ Not suitable for modern OSes beyond the bootloader stage.
///
/// # ATA I/O Port Map (ISA Legacy I/O Ports):
/// - `0x1F2`: Sector count (1)
/// - `0x1F3`: LBA bits 0–7
/// - `0x1F4`: LBA bits 8–15
/// - `0x1F5`: LBA bits 16–23
/// - `0x1F6`: LBA bits 24–27 (lower 4 bits) + Drive select (bit 4)
/// - `0x1F7`: Command port (0x20 = read)
/// - `0x1F0`: Data register (read sector as 32-bit chunks)
///
/// # Modern Alternatives:
/// For a modern OS or kernel:
/// - Use **AHCI** (SATA) via PCIe and memory-mapped registers
/// - Use **NVMe** for faster performance and better queuing
///
/// These alternatives are intentionally avoided in early boot_loaders and educational OSes.
///
/// # Safety
/// This function performs raw pointer writes and I/O port manipulation.
#[inline]
unsafe fn read_sector(dst: *mut u8, sector_offset: usize) {
    // See(&Find `48 bit PIO`): https://wiki.osdev.org/ATA_PIO_Mode
    unsafe {
        wait_disk();

        // Set sector count (we want to read 1 sector)
        outb(ATA_SEC_CNT, 1);

        // Send 28-bit LBA address split across 4 ports
        outb(ATA_LBA_LO, (sector_offset & 0xFF) as u8); // LBA bits 0–7
        outb(ATA_LBA_MID, ((sector_offset >> 8) & 0xFF) as u8); // LBA bits 8–15
        outb(ATA_LBA_HI, ((sector_offset >> 16) & 0xFF) as u8); // LBA bits 16–23
        outb(ATA_DRIVE, ((sector_offset >> 24) as u8) | ATA_DRIVE_MASTER); // Bits 24–27 + master bit

        // Send READ SECTORS command
        outb(ATA_STATUS, ATA_CMD_READ);

        // Read the 512 bytes (as 128 u32s) from data port
        // `/4` because `insl` reads in 4-byte units (32-bit words).
        insl(ATA_DATA, dst, SECTOR_SIZE / 4);

        // Wait for disk to finish
        wait_disk();
    }
}

/// Waits until the disk controller is ready to accept commands.
///
/// Reads the status byte from I/O port `0x1F7` (ATA status register),
/// and waits until the drive is no longer busy (BSY=0)
/// and is ready to transfer data (DRDY=1).
///
/// # ATA Status Bits (0x1F7)
/// - Bit 7 (BSY): Busy flag
/// - Bit 6 (DRDY): Drive ready
///
/// This function loops until `(status & 0xC0) == 0x40`.
#[inline]
unsafe fn wait_disk() {
    while (unsafe { inb(0x1F7) } & (ATA_STATUS_BSY | ATA_STATUS_RDY)) != ATA_STATUS_RDY {}
}

/// Fills a memory region starting at `dst` with `count` bytes of `data`.
///
/// Uses `write_volatile` to ensure the writes are not optimized out by the compiler.
#[inline]
unsafe fn stosb(mut dst: *mut u8, data: u8, count: usize) {
    for _ in 0..count {
        unsafe {
            write_volatile(dst, data);
            dst = dst.add(1);
        }
    }
}

// ===== Low-Level I/O Port Access =====

/// Reads a byte from the specified I/O port.
///
/// # Safety
/// This uses inline assembly and is inherently unsafe.
#[inline]
unsafe fn inb(port: u16) -> u8 {
    let ret: u8;
    unsafe { core::arch::asm!("in al, dx", in("dx") port, out("al") ret) };
    ret
}

/// Writes a byte to the specified I/O port.
///
/// # Safety
/// This uses inline assembly and is inherently unsafe.
#[inline]
unsafe fn outb(port: u16, val: u8) {
    unsafe { core::arch::asm!("out dx, al", in("dx") port, in("al") val) };
}

/// Reads `cnt` double words (4 bytes) from the given I/O port into memory at `addr`.
///
/// This uses the `rep insl` instruction to transfer `cnt` DWORDs (4-byte values) efficiently.
#[inline]
unsafe fn insl(port: u16, addr: *mut u8, cnt: usize) {
    unsafe {
        core::arch::asm!(
            "cld",          // Clear direction flag
            "rep insd",     // Repeat input DWORDs (cnt times)
            in("dx") port,
            inout("edi") addr as usize => _,
            inout("ecx") cnt => _,
            options(nostack, preserves_flags),
        );
    }
}
