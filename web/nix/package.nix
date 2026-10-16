{
  lib,
  stdenv,
  nodejs,
  importNpmLock,
  # Overridden by the flake with something more informative (git rev).
  version ? "0-unstable",
}:

let
  fs = lib.fileset;

  # Only the files the production build actually reads, so editing the README
  # or CI workflows does not force the deployment host to rebuild the app.
  src = fs.toSource {
    root = ../.;
    fileset = fs.unions [
      ../src
      ../angular.json
      ../package.json
      ../package-lock.json
      ../tsconfig.json
      ../tsconfig.app.json
      ../.postcssrc.json
    ];
  };
in

stdenv.mkDerivation {
  pname = "greenhouse-web";
  inherit version src;

  nativeBuildInputs = [
    nodejs
    # Links a node_modules built straight from package-lock.json into the build
    # directory. Every tarball is fetched by the integrity hash already recorded
    # in the lockfile, so there is no vendor hash to regenerate when
    # dependencies change -- unlike buildNpmPackage's npmDepsHash.
    importNpmLock.hooks.linkNodeModulesHook
  ];

  npmDeps = importNpmLock.buildNodeModules {
    npmRoot = src;
    inherit nodejs;
  };

  env = {
    NG_CLI_ANALYTICS = "false";
    CI = "true";
    HUSKY = "0";
  };

  buildPhase = ''
    runHook preBuild

    # The Angular CLI wants a writable HOME for its cache.
    export HOME=$(mktemp -d)

    # defaultConfiguration is "production": optimization, budgets, hashing.
    ./node_modules/.bin/ng build

    runHook postBuild
  '';

  installPhase = ''
    runHook preInstall

    mkdir -p "$out"
    cp -R dist/greenhouse_web/browser/. "$out"/

    runHook postInstall
  '';

  # The single de-DE locale is built flat (`"subPath": ""` in angular.json), so
  # the app is served from `/`. Fail loudly if Angular ever reverts to a
  # per-locale directory: serving under /de-DE/ breaks the auth cookie handling.
  doInstallCheck = true;
  installCheckPhase = ''
    runHook preInstallCheck
    grep -q '<base href="/"' "$out/index.html"
    runHook postInstallCheck
  '';

  meta = {
    description = "Web frontend for the Open Greenhouse Manager";
    homepage = "https://github.com/OpenGreenhouseManager/greenhouse_web";
    platforms = lib.platforms.linux;
  };
}
