# GDB Support Issues

## GDB Stepping Crash at Boot ROM (0x20000000)

### Symptoms
1. GDB connects successfully to Terminus
2. GDB reads registers - gets back RLE-encoded packet
3. Immediately after, connection error: "Resource temporarily unavailable"
4. GDB crashes with segfault in `compute_frame_id`

### Root Cause
- Boot ROM is mapped at `0x20000000` (see `src/bin/terminus.rs` line 489)
- The boot ROM contains reset vector code that jumps to the kernel entry point
- When GDB tries to analyze the stack frame at `0x20000000`, it has no symbols
- GDB's `frame.c` crashes in `compute_frame_id` when computing frame boundaries without debug info

### Code Locations

**Boot ROM creation** (`src/system/mod.rs` lines 466-498):
```rust
pub fn make_boot_rom(&mut self, base: u64, entry: u64, boot_args: Vec<&str>) -> Result<()> {
    let start_address = if entry == -1i64 as u64 {
        self.elf.entry_point().unwrap()  // Gets actual kernel entry
    } else {
        entry
    };
    // ... reset vector code that jumps to start_address
}
```

**Boot ROM mapping** (`src/bin/terminus.rs` line 489):
```rust
sys.make_boot_rom(0x20000000, -1i64 as u64, boot_args)
    .unwrap();
```

**Reset behavior** (`src/system/mod.rs` lines 500-525):
```rust
pub fn reset(&mut self, reset_vecs: Vec<u64>) -> Result<()> {
    // ...
    let entry_point = self.elf.entry_point().unwrap();  // Kernel entry stored here
    for (i, p) in self.processors().iter_mut().enumerate() {
        if reset_vecs[i] == -1i64 as u64 {
            if let Some(ref boot_rom) = boot_rom {
                p.reset(boot_rom.info.base)  // PC = 0x20000000 (boot ROM)
            } else {
                p.reset(entry_point)  // PC = kernel entry
            }
        }
    }
}
```

### Workarounds

1. **Use continue with breakpoint instead of stepi**:
   ```
   (gdb) break *0x80000000
   (gdb) continue
   ```

2. **Manually set PC to kernel entry after connecting**:
   ```
   (gdb) set $pc = 0x80000000
   (gdb) stepi
   ```

3. **Use a GDB script** to automatically set PC on connect

### Potential Fix

The `ElfLoader` already has an `entry_point()` method (`src/system/elf.rs` lines 79-81) that reads the kernel entry from the ELF header.

When GDB connects, Terminus could:
1. Detect GDB connection
2. Automatically set PC to `elf.entry_point()` instead of boot ROM base

This would require modifying the GDB session handling to set initial PC registers to the kernel entry point rather than leaving them at boot ROM.

### Kernel Entry Point
The actual kernel entry is typically at `0x80000000` (or similar) and can be obtained via:
```rust
let entry = self.elf.entry_point().unwrap();
```

### Related Files
- `src/bin/terminus.rs` - Boot ROM creation and GDB session handling
- `src/system/mod.rs` - System reset and boot ROM mapping
- `src/system/elf.rs` - ELF loading and entry point reading
