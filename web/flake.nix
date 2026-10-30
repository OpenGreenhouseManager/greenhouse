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
      # pinning a git tag of this flake.
      packages.${system}.default = pkgs.callPackage ./nix/package.nix {
        nodejs = pkgs.nodejs_24;
        version = self.shortRev or self.dirtyShortRev or "dirty";
      };

      devShells.${system}.default = pkgs.mkShell {
        packages = [ pkgs.nodejs_24 ];
        # @angular/build 22 defaults to sass-embedded, a prebuilt dynamically
        # linked Dart binary that NixOS cannot run. Use the pure-JS compiler.
        NG_BUILD_SASS_EMBEDDED = "0";
        # Vitest browser tests: Playwright's downloaded browsers do not run on
        # NixOS, so use the nixpkgs build. Keep playwright-driver's version in
        # step with the npm `playwright` that @vitest/browser-playwright pulls in.
        PLAYWRIGHT_BROWSERS_PATH = "${pkgs.playwright-driver.browsers}";
        PLAYWRIGHT_SKIP_VALIDATE_HOST_REQUIREMENTS = "true";
      };
    };
}
