# xv6 Kernel Image Build & Test

This repository builds the xv6 kernel image (`xv6.img`) using a custom `xtask` command.

---

## Build Instructions

You need Rust (nightly) and `rust-objcopy` installed.

```bash
# Build xv6.img with release profile
cargo run -p xtask -- image --profile release
```

## License

This project includes or is derived from the original xv6 kernel code:

> SPDX-License-Identifier: MIT
> Copyright (c) 2006-2018 Frans Kaashoek, Robert Morris, Russ Cox, Massachusetts Institute of Technology

The original xv6 code is provided under a permissive license.
Please refer to the `LICENSE` file for full details.

Your modifications and build scripts are provided under MIT.
