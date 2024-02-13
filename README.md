# h7

This is largely adapted from [olback/h7](https://github.com/olback/h7).

## Setup

```

// 
rustup component add llvm-tools-preview
rust-nm -S ./h7-cm7/target/thumbv7em-none-eabihf/release/h7-cm7 | grep "RTT"
2000001c 00000030 D _SEGGER_RTT
```

## Minimum supported Rust version

The Minimum Supported Rust Version (MSRV) at the moment is rustc **1.75.0-nightly** (aa1a71e9e 2023-10-26).

## License

Refer to LICENSE.