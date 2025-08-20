{pkgs ? import <nixpkgs> {}}: let
  manifest = (pkgs.lib.importTOML ./Cargo.toml).package;
in
  pkgs.rustPlatform.buildRustPackage rec {
    pname = manifest.name;
    version = manifest.version;
    cargoLock.lockFile = ./Cargo.lock;
    src = pkgs.lib.cleanSource ./.;

    # Add OpenSSL dependencies
    nativeBuildInputs = with pkgs; [pkg-config];
    buildInputs = with pkgs; [openssl];

    # If you need to set environment variables for OpenSSL
    OPENSSL_DIR = pkgs.openssl.dev;
    OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
    OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include";
  }
