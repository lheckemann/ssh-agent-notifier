{
  inputs = {
    nixpkgs.url = "git+https://cl.forkos.org/nixpkgs";
  };
  outputs = { self, nixpkgs }: let
    inherit (nixpkgs) lib;
    forAllSystems = lib.genAttrs [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" ];
  in {
    packages = forAllSystems (system: {
      default = nixpkgs.legacyPackages.${system}.callPackage ./package.nix {};
    });
    homeManagerModules.default = { pkgs, lib, ... }: {
      imports = [./hm-module.nix];
      services.ssh-agent-notifier.package = self.packages.${pkgs.system}.default;
    };
  };
}
