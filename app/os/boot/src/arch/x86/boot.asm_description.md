# Instructions & Glossary

## AT&T x86 Assembly Instructions (used in `boot.S`)

| Instruction | Signature                 | Description                                                                  |
| ----------- | ------------------------- | ---------------------------------------------------------------------------- |
| `cli`       | `cli`                     | Clear interrupt flag. Disables maskable hardware interrupts.                 |
| `xor`       | `xorw %ax, %ax`           | Sets `%ax` to zero (XOR a register with itself is a common zeroing idiom).   |
| `mov`       | `movw %ax, %ds`           | Move data from `%ax` to segment register `%ds`.                              |
| `inb`       | `inb $port, %al`          | Read a byte from I/O port into `%al`.                                        |
| `outb`      | `outb %al, $port`         | Write a byte from `%al` to I/O port.                                         |
| `testb`     | `testb $mask, %al`        | Bitwise AND `%al` with `mask`, sets flags only (used for polling bits).      |
| `jnz`       | `jnz label`               | Jump if Zero Flag is _not_ set (i.e., result was non-zero).                  |
| `lgdt`      | `lgdt gdtdesc`            | Load the Global Descriptor Table register (GDTR) with pointer `gdtdesc`.     |
| `movl`      | `movl %cr0, %eax`         | Move from control register CR0 into `%eax`.                                  |
| `orl`       | `orl $0x1, %eax`          | Bitwise OR `%eax` with 1 (used to set PE bit for protected mode).            |
| `ljmp`      | `ljmp $selector, $offset` | Far jump: loads CS with `selector`, and EIP with `offset` (enters pmode).    |
| `call`      | `call symbol`             | Call a procedure (`bootmain` in this case).                                  |
| `outw`      | `outw %ax, %dx`           | Write a word from `%ax` to I/O port specified in `%dx`.                      |
| `jmp`       | `jmp label`               | Unconditional jump to `label`.                                               |
| `.word`     | `.word value`             | Embed a 16-bit word value in output.                                         |
| `.long`     | `.long value`             | Embed a 32-bit value in output.                                              |
| `.byte`     | `.byte value`             | Embed an 8-bit value in output.                                              |
| `.macro`    | `.macro name args...`     | Define an assembler macro.                                                   |
| `.set`      | `.set symbol, value`      | Define a constant (symbol) to a specific value.                              |
| `.globl`    | `.globl symbol`           | Make symbol visible outside this file (like a public function).              |
| `.code16`   | `.code16`                 | Switch to 16-bit code mode for assembler output.                             |
| `.code32`   | `.code32`                 | Switch to 32-bit code mode for assembler output.                             |
| `.p2align`  | `.p2align N`              | Align the next instruction/data to 2^N bytes (e.g., `.p2align 2` → 4B align) |

---

## Notes on AT&T Syntax

- Operand order is **source → destination** (e.g., `movw %ax, %ds` moves `%ax` to `%ds`).
- Registers are prefixed with `%` (e.g., `%ax`, `%ds`).
- Immediate values are prefixed with `$` (e.g., `$0x1`).
- Some instructions like `inb`, `outb`, `ljmp` are specific to I/O port and CPU control operations.

---

## ===== Glossary & Notes =====

- Real Mode:
  The initial CPU mode after reset; 16-bit segmented addressing, limited to 1MB addressable memory.

- Protected Mode:
  32-bit mode with advanced features like virtual memory, paging, and hardware-level memory protection.

- A20 Line: (A20: Address of 20 bit. 0xFFFFF)
  A CPU address line that must be enabled to access memory beyond 1MB (beyond address 0xFFFFF).
  Historically disabled for compatibility with old software.

- GDT (Global Descriptor Table):
  A data structure used by the CPU in protected mode to define memory segments,
  including their base address, size, and access permissions.

- Segment Registers (%cs, %ds, %es, %ss, %fs, %gs):
  CPU registers that hold selectors into the GDT, defining the currently used code and data segments.

- CLI (Clear Interrupt Flag):
  An instruction to disable interrupts temporarily during critical setup.

- LGDT:
  Loads the GDT register with the address and size of the GDT table.

- LJMP (Long Jump):
  Far jump that loads a new code segment and instruction pointer; required to enter protected mode.

- I/O Ports (inb/outb):
  Instructions to communicate with hardware devices via I/O ports, used here to enable A20 and interact with disk/controller.

## ===== Global Descriptor Table (GDT) =====

The GDT is a critical data structure used by the CPU in protected mode to
define the characteristics of the various memory segments, including their
base address, size (limit), and access permissions.

Why is this needed?

---

When switching from real mode (simple 16-bit segmented addressing) to protected
mode (32-bit mode with advanced memory management), the CPU relies on the GDT
to understand how memory is organized and what permissions apply.

Each entry in the GDT describes a segment. The CPU uses segment selectors (values
loaded into segment registers like CS, DS, ES) as indexes into this table to
find segment details.

The GDT must be properly aligned and its address/size loaded into the CPU using
the `LGDT` instruction before entering protected mode.

Structure of this GDT:

---

1. `SEG_NULLASM`:
   The first entry must be a null descriptor (all zeros). This is a requirement
   of the x86 architecture and serves as a "null segment" to catch invalid segment
   accesses.

2. `SEG_ASM STA_X | STA_R, 0x0, 0xffffffff`:
   Defines a Code Segment descriptor covering the full 4GB address space (0x0 to
   0xFFFFFFFF). The segment is executable and readable, suitable for kernel code.

3. `SEG_ASM STA_W, 0x0, 0xffffffff`:
   Defines a Data Segment descriptor covering the full 4GB address space, writable
   and suitable for kernel data.

Why `.p2align 2`?

---

`.p2align 2` ensures the GDT starts at an address aligned to 4 bytes (2^2 = 4),
which is required for performance and correctness on x86 CPUs.

### About `gdtdesc`

---

This is a special structure that holds the size (limit) and linear address of the
GDT itself. It is loaded into the CPU's `GDTR` register with the LGDT instruction.

The size is defined as (sizeof(GDT) - 1) because the limit field in `GDTR` is
zero-based (max index), not size.

Example:
.word (gdtdesc - gdt - 1) # size of GDT minus one
.long gdt # linear address of GDT

The CPU reads this descriptor to know where the GDT is in memory and how large it is,
allowing it to correctly interpret segment selectors during protected mode execution.

Without this properly defined GDT and gdtdesc loaded, the CPU cannot switch to
protected mode correctly, leading to faults or undefined behavior.

```asm
.p2align 2
gdt:
    SEG_NULLASM                            // Null segment descriptor (required)
    SEG_ASM STA_X | STA_R, 0x0, 0xffffffff  // Code segment: base=0, limit=4GB, executable + readable
    SEG_ASM STA_W,        0x0, 0xffffffff  // Data segment: base=0, limit=4GB, writable

gdtdesc:
    .word (gdtdesc - gdt - 1)  // Size of GDT minus 1 (limit)
    .long gdt                  // Address of GDT in memory
```
