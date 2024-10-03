{ rustPlatform, version ? "unstable", nix-gitignore, libnotify, glib }:
rustPlatform.buildRustPackage {
  pname = "ssh-agent-notifier";
  inherit version;
  src = builtins.path {
    name = "source";
    path = ./.;
    filter = nix-gitignore.gitignoreFilterPure (_: _: true) [
      ./.gitignore
      "*.nix"
      ".gitignore"
      "README.md"
      ".envrc"
    ] ./.;
  };
  cargoLock = {
    lockFile = ./Cargo.lock;
    outputHashes."ssh-agent-lib-0.4.0" = "sha256-puYvOP9LiolcM460FJTBezBwwOkgW62I2UN8fe+Qb5k=";
  };
  buildInputs = [
    libnotify
    glib
  ];
}
