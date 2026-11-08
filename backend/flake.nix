{
  description = "greenhouse_backend: service binaries + devShell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    # Pinned to the last revision whose default ESP-IDF is v5.3.x, matching the
    # v5.3 line greenhouse_esp32 targets (.cargo/config.toml). Not following our
    # nixpkgs: its Python package set is tuned to the nixpkgs it was locked with.
    nixpkgs-esp-dev.url = "github:mirrexagon/nixpkgs-esp-dev/4e791fca5ae53623bba45157dae5d43e9cb0b335";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      nixpkgs-esp-dev,
      ...
    }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ (import rust-overlay) ];
      };
    in
    {
      # Deployed by pinning a git tag of this flake (`?dir=backend`).
      packages.${system}.default = pkgs.callPackage ./nix/package.nix {
        version = self.shortRev or self.dirtyShortRev or "dirty";
      };

      devShells.${system} =
        let
          toolchain = import ./nix/toolchain.nix pkgs;
        in
        {
          default = pkgs.callPackage ./nix/shell.nix { inherit toolchain; };
          # ESP32 firmware (greenhouse_esp32); see nix/esp32-shell.nix.
          esp32 = pkgs.callPackage ./nix/esp32-shell.nix {
            esp-idf = nixpkgs-esp-dev.packages.${system}.esp-idf-esp32c3;
          };
        };
    };
}
