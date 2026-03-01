---
title: RSS Summarizer
date: 2026-03-01T18:46:29.548Z
desc: >
  I've been using RSS to fetch and read things that interest me. I use Miniflux as
  my self-hosted feed reader. It has an API that lets you fetch unread articles
  programmatically. So I built something that takes those unread articles, sends
  them to Gemini, and gets back a nicely formatted editorial briefing.
---

I've been using RSS to fetch and read things that interest me. I use
[Miniflux](https://miniflux.app) as my self-hosted feed reader. It has an API
that lets you fetch unread articles programmatically. So I built something that
takes those unread articles, sends them to Gemini, and gets back a nicely
formatted editorial briefing.

Here's how that actually went.


## Getting the shell in place

It started with figuring out how to properly package a Rust server for NixOS. I
came across [naersk](https://github.com/nix-community/naersk). I wasn't sure if
the name was a pun on Nix + Maersk, but it builds Rust projects as a Nix flake
easily.

First step was successfully wrapping the Rust server for NixOS. The actual
server at this point had everything commented out. It literally just had one
route, `/hello`, returning `"world"`:

```rust
use axum::{routing::get, Router};
use std::env;
use tokio::net::TcpListener;

async fn hello() -> &'static str {
    "world"
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/hello", get(hello));

    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to port");

    println!("Server running on {}", addr);
    axum::serve(listener, app).await.expect("Server failed");
}
```

But the Nix side was already complete. The flake exposed a NixOS module with the
full systemd service definition: memory cap, hardening options, secrets via
environment file, restart policy, etc.

```nix
{
  description = "rss-summarizer NixOS service";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    naersk.url = "github:nix-community/naersk";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, naersk, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = (import nixpkgs) { inherit system; };
        naersk-lib = pkgs.callPackage naersk { };
      in {
        packages.rss-summarizer = naersk-lib.buildPackage { src = ./.; };
        packages.default = self.packages.${system}.rss-summarizer;
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [ cargo rustc rustfmt rustPackages.clippy ];
        };
      }
    ) // {
      nixosModules.default = { config, lib, pkgs, ... }:
        let cfg = config.services.rss-summarizer; in {
          options.services.rss-summarizer = {
            enable = lib.mkEnableOption "RSS Summarizer Service";
            port = lib.mkOption {
              type = lib.types.port;
              default = 3000;
              description = "Port to listen on.";
            };
            minifluxUrl = lib.mkOption {
              type = lib.types.str;
              default = "http://localhost:8080";
              description = "Miniflux API URL.";
            };
          };

          config = lib.mkIf cfg.enable {
            systemd.services.rss-summarizer = {
              description = "RSS Summarizer Daemon";
              wantedBy = [ "multi-user.target" ];
              after = [ "network-online.target" ];
              wants = [ "network-online.target" ];
              serviceConfig = {
                ExecStart = "${self.packages.x86_64-linux.rss-summarizer}/bin/rss-summarizer";
                DynamicUser = true;
                MemoryMax = "64M";
                EnvironmentFile = "/var/lib/rss-summarizer/secrets.env";
                Environment = [
                  "PORT=${toString cfg.port}"
                  "MINIFLUX_URL=${cfg.minifluxUrl}"
                ];
                Restart = "always";
                RestartSec = "10s";
                # Hardening
                CapabilityBoundingSet = "";
                NoNewPrivileges = true;
                ProtectSystem = "strict";
                ProtectHome = true;
                PrivateTmp = true;
                PrivateDevices = true;
              };
            };
          };
        };
    };
}
```

The nice thing about exposing a NixOS module from the flake is that adding this
service to the server config becomes trivial. In the server's `inputs`:

```nix
server = {
  url = "github:/kamoshi/server";
  inputs.nixpkgs.follows = "nixpkgs";
};
```

Then import `server.nixosModules.default` and it's just:

```nix
services.rss-summarizer = {
  enable = true;
  port = 4321;
};
```

And that generates a proper systemd service. Running and confirmed:

```
● rss-summarizer.service - RSS Summarizer Daemon
     Active: active (running) since Fri 2026-02-27 22:09:22 CET; 4min 49s ago
     Memory: 1.8M (max: 64M)
```

```
$ curl http://localhost:4321/hello
world
```

Good enough for now. I also decided at this point to port my existing bot
(Kotori, which was running in podman) into this same service instead. Run it as
a proper systemd unit rather than a container.

As for the actual RSS+Gemini functionality, that was still just a plan: fetch
unread articles from the last 2 days, send them to Gemini Flash, get back a
daily briefing. I hadn't used the Gemini API before, so this was going to be a
chance to learn how those API wrappers people build actually work. The official
SDKs are only Python, JavaScript, and Go. For Rust I found
[`gemini-rust`](https://crates.io/crates/gemini-rust) on crates.io, which is
unofficial, but it looked like it worked.


## Deploying to the real server

The build runs entirely on the local machine (my gaming PC, `momiji`) and the
result gets transmitted over SSH to the server (`megumu`). The server never has
to compile anything. It just receives the closure and switches to it.

```bash
nix run nixpkgs#nixos-rebuild -- switch --flake .#megumu \
  --target-host megumu --sudo --ask-sudo-password
```

Output:

```
building the system configuration...
copying 0 paths...
updating GRUB 2 menu...
activating the configuration...
reloading user units for kamov...
restarting sysinit-reactivation.target
the following new units were started: rss-summarizer.service
Done. The new configuration is /nix/store/brbix89q7zc6x6disi3gis6ixikg6lr2-nixos-system-megumu-25.11.20260218.6d41bc2
```

It also rebuilt the GRUB menu, because NixOS keeps all previous configurations
as bootable entries, so you can always roll back. Each generation is basically a
pointer into the Nix store; most of the content is shared between generations
rather than duplicated, similar to how persistent data structures work in
functional languages. `/nix/store` is a bit like that.

SSHed into the server to verify:

```
kamov@megumu ~> systemctl status rss-summarizer.service
● rss-summarizer.service - RSS Summarizer Daemon
     Active: active (running) since Fri 2026-02-27 23:41:09 CET; 1min 25s ago
     Memory: 1.6M (max: 64M)

Feb 27 23:41:09 megumu rss-summarizer[804684]: Server running on 0.0.0.0:4321
Feb 27 23:41:09 megumu rss-summarizer[804684]: Kotori is connected!
```

Kotori running podman-less. ✓


## Wiring up Miniflux and Gemini

Next I wired up the actual functionality. The Miniflux side: fetch up to 20
unread entries published in the last 2 days, ordered by publish date descending,
then concatenate title, date, URL, and content for each one with a separator:

```rust
async fn test_miniflux() -> String {
    let url = Url::parse("https://rss.kamoshi.org").unwrap();
    let api_token = env::var("TOKEN_MINIFLUX").unwrap_or_default();
    if api_token.is_empty() {
        return "MINIFLUX_API_TOKEN is not set".to_string();
    }

    let api = MinifluxApi::new_from_token(&url, api_token);
    let http_client = Client::new();
    let two_days_ago = (Utc::now() - Duration::days(2)).timestamp();

    let entries = api
        .get_entries(
            Some(EntryStatus::Unread),
            None,
            Some(20),
            Some(OrderBy::PublishedAt),
            Some(OrderDirection::Desc),
            None,
            Some(two_days_ago),
            None, None, None,
            &http_client,
        )
        .await;

    let entries = match entries {
        Ok(e) => e,
        Err(err) => return format!("Failed to fetch entries: {:?}", err),
    };

    let mut output = String::new();
    for entry in entries {
        output.push_str(&format!(
            "Title: {}\nPublished at: {}\nURL: {}\n\n{}\n\n===\n\n",
            entry.title, entry.published_at, entry.url, entry.content
        ));
    }

    if output.is_empty() { "No unread entries found.".to_string() } else { output }
}
```

That gets fed into Gemini with this system prompt:

```rust
const SYSTEM: &str = "
You are the user's highly capable Personal Press Secretary.
Your goal is to provide a concise, sophisticated, and engaging editorial briefing.

# Tone and Style:
- Professional yet conversational (think 'The Economist' meets a high-end personal assistant).
- Group related stories into narrative themes rather than just listing them.
- Use 'You' to address the user.
- Highlight *why* a story matters.
- Avoid repetitive bullet points; use short, punchy paragraphs.

# Structure:
1. A brief, warm greeting based on the current time (Morning/Evening).
2. 'The Big Story' - The most impactful trend or news item of the day.
3. 'Other Notable Developments' - Grouped by theme.
4. 'A Quick Look Ahead' - A one-sentence closing thought.

# Constraints:
- Use Markdown for clear headers and bold text.
- Maintain the original 'Read More' links but weave them naturally or place them at the end of sections.
";
```

And the Gemini call itself, using the builder pattern from `gemini-rust`:

```rust
async fn test_gemini() -> String {
    let api_key = env::var("GEMINI_API_KEY").unwrap_or_default();
    let client = Gemini::with_model(api_key, Model::Gemini25Flash).unwrap();
    let data = test_miniflux().await;
    let data = format!("I have the following articles from RSS tracker: {data}");

    client
        .generate_content()
        .with_system_instruction(SYSTEM)
        .with_user_message(data)
        .with_max_output_tokens(2048)
        .with_temperature(0.7)
        .with_top_p(0.9)
        .with_top_k(40)
        .execute()
        .await
        .map(|r| r.text())
        .unwrap_or_else(|e| format!("Error: {}", e))
}
```

Most of this code was actually written with Gemini CLI as an agent. It was
grepping through `~/.cargo/` to figure out the API shape of the crates, because
I didn't feel like reading the docs. It worked.


## The Final Setup

At this point the full picture was: Rust built with naersk, imported as a flake
into the NixOS server configuration, secrets loaded via sops-nix, served behind
nginx. The final `api.nix` for the server:

```nix
{ config, ... }:
let port = 4321; in {
  sops.secrets.kotori = { mode = "0400"; };

  services.rss-summarizer = {
    enable = true;
    port = port;
    portInternal = port + 1;
    envFile = config.sops.secrets.kotori.path;
  };

  services.nginx = {
    enable = true;
    virtualHosts."api.kamoshi.org" = {
      forceSSL = true;
      enableACME = true;
      locations."/" = {
        proxyPass = "http://localhost:${toString port}";
      };
    };
  };
}
```

There are two ports: 4321 is the public one behind nginx, and 4322
(`portInternal`) is a second Axum router only accessible on `127.0.0.1` for curl
testing and any admin-style operations without needing to set up accounts.
Keeping it maximally stateless.

There's also a cache on the summary endpoint so it can only be called fresh once
after 18:00 each day.


## Adding the Dashboard

The last piece was plugging it into
[Glance](https://github.com/glanceapp/glance), a self-hosted personal dashboard.
Glance supports extension widgets that fetch HTML from a URL and render it. The
summary endpoint returns the Gemini output as HTML, so the widget config is
just:

```nix
{ config, pkgs, ... }:
let
  port = 9292;
  domain = "glance.kamoshi.org";
in {
  services.glance = {
    enable = true;
    settings = {
      server.port = port;
      pages = [{
        name = "Home";
        columns = [{
          size = "full";
          widgets = [{
            type = "extension";
            title = "News Summary";
            url = "https://api.kamoshi.org/summary";
            allow-potentially-dangerous-html = true;
            cache = "1s";
          }];
        }];
      }];
    };
  };

  services.nginx.virtualHosts.${domain} = {
    enableACME = true;
    forceSSL = true;
    locations."/" = {
      proxyPass = "http://127.0.0.1:${toString port}";
    };
  };
}
```

Since Glance itself queries the API, the summary endpoint could probably even be
localhost, but I haven't tested that yet.

And that's it. The code is [on GitHub](https://github.com/kamoshi/server).
