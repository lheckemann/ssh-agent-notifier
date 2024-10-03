{ config, pkgs, lib, ... }: let
  cfg = config.services.ssh-agent-notifier;
in {
  options.services.ssh-agent-notifier = {
    enable = lib.mkEnableOption "ssh-agent-notifier";
    backend = lib.mkOption {
      type = lib.types.str;
      description = ''
        The backend agent to proxy and send notifications for.

        The module will attempt to guess an appropriate value here if
        you're using ssh-agent as your SSH agent, but its
        heuristics may be unreliable and you may have to set this
        explicitly.

        Note that Unix socket paths must be prefixed with `unix://`.

        systemd unit specifiers (see `man system.unit`) like `%t` for
        XDG_RUNTIME_DIR may be used.
      '';
      example = "unix://%t/ssh-agent.real";
    };
    listenSocket = lib.mkOption {
      type = lib.types.str;
      description = ''
        The name of the socket to listen on. This is always placed in
        XDG_RUNTIME_DIR.
      '';
      example = "ssh-agent.sock";
      default = "ssh-agent-notifier";
    };
    package = lib.mkOption {
      description = ''
        Package to use for ssh-agent-notifier.

        Required unless using the flake.
      '';
      type = lib.types.package;
      example = lib.literalExample ''
        (pkgs.callPackage ''${ssh-agent-notifier/package.nix} {})
      '';
    };
    setSshAuthSock = lib.mkEnableOption "pointing SSH_AUTH_SOCK to ssh-agent-notifier in session variables";
  };

  config = lib.mkIf cfg.enable {
    systemd.user.services.ssh-agent-notifier = {
      Install.WantedBy = ["default.target"];
      Service.ExecStart = "${cfg.package}/bin/ssh-agent-notifier --host unix://%t/${cfg.listenSocket} --target ${cfg.backend}";
    };

    # Some best-effort defaults.
    services.ssh-agent-notifier.backend = lib.mkIf config.services.ssh-agent.enable (lib.mkDefault "unix:///%t/ssh-agent");

    # Use mkBefore so that we get in before the equivalent settings
    # from services.ssh-agent and services.gpg-agent
    home.sessionVariablesExtra = lib.mkIf cfg.setSshAuthSock (lib.mkBefore ''
      [[ -n "$SSH_AUTH_SOCK" ]] || export SSH_AUTH_SOCK="$XDG_RUNTIME_DIR/${cfg.listenSocket}"
    '');
  };
}
