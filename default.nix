{pkgs ? import <nixpkgs> {}, fenix, system}: let
  manifest = (pkgs.lib.importTOML ./Cargo.toml).package;
  overrides = (builtins.fromTOML (builtins.readFile ./rust-toolchain.toml));
in
  (pkgs.makeRustPlatform {
          inherit (fenix.packages.${system}.minimal) cargo rustc;
        }).buildRustPackage rec {
    pname = manifest.name;
    version = manifest.version;
    cargoLock.lockFile = ./Cargo.lock;
    src = pkgs.lib.cleanSource ./.;

    # Add OpenSSL dependencies
    nativeBuildInputs = with pkgs; [pkg-config];
    buildInputs = with pkgs; [openssl];

    # If you need to set environment variables for OpenSSL
    RUSTC_VERSION = overrides.toolchain.channel;
    OPENSSL_DIR = pkgs.openssl.dev;
    OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
    OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include";
  }
