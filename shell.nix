{
  pkgs ? import <nixpkgs> {},
  stdenv,
  fenix,
  system,
}: let
  overrides = builtins.fromTOML (builtins.readFile ./rust-toolchain.toml);
in
  pkgs.mkShell {
    # get dependencies from the main package
    inputsfrom = [(pkgs.callPackage ./default.nix {inherit fenix system;})];
    # additional tooling
    buildinputs = with pkgs; [
      rustup
      rustPlatform.bindgenHook
      openssl
      pkg-config
      just
    ];

    RUSTC_VERSION = overrides.toolchain.channel;
    # https://github.com/rust-lang/rust-bindgen#environment-variables
    shellHook = ''
      export PATH="''${CARGO_HOME:-~/.cargo}/bin":"$PATH"
      export PATH="''${RUSTUP_HOME:-~/.rustup}/toolchains/$RUSTC_VERSION-${stdenv.hostPlatform.rust.rustcTarget}/bin":"$PATH"
    '';
  }
