# devShell for greenhouse_esp32: bare-metal esp-hal + embassy firmware for the
# ESP32-C3 (target riscv32imc-unknown-none-elf, tier 2 → prebuilt core/alloc,
# so plain stable Rust works; no ESP-IDF, no build-std).
# Keep the version in sync with greenhouse_esp32/rust-toolchain.toml.
{
  mkShell,
  rust-bin,
  espflash,
}:

let
  toolchain = rust-bin.stable."1.99.1".default.override {
    extensions = [
      "rust-src"
      "rust-analyzer"
    ];
    targets = [ "riscv32imc-unknown-none-elf" ];
  };
in
mkShell {
  packages = [
    toolchain
    espflash # flash + monitor: `cargo run --release` (runner in .cargo/config.toml)
  ];
}
