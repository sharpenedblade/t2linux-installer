# t2linux-installer

A desktop installer helper for Linux on T2 Macs.

It helps you:
- Pick a distro from t2linux metadata
- Download the ISO to a folder
- Or write directly to a removable USB/block device

## Features

- Distro list fetched from t2linux metadata
- Download progress with cancel support
- Optional direct write to removable disks
- Existing ISO at the same output path is removed before writing
- Device picker hides non-removable/system disks on Linux
- Finish screen shows:
  - `Your USB is ready to boot` when flashing a USB/block device
  - `Download Complete` for file downloads

## Requirements

- Rust toolchain (`cargo`, stable)
- Linux or macOS
- Platform tools used by the app:
  - Linux: `lsblk`
  - macOS: `diskutil`

## Build and Run

### Dev build
```bash
cargo run
```

### Release binary
```bash
cargo build --release
```

Binary path:
- `target/release/t2linux-installer`

## Project Structure

- `src/ui/` - app pages and UI flow
- `src/disk/` - disk detection per platform
- `src/distro.rs` - distro metadata + ISO download logic
- `src/install.rs` - install/download orchestration
- `assets/` - UI assets (finish screen icon)

## Contributing

1. Create a branch
2. Make changes
3. Run checks:
   ```bash
   cargo check
   cargo check --release
   ```
4. Open a PR with a clear summary and testing notes
