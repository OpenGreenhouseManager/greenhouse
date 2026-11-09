# The toolchain pinned in rust-toolchain.toml (stable, plus the ESP32-C3 target),
# with rust-analyzer for the devShell. Needs pkgs with the rust-overlay.
pkgs:
let
  toolchain = (builtins.fromTOML (builtins.readFile ../rust-toolchain.toml)).toolchain;
in
pkgs.rust-bin.stable.${toolchain.channel}.default.override {
  extensions = toolchain.components ++ [ "rust-analyzer" ];
  inherit (toolchain) targets;
}
