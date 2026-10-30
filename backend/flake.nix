{
  description = "greenhouse_backend: service binaries + devShell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
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

      devShells.${system}.default = pkgs.callPackage ./nix/shell.nix {
        toolchain = import ./nix/toolchain.nix pkgs;
      };
    };
}
