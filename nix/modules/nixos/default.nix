{
  perSystem,
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.services.automatic-redshift;
in
{
  options = {
    services.automatic-redshift = {
      enable = lib.mkOption {
        type = lib.types.bool;
        default = false;
      };
      package = lib.mkPackageOption perSystem.self "automatic-redshift" { };
    };
  };

  config = lib.mkIf cfg.enable {
    services.geoclue2 = {
      enable = true;
      appConfig.automatic-redshift = {
        isAllowed = true;
        isSystem = true;
        users = [ (toString config.ids.uids.automatic-redshift) ];
      };
    };

    systemd.services = {

      automatic-redshift = {
        description = "Automatically adjust screen color temperature based on location and time of day";
        requires = [ "automatic-redshift-geoclue-agent.service" ];
        after = [ "automatic-redshift-geoclue-agent.service" ];
        serviceConfig = {
          Type = "exec";
          User = "automatic-redshift";
          ExecStart = "${cfg.package}/bin/automatic-redshift";
        };
        wantedBy = [ "default.target" ];
      };

      automatic-redshift-geoclue-agent = {
        description = "Geoclue agent for automatic-redshift";
        requires = [ "geoclue.service" ];
        after = [ "geoclue.service" ];
        serviceConfig = {
          Type = "exec";
          User = "automatic-redshift";
          ExecStart = "${pkgs.geoclue2-with-demo-agent}/libexec/geoclue-2.0/demos/agent";
          Restart = "on-failure";
          PrivateTmp = true;
        };
        wantedBy = [ "default.target" ];
      };
    };

    users = {
      users.automatic-redshift = {
        description = "automatic-redshift";
        uid = config.ids.uids.automatic-redshift;
        group = "automatic-redshift";
      };
      groups.automatic-redshift = {
        gid = config.ids.gids.automatic-redshift;
      };
    };
  };
}
