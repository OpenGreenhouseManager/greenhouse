{
  mkShell,
  nodejs,
  playwright-driver,
}:

mkShell {
  packages = [ nodejs ];
  # @angular/build 22 defaults to sass-embedded, a prebuilt dynamically
  # linked Dart binary that NixOS cannot run. Use the pure-JS compiler.
  NG_BUILD_SASS_EMBEDDED = "0";
  # Vitest browser tests: Playwright's downloaded browsers do not run on
  # NixOS, so use the nixpkgs build. Keep playwright-driver's version in
  # step with the npm `playwright` that @vitest/browser-playwright pulls in.
  PLAYWRIGHT_BROWSERS_PATH = "${playwright-driver.browsers}";
  PLAYWRIGHT_SKIP_VALIDATE_HOST_REQUIREMENTS = "true";
}
