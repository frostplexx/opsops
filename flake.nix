{
  description = "Foo Bar";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs = {
    self,
    nixpkgs,
    fenix,...
  }: let
    supportedSystems = ["x86_64-linux" "aarch64-darwin"];
    forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
    pkgsFor = nixpkgs.legacyPackages;
  in {
packages = forAllSystems (system: {
  default = pkgsFor.${system}.callPackage ./. {inherit fenix system;};
});
devShells = forAllSystems (system: {
  default = pkgsFor.${system}.callPackage ./shell.nix {inherit fenix system;};
});
  };
}
