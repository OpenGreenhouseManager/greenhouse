{
  mkShell,
  toolchain,
  pkg-config,
  openssl,
  postgresql,
  just,
  typos,
  cargo-sort,
  cargo-nextest,
  bacon,
  diesel-cli,
}:

mkShell {
  nativeBuildInputs = [ pkg-config ];

  buildInputs = [
    openssl # openssl-sys (reqwest/native-tls, sentry)
    postgresql # pq-sys: libpq + pg_config
  ];

  packages = [
    toolchain
    just # task runner (justfile)
    typos # CI: typo check
    cargo-sort # CI: cargo sort --workspace --grouped
    cargo-nextest
    bacon
    (diesel-cli.override {
      sqliteSupport = false;
      mysqlSupport = false;
    })
  ];

  # Each service owns its own database (auth/data/device), so DATABASE_URL
  # is per-service — e.g. postgres://admin:password@localhost:5432/auth —
  # and is left to the caller rather than set here.
  OPENSSL_NO_VENDOR = "1"; # never let openssl-sys vendor its own copy
}
