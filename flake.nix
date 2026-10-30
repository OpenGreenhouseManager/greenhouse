{
  description = "greenhouse: Rust backend + Angular web monorepo";

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
      version = self.shortRev or self.dirtyShortRev or "dirty";
      nodejs = pkgs.nodejs_24;

      backendShell = pkgs.callPackage ./backend/nix/shell.nix {
        toolchain = import ./backend/nix/toolchain.nix pkgs;
      };
      webShell = pkgs.callPackage ./web/nix/shell.nix { inherit nodejs; };
    in
    {
      # backend/flake.nix and web/flake.nix stay usable on their own
      # (`github:…/greenhouse?dir=backend`); both build from the same nix/ files.
      packages.${system} = {
        backend = pkgs.callPackage ./backend/nix/package.nix { inherit version; };
        web = pkgs.callPackage ./web/nix/package.nix { inherit nodejs version; };
      };

      devShells.${system} = {
        backend = backendShell;
        web = webShell;
        default = pkgs.mkShell {
          inputsFrom = [
            backendShell
            webShell
          ];
          inherit (backendShell) OPENSSL_NO_VENDOR;
          inherit (webShell)
            NG_BUILD_SASS_EMBEDDED
            PLAYWRIGHT_BROWSERS_PATH
            PLAYWRIGHT_SKIP_VALIDATE_HOST_REQUIREMENTS
            ;
        };
      };
    };
}
