---
title = "My Thoughts on NixOS"
tags = ["programming", "nix", "nixos"]
date = "2024-10-02T9:00:00"
draft = true
---

Up until recently, I daily drove Fedora on the main machine I use for development. However, I recently decided to wipe my system and make the jump to [NixOS](https://nixos.org/).

For those that aren't aware, NixOS is an operating system that's centered around the `nix` package manager — a purely functional package manager that allows packages to be built declaratively and without side effects. Additionally, software is never installed globally. It's instead stored in what's called the nix store, in which each package is given a unique subdirectory whose path is composed of a crypographic hash of the package's dependencies followed by the package name and version. Once a package is built and put into the nix store, it's never changed — if you change the build description of the package, this "new" package will be rebuilt into a different subdirectory in the nix store. This provides a few things, including:

-   Multiple versions of a package can exist without interfering with each other.
-   Package management is atomic, and consequently so are the upgrades and rollbacks.
-   Building a package with the same build description will (at least, almost) always yield the same output.

What NixOS does is it allows for you to describe the entire state of your system declaratively using Nix configuration files. So, in addition to packages being handled by Nix, the entire system configuration is handled by it as well. This allows for reproducable configurations, atomic upgrades and rollbacks, and lets you describe the entire state of your system configruation declaratively. This brings in some additional overhead complexity, but yields a reproducible system environment and well composed configurations.

{{! note !}}  
For some clarification, `nix` can refer to both the nix package manager as well as the nix language — the nix language is a purely functional programming language that's used to define nix packages and such.
{{! end !}}

I thought a composable and reproducable approach to handling my system configuration, as well as my user environment (via [home-manager](https://github.com/nix-community/home-manager)), seemed cool, so Nix seemed pretty appealing.

Figuring that since I was already making the switch to NixOS, I decided that I may as well take the opportunity to experiment with some other changes to my workflow as well. Up until this point, I had been running AwesomeWM under X, but I decided to give Wayland a shot this time around. I had also stuck with ZSH for the past couple of years, but `fish` seemed interesting, and who really needs POSIX compliance anyways.

And, so, I spent a couple of weeks setting up NixOS, and this post details my experience, and some thoughts I have after having used it for a month. And, well, it was definitely an experience.

## A Quick Map of the Territory

I'll start by providing a brief overview of how I've got my NixOS system set up, so as to contextualize some of what I touch upon later on.

### Installation

The [downloads page](https://nixos.org/download/) on the NixOS website provides a graphical installer for both GNOME and the KDE Plasma desktop environments. I didn't intend to use either of these, however, and so I went with the minimal ISO image instead. I followed through with the installation without many problems, and ended up at the TTY with the operating system installed. The default configuration for the system is located at `/etc/nixos`, with two files in the folder: `configuration.nix`, which is where the system configuration lives, and `hardware-configuration.nix`, which is generated automatically by the NixOS installer. In `configuration.nix`, you can make changes to essentially part of your system configuration - including setting up users, default programs or services, SSH, your boot settings, and so on. To configure a new user, for example, you'd do the following.

```nix
# configuration.nix
# ...
{
  # ...
  users.users.rcyclegar = {
    isNormalUser = true;
    extraGroups = [ "wheel" "networkmanager" "docker" ];
    shell = pkgs.fish;
  };
}
```

Any time a change is made to the system configuration, you need to run `sudo nixos-rebuild switch` to apply the changes. What this does specifcally is it builds the new configuration, sets it as the default, and attemps to realize it in the running system.

### Flakes

Nix flakes are an experimental feature of Nix, but they've been generally deemed beneficial enough that their usage is becoming increasingly widespread. Essentially, what they provide is a standard way to write and manage dependencies for nix expressions. A flake is any directory with a `flake.nix` file in it describing the dependencies of an expression and how to build that expression. These dependencies are pinned in a `flake.lock` file, which faciliatates reproducibility. I won't go too in-depth with the explanation here, this [page on the NixOS wiki](https://wiki.nixos.org/wiki/Flakes) provides a more involved description.

By default, NixOS doesn't use flakes for its system configuration, but they can be enabled by setting `nix.settings.experimental-features = [ "nix-command" "flakes" ];` in `configuration.nix`. I ended up composing my system configuration using flakes, adding a `flake.nix` file to `/etc/nixos`, and importing `configuration.nix` as a module into the flake.

{{! note !}}  
The package source used by NixOS is called `nixpkgs` - it consist of over 100,000 packages packaged for Nix, and is an input to the flake for my system configuration.
{{! end !}}

### Home Manager

Currently, `configuration.nix` details my system configuration. For a more streamlined way of managing using-level configuration, [Home Manager](https://nix-community.github.io/home-manager/) comes into play. What Home Manager does is it allows me to manage my user-level configuration, such as my programs, configuration files, environment variables, and whatever else might be in my home directory with nix. If I install and configure `git` via Home Manager, it will generate a file a `~/.config/git` for me.

```nix
# home.nix
# ...
{
  # ...
  programs.git = {
    enable = true;
    userEmail = "aboominister@gmail.com";
    userName = "AbooMinister25";
    delta.enable = true;
  };
}
```

And the resulting `~/.config/git/config`.

```ini
[core]
    pager = "/nix/store/a4x3xnxil85j38n9yc8126paqw4dzfg9-delta-0.17.0/bin/delta"

[interactive]
    diffFilter = "/nix/store/a4x3xnxil85j38n9yc8126paqw4dzfg9-delta-0.17.0/bin/delta --color-only"

[user]
    email = "aboominister@gmail.com"
    name = "AbooMinister25"
```

Home Manager can be installed as a standalone tool or as a module in the NixOS configuration. I opted to do the latter, as it allows me to apply my home and system configurations with a single `nixos-rebuild switch` command, rather than relying on the `home-manager` tool, and it makes my configuration feel more cohesive.

So, I created a file at `/etc/nixos/home/home.nix`, added the `home-manager` input to `flake.nix`, and imported `home.nix` as a module.

```nix
{
    # ...
    inputs = {
        # ...
        home-manager = {
            url = "github:nix-community/home-manager/release-24.05";
            inputs.nixpkgs.follows = "nixpkgs";
        };
    };

    outputs = { self, nixpkgs, home-manager, ...}@inputs: {
        # ...
        modules = [
            ./configuration.nix

            home-manager.nixosModules.home-manager
            {
                home-manager.useGlobalPkgs = true;
                home-manager.useUserPackages = true;

                home-manager.users.rcyclegar = import ./home/home.nix;

                home-manager.extraSpecialArgs = {
                    inherit inputs;
                    pkgs-unstable = import nixpkgs-unstable {
                        inherit system;
                        config.allowUnfree = true;
                    };
                };
            }
        ];
    };
}
```

Being able to modularize my configuration is pretty convenient. I can create a new nix file in the `home/` directory for every user level program I want to manage with nix, and if I can't (or don't wish to) configure a specific program with nix, I can still have those configurations managed by home-manager by adding to `home.file`:

```nix
# home.nix
# ...
{
  # ...
  home.file.".config/rofi/" = {
    source = ./rofi;
    recursive = true;
  };
}
```

### Managing with Git

Another thing I wanted to do was to manage my NixOS configuration with `git`. However, the default location of the configuration in `/etc/nixos` requires me to use root permissions whenever I want to modify something. What I _could_ do, however, was move my configuration to my home directory and symlink it to `/etc/nixos` — letting me manage it with `git`.

So I placed my configuration in `~/nixos-config`, created the symlink, and got working.

## Wayland

I mentioned that I wanted to make the switch to Wayland, which required me to choose a compositor. I'm not very picky — the biggest thing I wanted was tiling, but some eye candy and decent animations would have been nice as well.

I decided to opt for [Hyprland](https://hyprland.org/) as my wayland compositor of choice; it has tiling, cool animations, and satisfies the eye-candy element I was looking for.

Nix made it pretty straightforward to get Hyprland running on my system. I can enable the existing NixOS module in `configuration.nix`

```nix
# configuration.nix
# ...
{
  # ...
  programs.hyprland = {
    enable = true;
  };

  programs.xwayland.enable = true;
}
```

And use the Hyprland module for Home Manager to have it manage the configuration.

```nix
# home/hyprland.nix
# ...
{
  wayland.windowManager.hyprland = {
      enable = true;
      xwayland.enable = true;
      settings = {
          # ...
      };
  };
}
```

One more thing to note is that I'm running an Nvidia GPU, so getting the drivers working was another concern. Surprisingly, I didn't have much trouble — I was even able to get the newer 555 drivers running.

```nix
# configuration.nix
# ...
{
  hardware.nvidia = {
    modesetting.enable = true;
    open = false;
    nvidiaSettings = true;
    package = config.boot.kernelPackages.nvidiaPackages.mkDriver {
      version = "555.58.02";
      sha256_64bit = "sha256-xctt4TPRlOJ6r5S54h5W6PT6/3Zy2R4ASNFPu8TSHKM=";
      sha256_aarch64 = "sha256-xctt4TPRlOJ6r5S54h5W6PT6/3Zy2R4ASNFPu8TSHKM=";
      openSha256 = "sha256-ZpuVZybW6CFN/gz9rx+UJvQ715FZnAOYfHn5jt5Z2C8=";
      settingsSha256 = "sha256-ZpuVZybW6CFN/gz9rx+UJvQ715FZnAOYfHn5jt5Z2C8=";
      persistencedSha256 = lib.fakeSha256;
    };
  };
}
```

Hyprland doesn't have [official support for Nvidia hardware](https://wiki.hyprland.org/Nvidia/), and given that I was already on a new operating system, I was pretty pleasantly surprised to discover that it worked fine, without me needing to mess with it much.

I have a two monitor setup, and by default what Hyprland does is it creates a shared set of workspaces between both monitors, and creates more workspaces as you need them. What this ends up giving me, then, is that I might end up with workspaces `1`, `3`, and `6` on my first monitor, and then `2`, `4`, and `5` on my second. I wasn't a huge fan of this, especially considering I was coming from AwesomeWM, which assigns a split set of workspaces between each monitor with independent numbering. Hyprland supports plugins, however, and turns out there's a handful of plugins that exist that provide this functionality. I ended deciding on [this plugin](https://github.com/Duckonaut/split-monitor-workspaces).

To install plugins, Hyprland provides the Hyprland Plugin Manager, `hyprpm`, but this is unsupported on NixOS. Hyprland _does_ provide an alternative way to build plugins through Nix, though, so that's what I used instead. The plugin I wanted already provided a flake I could use, so I went ahead and added it as an input to my `flake.nix`.

```nix
# flake.nix
{
    inputs = {
        # ...
        hyprland.url = "git+https://github.com/hyprwm/Hyprland?submodules=1";
        split-monitor-workspaces = {
            url = "github:Duckonaut/split-monitor-workspaces";
            inputs.hyprland.follows = "hyprland";
        };
        # ...
    };
    # ...
}
```

{{! note !}}
Note that I also added `hyprland` as a flake input here, since `split-monitor-workspaces` required it.
{{! end !}}
and then added it as a plugin through the exposed `plugins` option in Hyprland's `home-manager` module.

```nix
# hyprland.nix
# ...
{
    wayland.windowManager.hyprland = {
        # ...
        plugins = [
            inputs.split-monitor-workspaces.packages.${pkgs.system}.split-monitor-workspaces
        ];
    };
}
```

This ended up not building, however. The issue was that the `split-monitor-workspaces` plugin was pointed to the Hyprland flake specified by `hyprland.url`, whereas the actual version of Hyprland I was using was from `home-manager`, which uses the package provided by the latest stable channel of `nixpkgs`. There was a version mismatch, then.

The fix was easy enough, I just had to change the package that the Hyprland I was using to use the flake version at `inputs.hyprland` instead.

```nix
# configuration.nix
# ...
{
    # ...
    programs.hyprland = {
        enable = true;
        package = inputs.hyprland.packages.${pkgs.stdenv.hostPlatform.system}.hyprland;
        portalPackage = inputs.hyprland.packages.${pkgs.stdenv.hostPlatform.system}.xdg-desktop-portal-hyprland;
    };
    # ...
}
```

and for `home-manager`

```nix
# hyprland.nix
# ...
{
    wayland.windowManager.hyprland = {
        # ...
        package = inputs.hyprland.packages.${pkgs.stdenv.hostPlatform.system}.hyprland;
    };
}
```

Now that all the versions lined up, I could restart and (hopefully) everything would work.

Unfortunately it turns out that everything did not, in fact, work — Hyprland was now crashing on startup. After some inspection of the crash logs, it seemed like it was segfaulting, but I had absolutely no idea why. The issue seemed independent of my usage of the `split-monitor-workspaces` plugin, and it seemed to derive from me switching to the flake version of Hyprland as opposed to the one on nixpkgs. The flake is, as I mentioned, newer than the one on nixpkgs — it's the development version, so it makes sense that there might have been some issues with it. The workspaces issue was a deal breaker for me, though, and I wasn't willing to use Hyprland without awesome-style workspaces, so I figured I would put some more effort into debugging the issue.

Looking closer at the crash logs again, it seemed like the issue was linked to `aquamarine`, a rendering backend that Hyprland uses. It turns out that the version of Hyprland available through the stable channel of nixpkgs was not yet using the `aquamarine` backend, and so it was the change in this backend that was causing the flake version to crash. `aquamarine` seemed to be trying to use the GPU at `/dev/dri/card0`, but the Nvidia GPU I was actually using was at `/dev/dri/card1`. I poked around the documentation for a while, and it turns out that the `aquamarine` backend allows you to set the `AQ_DRM_DEVICES` environment variable to specificy the GPU you're running (in my case, `/dev/dri/card1`). Setting this environment variable fixed this issue, and I was (finally) able to run Hyprland.

## Trying to Package Something

The next step was to install a display manager, so I didn't need to launch Hyprland from the TTY every time I booted my system. My initial choice was to use LightDM — nixpkgs _does_ have a handful of LightDM greeters I could choose, but I deemed that I would prefer using something like [this webkit2 greeter](https://github.com/sbalneav/lightdm-webkit2-greeter). Unfortunately it was no longer maintained, but after a bit of searching I found [Nody Greeter](https://github.com/JezerM/nody-greeter), which seemed like a decent alternative.

Nody greeter wasn't already packaged for Nix, so I figured I'd try and package it myself. Having only been a few days into using the OS and possessing only a rudimentary knowledge of how Nix and packaging with it worked, I didn't have a great time, and ultimately couldn't get it working. I couldn't find the root of the issue, and so I decided to pivot.

I decided to use SDDM instead, found a decent looking theme that I _did_ manage to package correctly, and that worked for a while, until I eventually ended up switching to greetd and [tuigreet](https://github.com/apognu/tuigreet). Nix made this very straightforward.

```nix
# configuration.nix
# ...
{
  # ...
  services.greetd = {
    enable = true;
    settings = {
      default_session = {
        command = "${pkgs.greetd.tuigreet}/bin/tuigreet --time --issue --user-menu --remember -
    -cmd Hyprland --remember-user-session --asterisks";
        user = "greeter";
      };
    };
  };
}
```

Overall, I think this reveals one of the issues with NixOS — it's great when it works and tools exist for your usecase, but when doing something somewhat unconventional, it can end up being a pain, especially given the lack of precedence or documentation in regards to some of these things. I'm not sure how unconventional using a custom LightDM greeter was, but I couldn't find many people who had already done it, and so I was left pretty much in the dark as to how I wanted to go about it. With any other Linux distribution, I wouldn't have had nearly as much of a problem trying to get these tools working.

That said, NixOS _is_ different from other linux distributions in how it operates, and I signed up to deal with the consequences of those differences when I decided to switch. As such, villainizing the operating system for this is unfair, and it's a given that getting things to work is going to involve a different process. There is undoubtedly a learning curve, and figuring out how to package something isn't immediately obvious, but the wiki is pretty great, and the quality and degree of resources has been improving.

## Development Environments

Development on NixOS is somewhat different from a traditional desktop Linux system. To start with, Nix doesn't follow the [filesystem hierarchy standard](https://en.wikipedia.org/wiki/Filesystem_Hierarchy_Standard), which prevents you from running any random dynamically linked executable. Instead, everything is stored in the immutable Nix store. This has some implications development wise, and I'll describe how they affected me by walking through how I set up this system for development.

To start with, I needed to install the relevant tools. I primarily work with Python and Rust, so environments for those were what I wanted to set up first. The first question was, well, how do I install these? NixOS doesn't ship with a standard Python installation, so what I first reached for was just installing it via home-manager by listing it in `home.packages`. After some googling, however, it seemed like this wasn't the recommended solution — development packages don't generally go in your system or home configuration. Instead of installing Python on the user level, it was recommended to install project-specific software within something called a _development shell_.

So, hey, what are development shells? The top three results from a quick google for "Nix development shells" are:

```
- Development environment with nix-shell
- Declarative shell environments with shell.nix
- Managing development environments with Nix
```

Digging a little deeper, I found that "development shells", or "shells" in general, usually referred to or involved one of the following:

-   `nix-shell`
-   `nix shell`
-   `nix develop`

This is confusing. A good place to start is probably by defining what exactly a "development shell" in Nix is. Nix gives you the ability to create temporary shell environments with tools and software needed to develop or debug packages. This is something you can use `nix-shell` for.

Originally the purpose of `nix-shell` was to, given a derivation, place you into a shell that is similar to the derivation's build environment. This allowed you to develop the package, debug and run through the build steps, etc. `nix-shell` also allows you to temporarily get access to a package without permanently installing it. For example, if I wanted to run `cowsay` one-off, I could do `nix-shell -p cowsay`, which puts me into a shell with `cowsay` available.

```shell
$ nix-shell -p cowsay
...
[nix-shell:~]$ cowsay i love oranges
 ________________
< i love oranges >
 ----------------
        \   ^__^
         \  (oo)\_______
            (__)\       )\/\
                ||----w |
                ||     ||
```

Later, people began using `nix-shell` to create development environments via `pkgs.mkShell`. Taken from the NixOS wiki, the following `shell.nix` defines a development environment with Ruby available.

```nix
{ pkgs ? import <nixpkgs> {} }:
  pkgs.mkShell {
    nativeBuildInputs = with pkgs.buildPackages; [ ruby_3_2 ];
}
```

Running `nix-shell shell.nix` would drop you into a shell with `ruby` available.

Now, when flakes came around, so did the Nix command line interface, `nix`, which collected a bunch of common `nix-X` commands as subcommands under `nix`. However, the functionality of `nix-shell` in particular was split up. This gave us a few commands, including `nix shell` and `nix develop`.

`nix shell` creates a shell with the outputs of a given flake. To replicate my cowsay example from earlier, if I wanted to bring `cowsay` into my environment with `nix shell`, I would do `nix shell nixpkgs#cowsay`.

`nix develop` allows you to debug derivations by placing you into a shell that is similar to the derivation's build environment, or to create a development environment with `pkgs.mkShell`. The difference is that instead of a `shell.nix`, `nix develop` uses flakes. Specifically, for a development environment, `nix develop` will create the development shell defined in the `devShell` output of the flake. The Rust development environment of the static site generator that powers this blog is as follows.

```nix
# flake.nix
{
  description = "A basic flake with a shell.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };

  outputs ={ self, nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem
    (system:
      let
        overlays = [ ( import rust-overlay )];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
      in
      with pkgs;
      {
        devShells.default = mkShell {
          buildInputs = [
            (rust-bin.stable.latest.default.override {
              extensions = ["rust-src"];
            })

            pkgs.openssl
            pkgs.pkg-config

            pkgs.go

            nodejs nodePackages.pnpm
          ];
        };
      }
    );
}
```

{{! note !}}

To obtain `rust` and `cargo`, I used the versions provided by [https://github.com/oxalica/rust-overlay](https://github.com/oxalica/rust-overlay). It also turns out that in order to build, rust wanted access to libssl, so I added `pkgs.openssl` to the dev shell's build inputs as well.

My static site generator also depends on Go and Node, so I added those.

{{! end !}}

Great, that wasn't too bad, and now I had a functional development environment for Rust. Doing the same for Python shouldn't be too bad, yeah?

Turns out it _was_ that bad.

So, the big gripe I had was that when developing Python, I did not want to package my projects with Nix — I wanted to keep using the tools I had always been using, such as `pdm` and `uv`. All I wanted Nix to do here was stick me in an environment with Python and these tools available.

My first inclination was to just create a shell that did exactly that — put me into an environment with python and pdm installed, and from there I would just use `pdm` as normal.

```nix
# flake.nix
{
  description = "A basic flake with a shell.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs ={ self, nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem
    (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
      in
      with pkgs;
      {
        devShells.default = mkShell {
          buildInputs = [
            pkgs.python312
            pkgs.pdm
          ];
        };
      }
    );
}
```

This _seemed_ like it was fine, up until I tried to use any sort of executable program I had installed. This is where the aforementioned issue with NixOS not being FHS compliant popped up. When trying to install and use Ruff, for example, I would get the following:

```shell
$ pdm add ruff
Adding packages to default dependencies: ruff
  0:00:03 🔒 Lock successful.
Changes are written to pyproject.toml.
Synchronizing working set with resolved packages: 1 to add, 0 to update, 0 to remove

  ✔ Install ruff 0.7.3 successful
  ✔ Install testing-python 0.1.0 successful

  0:00:01 🎉 All complete! 1/1
$ pdm run ruff
Could not start dynamically linked executable: /home/rcyclegar/env-definitions/test-python/.venv/bin/ruff
NixOS cannot run dynamically linked executables intended for generic
linux environments out of the box. For more information, see:
https://nix.dev/permalink/stub-ld
```

so, yeah, I can't run any of my linters, formatters, `pytest`, and whatever else I have installed this way.

What I _could_ do though was add these executables to the build inputs of my shell, but I wanted to avoid that. Ideally my projects would have a single source of truth I could build from (that being my `pyproject.toml`) rather than having everything duplicated across different components.

The next alternative was to further nixify everything and create nix expressions for my project. Like I mentioned earlier, though, I didn't want to do this, and instead continue using the tools I had always been using. After some looking around, I found [poetry2nix](https://github.com/nix-community/poetry2nix), which autogenerates Nix derivations on the fly by parsing your `pyproject.toml` and `poetry.lock` files. Now, this is nice enough if I was using poetry, but I don't, and neither `uv` or `pdm` have any established alternatives. However, it turns out that `poetry2nix` is implemented using a project called [pyproject.nix](https://github.com/nix-community/pyproject.nix), which is a collection of Nix utilitites to work with python project metadata. I took a quick crack at using it, but the issue was that it is a lot more barebones than `poetry2nix`, and I pretty much needed to figure out everything myself, including all the special cases with how `uv` and `pdm` handle things, to get it to work. That was too much of a time commitment than I was willing to make just to get a python development environment working.

At this point, it turns out that NixOS has an escape hatch — `buildFHSUserEnv`, which essentially lets me create a lightweight FHS-compatible sandbox. This is usually a last resort, but I figured I was already at that point anyways. Here's a shell for python and `uv`, which works pretty great:

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs ={ self, nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem
    (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
      in
      with pkgs;
      {
        devShells.default = (pkgs.buildFHSUserEnv {
          name = "uv";
          targetPkgs = pkgs: (
            with pkgs;[
              python312
              uv
            ]
          );
          runScript = "fish";
        }).env;
      }
    );
}
```

{{! note !}}
Another option I had was to use a tool called [distrobox](https://github.com/89luca89/distrobox), which lets me use a multitude of different linux distributions in my terminal, while being closely tied to the host system.
{{! end !}}

Now that everything worked, I figured I'd round out the corners of my workflow for niceties and convenience.

### Direnv

[Direnv](https://direnv.net/) is a pretty neat tool that exists. What it does is it loads and unloads environment variables depending on your current directory. You can use it to automatically load/unload nix shells upon navigating to your project's directory. [Nix-direnv](https://github.com/nix-community/nix-direnv) makes the process even easier.

Getting it working is fairly simple. You enable it via home-manager

```nix
{
  # ...
  programs.direnv = {
    enable = true;
    enableFishIntegration = true;
    nix-direnv.enable = true;
  };
}
```

add `use flake` to a `.envrc` file in your project directory, and run `direnv allow`. Anytime you navigate into this directory, direnv will automatically activate the nix shell.

`direnv` also has a nice side effect of automatically putting me into a `fish` shell, instead of the default `bash` shell that devshells use.

There was a small issue, though. It turns out that `direnv` doesn't work in FHS-sandboxes, like the devshell I created for my python projects with `buildFHSUserEnv`. It's not a dealbreaker, it's not that much additional work to run `nix develop`, it's just kind of annoying.

### VSCode

The whole FHS situation popped up again with VSCode — It turns out that extensions I have installed traditionally through VSCode won't be able to run any pre-compiled binaries they come with. Now, I could configure VSCode via nix, and use the nix expressions available at [nix-vscode-extensions](https://github.com/nix-community/nix-vscode-extensions), and everything probably would've worked fine. At the moment, though, I didn't want to spend too much more time configuring my system, so I opted to use `vscode.fhs` instead, which launches the editor inside a FHS compliant environment with `buildFHSUserEnv`.

However, that had its own issues. If I run it within a FHS environment, VSCode won't launch properly — it just shows me a blank, transparent square. Some sort of rendering issue. The fix was to run VSCode with the `--disable-gpu-compositing` flag, which works for now. I intend to move to a more permanent, fully nixified configuration in the future, once I have the time to sit down and further configure everything.

## SSH

Setting SSH up took longer than I expected it to. My primary use of it is for authenticating with GitHub, which is simple enough in principle. I generated a keypair, registered the public key with my GitHub account, and everything worked fine. The issue came when I wanted my workflow to be more convenient.

So, I'm running a fairly barebones system, which means that I didn't have any sort of SSH agent or keychain installed. What an SSH agent does is it runs in the background and manages your SSH keys, so that once a key is registered, you don't need to enter in a passkey every time you want to use it. Without this, every time I wanted to push to GitHub, for example, I had to re-enter my SSH passphrase. So naturally, I set up `ssh-agent` via my home-manager configuration.

```nix
# home.nix
{
  # ...
  services.ssh-agent.enable = true;
}
```

I could then use `ssh-add` to add my keys to the agent. However, what I really wanted was to replicate the functionality that my previous systems had, namely barely ever needing to enter an SSH passphrase, and have all of my keys auto-add to the agent on login. It turns out this functionality is provided by a keyring.

What a keyring does is it stores passwords and encryption keys, and is usually unlocked with your login password upon login. I can use it to store my SSH keys, and have those open and available once I log into my system.

I decided to use `gnome-keyring` since it already seemed fairly prevalent. `gnome-keyring` provides its own ssh-agent, so I needed to disable `ssh-agent` to proceed.

```nix
# configuration.nix
{
  # ...
  services.gnome.gnome-keyring.enable = true;
  programs.seahorse.enable = true;
  security.pam.services.greetd.enableGnomeKeyring = true;
  security.pam.services.login.enableGnomeKeyring = true;
}
```

```nix
# home.nix
{
  # ...
  services.gnome-keyring.enable = true;
  services.gnome-keyring.components = [
    "ssh"
    "secrets"
  ];
}
```

{{! note !}}  
`seahorse` is sort of a graphical frontend for `gnome-keyring`, you can use it to manage your keyring.
{{! end !}}

Upon restart, I could see that `gnome-keyring-daemon` was now running, and `seahorse` was showing me that my keyring was unlocked, but for some reason `git` was still prompting me for my SSH passphrase when pushing to a GitHub remote. After some looking into it, turns out I need to set `SSH_AUTH_SOCK` accordingly, which is used by the ssh agent. The `ssh-agent` service set it automatically, but now I needed to set it myself.

```nix
# home.nix
{
  # ...
  home.sessionVariables = {
    # ...
    SSH_AUTH_SOCK = "/run/user/$(id -u)/keyring/ssh";
  };
}
```

On reboot, everything worked fine.

## Links not opening in Firefox

I noticed that apps (namely VSCode and Discord) weren't opening links in my browser (Firefox). At first, I thought this might be because they were both electron apps, but that didn't make some sense. After looking through [many GitHub issue threads](https://github.com/NixOS/nixpkgs/issues/160923), it turns out that the root of the issue was that apps sandboxed with `buildFHSUserEnv` were unable to properly use `xdg-open`.

The solution was to enable `xdg.portal.xdgOpenUsePortal` and install `xdg-utils`. I'm still not sure _exactly_ why this was happening in the first place, and I figure that this should probably be better documented somewhere, but I'm glad it works.

```nix
# configuration.nix
{
  # ...
  xdg = {
    portal = {
      enable = true;
      xdgOpenUsePortal = true;
      # ...
    };
  };
}
```

```nix
# home.nix
# ...
{
  # ...
  home.packages = with pkgs; [
    # ...
    xdg-utils
  ];
  # ...
}
```

## Personal Thoughts

So far, most of what I've discussed has been in regards to the issues I've encountered with NixOS (and it wasn't an exhaustive list either), but I think that paints a more negative view of the operating system than what I genuinely feel towards it. I'll go over some things I both like and dislike about this operating system.

### What I Like

-   The universal declarative configuration is nice, I'm liking it a lot more than I thought I would. The majority of the software I use supports being configured with nix, and everything comes together very neatly. Additionally, just having my system configuration consolidated in a single location, and edited through a single interface, makes the system itself just that much easier to reason about.
    -   Being able to consolidate the configuration for all of my user-level programs and software with `home-manager`, and compose the entire thing with flakes is a good user experience.
-   I'll reiterate in more depth how nice it's been to configure system-level things with nix. Getting Nvidia drivers working was trivial, enabling and configuring audio via pipewire was easy, and working with things such as bluetooth (among the other [hardware options](https://search.nixos.org/options?channel=24.05&from=100&size=50&sort=relevance&type=packages&query=hardware) NixOS makes available) is very convenient.
    -   Additionally, swapping out componenents of your system, such as your desktop environment, is made incredibly easy. It's a few line changes for me to go ahead and install Gnome, for example, and everything just works.
-   It's reproducible and atomic — one of the main "tenets" of NixOS is that package builds are isolated and reproducible, and you're able to easily roll back to previous revisions of your system configuration in case something breaks without any hassle. This makes system updates significantly less of a stressful endeavor.
-   I did complain about the pitfalls of shell environments in regards to Python development shells, but aside from that I have found myself really liking them. I can create reproducable shell environments for different types of projects, and it keeps projects and their development dependencies isolated from the rest of the system, and thus my user level environment is kept clean.
-   Being able to manage my system's configuration with `git`, store it on `GitHub`, and prospectively replicate it on any machine I use in the future is pretty appealing. Granted, I don't know how true this will actually hold in practice, but it's an appealing idea.
-   nixpkgs is huge and frequently updated, and I'm impressed that I can use both the stable and unstable versions of nixpkgs without so much as a second thought.

### What I Don't Like

-   I haven't mentioned it yet, but I'm not fond of the Nix language. Maybe it's a lack of experience or exposure to functional languages in general, but it's just...not very wieldy to me. I've seen it described as "JSON with functions", and I see the appeal when you're configuring something without much programatic logic, but it becomes less intuitive once you breach that. Granted, like I mentioned, this could all be attributed to my own inexperience. That said, aside from writing Nix, I've found that it isn't much fun to debug it either. The errors aren't great, in my experience, and make debugging failed configurations _that_ much harder.
-   The fact that I can't run dynamically linked scripts off of GitHub, without having them packaged by Nix. This stems from the operating system not adhering to the filesystem hierarchy standard, but it's still not fun. Granted, tools such as [`nix-alien`](https://github.com/thiagokokada/nix-alien) and [`nix-ld`](https://github.com/nix-community/nix-ld) exist, but they aren't foolproof. Nix was designed with this in mind, so I won't villainize the operating system for it, but it does make for a frustrating experience at times, and was ultimately the root of most of the major issues I had while using Nix.
-   I mentioned that I'm fond of how easy it is to do things like switching out a desktop environment, or handling various system and hardware level configurations with Nix. On the flip side of the coin, though, anything that isn't "officially" supported by Nix ends up being a pain to implement. My earlier issues with getting `nody-greeter` packaged for `lightdm` is an example of this. It's not that Nix is incapable of doing so, but rather the scarcity of documentation, precedence, and examples for workflows and setups that may be less common or unconventional end up making it such that a lot more effort is required to find solutions for these issues.
-   And that leads me into the issue of documentation. Resources and documentation have improved a _lot_, but at the same time it's still lacking a lot of things. My earlier issue with sandboxed applications being unable to open links in my default browser seems like it should be better documented, and it's easy to be confused at first as to what sort of tooling you should be using for things like development shells (flakes with `nix shell` or no flakes?).

I think in general, NixOS has a fairly steep learning curve. It'll be fine enough if you're using a straightforward and conventional workflow, but a lack of experience with functional languages, how packaging with Nix works, and other intricacies of the operating system might make it more of a pain to use than most people are willing.

And on top of that, if all someone is looking for is a consolidated and declarative system configuration, there's other tools out there that can achieve this, without the additional complexity that `NixOS` brings. Or, using `nix` as a package manager on a non NixOS system is another option.

Was it worth it for me?

It was definitely a time sink, I put in a _lot_ of work, maybe more than was warranted, but I did enjoy myself. I ultimately like NixOS, at this point my system is at a state where it's just working, and I don't need to be worried about stuff breaking. The benefits of Nix appeal to me, and the ecosystem is in the process of growing. As it stands, I don't see myself switching operating systems for now — I'll probably continue to use NixOS for the time being. Many of the issues I encountered can likely be attributed to just my lack of experience; NixOS is different from most other operating systems, and it's unfamiliar. I made somewhat of an impulsive blind jump, and switched up nearly all aspects of the software I daily drive (X to Wayland, `zsh` to `fish`, and so on), which created a greater pit of unfamiliarity that I had thrown myself into.

I won't decide if it was worth it or not yet, but I'm sticking with it for now.

My system configuration is all [on GitHub](https://github.com/AbooMinister25/dotfiles).
