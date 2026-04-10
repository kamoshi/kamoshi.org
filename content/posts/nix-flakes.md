---
title: Are Nix flakes worth it?
date: 2026-04-10T00:00:00Z
tags: [nix, nixos]
desc: >
  Someone recently asked me if Nix flakes are worth it even if you don't care
  about full reproducibility. Here are my thoughts.
---

Someone asked me: if you don't care about pinning versions and just want the
newest stuff, is there any point to enabling Nix Flakes?

In my opinion, yes. Not because of reproducibility, but because flakes give Nix
a clear, consistent structure that makes everything easier to reason about.


## One config to rule them all

The biggest thing for me is that I have one config that works for all my
machines: NixOS, macOS (via `nix-darwin`), Arch Linux (via `home-manager`).
Flakes give you a clean way to structure multiple outputs from a single entry
point:

```nix
in
{
  nixosConfigurations = lib.pipe devices [
    (lib.filterAttrs (matches Type.NixOS))
    (lib.mapAttrs (key: device: device // { inherit key; }))
    (lib.mapAttrs (util.mkNixOS meshFor vpnFor))
  ];
 
  darwinConfigurations = lib.pipe devices [
    (lib.filterAttrs (matches Type.Darwin))
    (lib.mapAttrs (key: device: device // { inherit key; }))
    (lib.mapAttrs (util.mkDarwin meshFor vpnFor))
  ];
 
  homeConfigurations = lib.pipe devices [
    (lib.filterAttrs (matches Type.Home))
    (lib.mapAttrs (key: device: device // { inherit key; }))
    (lib.mapAttrs (util.mkHome meshFor vpnFor))
  ];
};
```
 
This `pipe` is something I wrote, and it just creates all configs from single
source of truth.

You can define your devices, pick the right entry, and one flake handles all of
them. It just has more logical sense than the old channels-based approach.

And even if you track `nixos-unstable` and always want the latest `flake.lock`
still saves you. When an update breaks something, you can revert the lockfile
via `git` and rebuild. You always know exactly what commit your working system
was built from.


## One flake, multiple outputs

I run a server API and a private Discord bot written in Rust. This is where the
flakes model really clicks.

A flake has two sides: inputs and outputs. Inputs are other flakes you pull in,
outputs are what you expose to the world. The Rust repo's `flake.nix` exposes
two outputs: the compiled binary as a package, and a NixOS module that wires up
a systemd service.

My server flake lists the Rust repo as an input:

```nix
fukurou = {
  url = "github:/kamoshi/fukurou";
  inputs.nixpkgs.follows = "nixpkgs";
};
```

This gives the server flake access to both outputs. The NixOS module looks like
this:

```nix
nixosModules.default = { config, lib, pkgs, ... }:
  let
    cfg = config.services.fukurou;
  in
  {
    options.services.fukurou = {
      enable = lib.mkEnableOption "Fukurou Service";
      port = lib.mkOption { type = lib.types.port; default = 3000; ... };
      portInternal = lib.mkOption { type = lib.types.port; default = 3001; ... };
      envFile = lib.mkOption { type = lib.types.path; default = "/var/lib/fukurou/secrets.env"; ... };
    };
 
    config = lib.mkIf cfg.enable {
      systemd.services.fukurou = {
        description = "Fukurou Daemon";
        wantedBy = [ "multi-user.target" ];
        after = [ "network-online.target" ];
        wants = [ "network-online.target" ];
        serviceConfig = {
          ExecStart = "${self.packages.x86_64-linux.fukurou}/bin/fukurou";
          DynamicUser = true;
          StateDirectory = "fukurou";
          MemoryMax = "64M";
          EnvironmentFile = cfg.envFile;
          Restart = "always";
          RestartSec = "10s";
          # hardening
          CapabilityBoundingSet = "";
          NoNewPrivileges = true;
          ProtectSystem = "strict";
          PrivateTmp = true;
          PrivateDevices = true;
        };
      };
    };
  };
```
 
I import that module in my server flake, which makes `services.fukurou`
available just like any other NixOS service:
 
```nix
services.fukurou = {
  enable = true;
  port = portExternal;
  portInternal = portInternal;
  envFile = config.sops.secrets.kotori.path;
};
```
 
And that sets up the Rust daemon on my NixOS server. It allows me to deploy it
extremely easily.

The same `flake.nix` also defines `devShells.default`, which means:
 
```
[I] kamov@aya ~/r/server (main)> nix develop
(nix:nix-shell-env) kamov@aya:~/repo/server$ cargo run
   Compiling proc-macro2 v1.0.106
   ...
```

This works even if you don't have `rustc` or `rustup` installed on your system.
`nix develop` drops you into a shell with everything needed to build that
specific project, in the right versions, isolated from your global environment.
When you do `nix build .` it compiles everything and puts the result in a
`./result` symlink.
 
If you want it to be fully automatic, there's `direnv`. With it you can drop a
`.envrc` with `use flake` in your project root, and every time you `cd` into the
folder, the dev environment loads. `cd ..`, it unloads.


## Deploying from Arch Linux to a NixOS server

My main PC runs Arch Linux with Nix and `home-manager` on top, not NixOS itself.
So when I want to deploy to my server, I use:

```bash
nix run nixpkgs#nixos-rebuild -- switch --flake .#megumu --target-host megumu --sudo --ask-sudo-password
```

`nixos-rebuild` is a NixOS tool, so it's not installed on Arch. `nix run` just
fetches and runs it from nixpkgs on the fly. It builds everything locally, sends
the compiled closures over SSH, and switches the server, so the server itself
doesn't have to compile anything.
 
I prefer to use that over tools like `deploy-rs` or `colmena`, because I only have
one server. My approach is that when a plain command does the job fine, I avoid
adding more complexity just because it exists.
 
 
## Why I use Nix
 
I don't like putting effort into things and then losing them.
 
Every time I add something to PATH, write a shell function, or tweak some
config, I can save it into git. It has a commit message explaining why I did it,
or I can add comment directly in the config.
 
It's not that I want version control for the sake of version control, it's just
that when I put effort setting something up nicely, I want that effort to be
permanently saved for later.
 
A useful trick: when you're trying to figure out how to configure something in
Nix, search GitHub for it directly. Not in the docs, on GitHub. Something like:

```
"services.nginx = {" language:nix
```
 
You'll see how real people have actually set it up, what options they've used,
what they've combined it with.

I've never truly sat down and learned Nix. I still find parts of it weird and
esoteric. I just picked it up piece by piece as I needed things: first to stand
up a server, then to manage my home config, then to wire up my own apps as
modules.

If you're curious about it, that's probably how I'd recommend approaching it.
Pick one concrete thing you want to do, a server config or just your dotfiles,
and learn what you need for that. The ecosystem is huge, but you don't have to
absorb all of it to get real value out of it.

And there's genuinely nothing else on the market with these properties. Guix
exists, but I'm not convinced it's better.


## AI agents + Nix is a surprisingly good combo

*(A tangent. Skip if you're not into that.)*

I've been using an AI coding agent to help configure stuff on my servers,
editing the Nix files locally and then deploying over SSH. It works really
well. The reason, I think, is that Nix is purely declarative. The agent is just
editing text files. Nothing in the actual system changes until you explicitly
rebuild.

It can even run `nix instantiate` to check if the generated config is
syntactically valid without doing a full build, for a fast feedback loop.


## Things worth looking into

If you decide to go down this path, these are the tools I use on top of vanilla Nix:

- **nix-darwin**: brings the NixOS-style declarative config to macOS. I can
  define dock settings, system daemons, installed apps.
- **home-manager**: manages my user environment: shell config, dotfiles, PATH
  modifications, everything in `~`.
- **sops-nix**: handles secrets. Nix's store isn't a safe place for passwords,
  so `sops-nix` lets me store encrypted secrets in my public git repo, decrypted
  only on the target machine using age keys.
- **nix-command + flakes**: both experimental, but basically standard at this
  point.
