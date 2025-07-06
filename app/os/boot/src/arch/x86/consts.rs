/// I/O port for reading/writing data (16-bit or 32-bit I/O).
pub const ATA_DATA: u16 = 0x1F0;

/// I/O port to specify the number of sectors to read or write.
pub const ATA_SEC_CNT: u16 = 0x1F2;

/// Low byte of the 28-bit Logical Block Address (LBA).
pub const ATA_LBA_LO: u16 = 0x1F3;

/// Mid byte of the 28-bit Logical Block Address (LBA).
pub const ATA_LBA_MID: u16 = 0x1F4;

/// High byte of the 28-bit Logical Block Address (LBA).
pub const ATA_LBA_HI: u16 = 0x1F5;

/// Drive/head register. Use 0xE0 for master drive in LBA mode.
pub const ATA_DRIVE: u16 = 0x1F6;

/// Command and status register.
/// Writing issues a command; reading returns status.
pub const ATA_STATUS: u16 = 0x1F7;

/// ATA command: Read sectors using 28-bit LBA.
pub const ATA_CMD_READ: u8 = 0x20;

/// Drive/Head register value for master drive with LBA addressing (used in ATA PIO mode).
///
/// Bit layout of 0x1F6 port:
/// ```text
///  7 6 5 4 3 2 1 0
/// +---------------+
/// |1|1|1|0|0|0|0|0| = 0xE0
///  | | | |
///  | | | └──── 4bits LBA[27:24]
///  | | └───────── bit5: Drive select: 0 = master, 1 = slave
///  | └──────────── bit6: LBA mode enable (1 = LBA, 0 = CHS).
///  └────────────── bit7: Reserved/legacy. Must be set to 1 (always)
/// ```
///
/// [See this](https://wiki.osdev.org/ATA_PIO_Mode#:~:text=the%20Primary%20bus%3A-,Send%200xE0%20for%20the%20%22master%22%20or%200xF0%20for%20the%20%22slave%22%2C%20ORed%20with%20the%20highest%204%20bits%20of%20the%20LBA%20to%20port%200x1F6%3A%20outb(0x1F6%2C%200xE0%20%7C%20(slavebit%20%3C%3C%204)%20%7C%20((LBA%20%3E%3E%2024)%20%26%200x0F)),-Send%20a%20NULL)
/// > `Send 0xE0 for the "master" or 0xF0 for the "slave", ORed with the highest 4 bits of the LBA to port 0x1F6: outb`
pub const ATA_DRIVE_MASTER: u8 = 0xE0;

/// ATA Status Register bit: Drive is busy (1 = busy).
pub const ATA_STATUS_BSY: u8 = 1 << 7;

/// ATA Status Register bit: Drive is ready (1 = ready).
pub const ATA_STATUS_RDY: u8 = 1 << 6;

/// Size of a disk sector in bytes (512 bytes).
///
/// One sector of a floppy disk or HDD is 512 bytes, and the boot loader is stored in the first sector.
pub const SECTOR_SIZE: usize = 512;
