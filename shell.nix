{pkgs ? import <nixpkgs> {}}:
pkgs.mkShell {
  # get dependencies from the main package
  inputsfrom = [(pkgs.callpackage ./default.nix {})];
  # additional tooling
  buildinputs = with pkgs; [
    rust-analyzer # lsp server
    rustfmt # formatter
    clippy # linter
    openssl
    pkg-config
    cargo-deny
    cargo-edit
    cargo-watch
    just
  ];
}
