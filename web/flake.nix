{
  description = "greenhouse_web: static production build + devShell";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
    in
    {
      # The built SPA (index.html + assets), served as static files. Deployed by
      # pinning a git tag of this flake (`?dir=web`).
      packages.${system}.default = pkgs.callPackage ./nix/package.nix {
        nodejs = pkgs.nodejs_24;
        version = self.shortRev or self.dirtyShortRev or "dirty";
      };

      devShells.${system}.default = pkgs.callPackage ./nix/shell.nix {
        nodejs = pkgs.nodejs_24;
      };
    };
}
