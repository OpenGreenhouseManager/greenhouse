# devShell for greenhouse_esp32 (RISC-V ESP32-C3/C6/C2, target riscv32imc-esp-espidf).
#
# esp-idf-sys normally downloads ESP-IDF and Espressif's prebuilt gcc into
# .embuild/; those binaries are FHS-linked and do not run on NixOS. Instead
# ESP-IDF + its toolchain come from nixpkgs-esp-dev and esp-idf-sys is told to
# use the activated environment (ESP_IDF_TOOLS_INSTALL_DIR=fromenv). With
# fromenv, IDF_PATH decides the ESP-IDF version — ESP_IDF_VERSION in
# greenhouse_esp32/.cargo/config.toml is ignored.
#
# Kept separate from the backend shell: the closure is large and the esp-idf
# setup hook exports IDF/Python/git env that normal builds must not see.
{
  mkShell,
  rust-bin,
  esp-idf,
  ldproxy,
  espflash,
  llvmPackages,
}:

let
  # Pinned: rust-lang/rust b728431 (2026-10-16) bumped std to libc 0.2.191, whose
  # espidf `stat` uses `st_atim` timespecs while std/os/espidf still reads
  # `st_atime`, so build-std fails on later nightlies. Bump once upstream is
  # consistent again. Keep in sync with greenhouse_esp32/rust-toolchain.toml.
  toolchain = rust-bin.nightly."2026-10-15".default.override {
    extensions = [
      "rust-src"
      "rust-analyzer"
    ];
  };

  # esp-idf-sys (fromenv) runs `git rev-parse` / `git describe` in IDF_PATH, so it
  # must be a git repo tagged with its version. nixpkgs-esp-dev does this itself
  # since 7579f37 (Aug 2025, IDF 5.5); backported here for the pinned 5.3 build.
  esp-idf-git = esp-idf.overrideAttrs (old: {
    installPhase = old.installPhase + ''
      cd $out
      export GIT_AUTHOR_DATE="1970-01-01T00:00:00Z" GIT_COMMITTER_DATE="1970-01-01T00:00:00Z"
      git init -q .
      git -c user.name=nixbld -c user.email=nixbld@localhost commit -q --allow-empty -m "esp-idf ${old.version}"
      git tag "${old.version}"
    '';
  });
in
mkShell {
  packages = [
    toolchain # nightly + rust-src, required for build-std on the tier-3 target
    esp-idf-git # sets IDF_PATH, IDF_PYTHON_ENV_PATH and puts riscv32-esp-elf-* on PATH
    ldproxy # linker configured in .cargo/config.toml
    espflash # flash + monitor: `espflash flash --monitor <elf>`
  ];

  ESP_IDF_TOOLS_INSTALL_DIR = "fromenv";
  # bindgen for esp-idf-sys; stock libclang handles RISC-V (esp-clang is only
  # needed for Xtensa chips).
  LIBCLANG_PATH = "${llvmPackages.libclang.lib}/lib";

  # The store copy is owned by root; let git operate on it without touching the
  # user's global config.
  GIT_CONFIG_COUNT = "1";
  GIT_CONFIG_KEY_0 = "safe.directory";
  GIT_CONFIG_VALUE_0 = "${esp-idf-git}";
}
