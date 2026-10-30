# rust-toolchain.toml pins the nightly channel; pick the most recent nightly
# that ships every component we ask for. Needs pkgs with the rust-overlay.
pkgs:
pkgs.rust-bin.selectLatestNightlyWith (
  t:
  t.default.override {
    extensions = [
      "rust-src"
      "rust-analyzer"
    ];
  }
)
