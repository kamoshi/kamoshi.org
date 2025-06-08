---
title: A declarative server
date: 2025-06-08T14:05:02.692Z
tags: [ nix, nixos ]
desc: >
  You can use Nix to write a fully declarative specification of a server.
---

Once upon a time, running a server meant installing something like Debian and
then manually configuring Apache or Nginx by hand. If you messed up, or if the
server broke, you were often left with no choice but to redo the whole
installation.

Thankfully, we now have Nix and NixOS, which let you declare the configuration
of an entire server in a single file (or more, if you prefer).

This is a game changer if all you want is a simple server for yourself, without
sinking hours into maintenance. You can version the configuration in a git
repository, track every change, and easily roll back to a previous state.
Rollbacks are a built-in feature integrated directly at the bootloader level.

At this point, I honestly can't imagine running any server without NixOS.


## Declarative service module

Here's a small example: a custom Gatus healthcheck service, with full
declarative setup including Nginx reverse proxy. It lives inside a module,
toggled via `kamov.gatus.enable`.

```nix
{ config, pkgs, lib, ... }:
let
  cfg = config.kamov.gatus;
in {
  options.kamov.gatus = {
    enable = lib.mkEnableOption "Enable Gatus";

    port = lib.mkOption {
      type = lib.types.port;
      description = "Port for Gatus";
    };

    bind = lib.mkOption {
      type = lib.types.str;
      description = "Bind address for Gatus";
    };

    domain = lib.mkOption {
      type = lib.types.str;
      description = "Nginx domain";
    };
  };

  config = lib.mkIf cfg.enable {
    services.gatus = {
      enable = true;
      settings.web = {
        port = cfg.port;
        address = cfg.bind;
      };
    };

    services.nginx.virtualHosts."${cfg.domain}" = {
      enableACME = true;
      forceSSL = true;
      locations."/".proxyPass = "http://${cfg.bind}:${toString cfg.port}";
    };
  };
}
```

This probably looks more complex than it really is, especially if you haven't
ever used NixOS before. In the snippet we use `options` key to declare custom
options for our service. Really, we just build an abstraction around `gatus`
service that already exists for NixOS, we add custom options on top of it to
reduce repetition. Then under `config` key we declare the service.

Then, under the `config` key, we apply those options conditionally.

Here's how this module might be used:

```nix
{ config, ... }:
{
  kamov.gatus = {
    enable = true;
    domain = "status.kamoshi.org";
    port = 2138;
    bind = "127.0.0.1";
  };
}
```

Note: config is implicit - you can omit it when assigning values unless you're
introducing new options. The Nix evaluator merges all these fragments into a
concrete system state, which then becomes a bootable operating system.


## Declaratice secrets

So now that we have some service defined, and the entire system config is
declarative, how would we add secrets? For example private keys? We can't keep
these in config, that would defeat the purpose of the key! We could also
manually create the file with secrets and point the config to it, many options
allow you to specify a path to a secret.

However, there's a better way. These issues are why some clever people created
modules that help specifically with this. We can for example use the
[sops-nix](https://github.com/Mic92/sops-nix) module to specify all secrets
declaratively.

For example, let's say we have some secret `.env` variables we would like to
pass to some service. We can import the module from external source and then use
the options from the `sops` module.

```nix
{ config, ... }:
{
  imports = [
    # Sops-nix - secret provisioning
    "${builtins.fetchTarball "https://github.com/Mic92/sops-nix/archive/8d215e1c981be3aa37e47aeabd4e61bb069548fd.tar.gz"}/modules/sops"
  ];

  # Load secrets
  sops = {
    age.keyFile = "/path/to/sops/age/keys.txt";
    defaultSopsFile = /path/to/secrets.yaml;
  };

  # Kotori
  # ======
  sops.secrets.kotori = {
    mode = "0400";
  };

  kamov.kotori = {
    enable = true;
    envPath = config.sops.secrets.kotori.path;
  };
}
```

At build time Nix will create the files with secrets, so that they can be
secuerely consumed by various services at runtime.


## Auth provider service

Once you add more services, you might want to secure them with some
authentication provider. One option is Kanidm, which is unique in that it can be
defined on NixOS declaratively. It doesn’t even have a native GUI, everything is
done via CLI. For NixOS that's perfect.

Here's an example:

```nix
{ config, pkgs, lib, ... }:
let
  cfg = config.kamov.kanidm;
  certs = config.security.acme.certs."${cfg.domain}";
in {
  services.kanidm = {
    package = pkgs.kanidmWithSecretProvisioning;

    enableServer = true;
    serverSettings = {
      domain = cfg.domain;
      origin = "https://${cfg.domain}";
      tls_chain = "${certs.directory}/fullchain.pem";
      tls_key   = "${certs.directory}/key.pem";
      trust_x_forward_for = true;
      bindaddress = "${cfg.bind}:${toString cfg.port}";

      online_backup = {
        path = "/var/backup/kanidm/";
        schedule = "0 20 * * *"; # UTC time
        versions = 2;
      };
    };

    enableClient = true;
    clientSettings = {
      uri = "https://${cfg.domain}";
    };

    provision = {
      enable = true;
      autoRemove = true;

      persons = {
        someperson = {
          displayName = "someperson";
          mailAddresses = [ "someperson@example.org" ];
          groups = [
            "miniflux.access"
          ];
        };
      };

      groups = {
        # Miniflux
        "miniflux.access" = {};
      };

      systems.oauth2 = {
        miniflux = {
          displayName = "Miniflux";
          originUrl = "https://rss.example.org/oauth2/oidc/callback";
          originLanding = "https://rss.example.org/oauth2/oidc/redirect";
          basicSecretFile = config.sops.secrets."kanidm/miniflux".path;
          preferShortUsername = true;
          scopeMaps = {
            "miniflux.access" = [ "openid" "profile" "email" ];
          };
        };
      };
    };
  };
}
```

Notice the `pkgs.kanidmWithSecretProvisioning` bit. This is Kanidm patched to work
with NixOS provisioning, meaning NixOS will automatically call the CLI commands
to set up the internal state of the auth provider.

For me, this is a killer feature: I don't have to manage anything via GUI, and I
don't even have to touch the CLI. Everything is done automatically.


## Auth client service

Now, how do you add a service compatible with Kanidm? Let's look at a simple
Miniflux config. Miniflux is an RSS reader written in Go, so it takes minimal
resources. It's also simple to use and feature-rich.

This is the full module for Miniflux:

```nix
{ config, pkgs, lib, ... }:
let
  cfg = config.kamov.miniflux;
in {
  options.kamov.miniflux = {
    enable = lib.mkEnableOption "Enable Miniflux";

    port = lib.mkOption {
      type = lib.types.port;
      description = "Port for Miniflux.";
    };

    bind = lib.mkOption {
      type = lib.types.str;
      description = "Bind address for Miniflux.";
    };

    domain = lib.mkOption {
      type = lib.types.str;
      description = "Nginx domain";
    };

    oauthSecretPath = lib.mkOption {
      type = lib.types.path;
      description = "OAuth secret path.";
    };
  };

  config = lib.mkIf cfg.enable {
    # System security
    # ----------
    users.groups.miniflux = {};
    users.users.miniflux = {
      isSystemUser = true;
      createHome = false;
      group = "miniflux";
    };

    systemd.services.miniflux = {
      serviceConfig = {
        User  = "miniflux";
        Group = "miniflux";
      };
    };

    # Miniflux
    # ----------
    services.miniflux = {
      enable = true;
      config = {
        LISTEN_ADDR = "${cfg.bind}:${toString cfg.port}";
        BASE_URL = "https://${cfg.domain}/";
        RUN_MIGRATIONS = 1;
        CREATE_ADMIN = 0;
        CLEANUP_ARCHIVE_UNREAD_DAYS = "-1";
        CLEANUP_ARCHIVE_READ_DAYS = "-1";
        POLLING_FREQUENCY = 480;

        # OIDC
        OAUTH2_PROVIDER = "oidc";
        OAUTH2_CLIENT_ID = "miniflux";
        OAUTH2_CLIENT_SECRET_FILE = cfg.oauthSecretPath;
        OAUTH2_REDIRECT_URL = "https://${cfg.domain}/oauth2/oidc/callback";
        OAUTH2_OIDC_DISCOVERY_ENDPOINT = "https://auth.example.org/oauth2/openid/miniflux";

        # Disable local auth
        DISABLE_LOCAL_AUTH = "true";
        OAUTH2_USER_CREATION = "true";
      };
    };

    # Network
    # ----------
    networking.firewall = {
      enable = true;
      interfaces.wg0.allowedTCPPorts = [ cfg.port ];
    };

    services.nginx.virtualHosts."${cfg.domain}" = {
      enableACME = true;
      forceSSL = true;
      locations."/" = {
        proxyPass = "http://${cfg.bind}:${toString cfg.port}";
      };
    };

    # Database
    # ----------
    services.postgresqlBackup.databases = [
      "miniflux"
    ];
  };
}
```

As you can see, Nix and NixOS let us do a lot, cleanly. It's a bird's-eye view
of the entire system and how each fragment is interconnected. We don't have to
do it manually - Nix does it for us. We just define the end state.

The module can still be refactored e.g., that one URL could be extracted into an
option, but you might just as well avoid using options entirely. It's up to you.
