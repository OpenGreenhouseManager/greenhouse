# The six service binaries (the workspace's default-members): web_api,
# scripting_api, auth_service, data_storage_service, device_service,
# scripting_service.
{
  rustPlatform,
  pkg-config,
  openssl,
  postgresql,
  # Overridden by the flake with something more informative (git rev).
  version ? "0-unstable",
}:

rustPlatform.buildRustPackage {
  pname = "greenhouse-backend";
  inherit version;
  src = ../.;
  cargoLock.lockFile = ../Cargo.lock;

  nativeBuildInputs = [ pkg-config ];
  buildInputs = [
    openssl
    postgresql # libpq for diesel's migrations
  ];
  env.OPENSSL_NO_VENDOR = "1";

  # integration-tests need Docker (testcontainers).
  doCheck = false;
}
