# Zellij Tab Bar Next-Generation

[![Lint](https://github.com/fulldecent/zellij-tab-bar-ng/actions/workflows/lint.yml/badge.svg)](https://github.com/fulldecent/zellij-tab-bar-ng/actions/workflows/lint.yml) [![Build and test](https://github.com/fulldecent/zellij-tab-bar-ng/actions/workflows/build-test.yml/badge.svg)](https://github.com/fulldecent/zellij-tab-bar-ng/actions/workflows/build-test.yml)

A drop-in replacement for Zellij’s default `tab-bar` plugin. Ribbon tabs on the active Zellij theme. The goal is to become the default tab bar.

Zellij Tab Bar Next-Generation offers:

* Ribbon tabs, a session placard, and a `+` button
* Fully responsive across all terminal widths
* Click a tab to switch to it. Click `+` to open a tab

![Ribbon tabs on the default theme: session placard, three tabs, plus button](screenshot.svg)

This is a [Zellij](https://zellij.dev/) plugin and supports Zellij 0.45 or newer.

A [Nerd Font](https://www.nerdfonts.com/) is required for the slants `` (U+E0BA) and `` (U+E0BC).

## Try it out

1. Select the latest wasm release and install to your plugins folder:

   ```sh
   name="com.github.fulldecent.zellij-tab-bar-ng.wasm"
   base="https://github.com/fulldecent/zellij-tab-bar-ng/releases/latest/download"
   mkdir -p ~/.config/zellij/plugins
   curl -fL -o ~/.config/zellij/plugins/"$name" "$base/$name"
   curl -fL -o ~/.config/zellij/plugins/"${name}.sigstore.jsonl" "$base/${name}.sigstore.jsonl"
   ```

   If you have the `gh` command, verify the file build attestation:

   ```sh
   gh attestation verify ~/.config/zellij/plugins/"$name" \
     -R fulldecent/zellij-tab-bar-ng \
     --bundle ~/.config/zellij/plugins/"${name}.sigstore.jsonl"
   ```

2. Start a new session (or tab if already in Zellij):

   ```sh
   zellij --layout-string 'layout {
       pane size=1 borderless=true {
           plugin location="file:~/.config/zellij/plugins/com.github.fulldecent.zellij-tab-bar-ng.wasm"
       }
       pane
       pane size=1 borderless=true {
           plugin location="status-bar"
       }
   }'
   ```

   Grant `ReadApplicationState` and `ChangeApplicationState` when Zellij asks. Click that row and press `y`.

3. Click a tab. Click `+`. Narrow the terminal and watch the hidden tabs become `← +N` and `+N →`.

## Installation

Use [Try it out](#try-it-out) to download the plugin (this also confirms permissions).

Open your `~/.config/zellij/config.kdl` and replace the existing `tab-bar` line with:

```kdl
tab-bar location="file:~/.config/zellij/plugins/com.github.fulldecent.zellij-tab-bar-ng.wasm"
```

Quit and reopen Zellij to load your new tab bar!

If you already have `layouts/default.kdl`, keep `plugin location="tab-bar"` on the top row.

To upgrade: repeat [Try it out](#try-it-out) > step 1, then start a new session.

## Usage

The top row keeps the session placard (`Zellij` and the session name) and a `+` button. Tabs that do not fit collapse to `← +N` / `+N →`. A tab that still does not fit shrinks its name with `…`. Click a tab to switch to it. Click `+` to open a tab. Hovering `+` uses the theme hover color.

The bar, the session placard, the `+` face, and the `+` hover color come from the session’s existing theme on `ModeUpdate`. The ribbon fills stay the classic green (active) and gray (inactive). There is no color config in this plugin.

80 columns:

![Ribbon tabs at 80 columns](screenshot.svg)

Mouse over `+`, 80 columns:

![Mouse over the plus button at 80 columns](shots/hover.svg)

Eight tabs, 80 columns. Five names fit. The other three collapse to `+3 →`:

![Eight tabs at 80 columns](shots/overflow.svg)

### Responsive display

A wide terminal keeps every name and the `+` button. A narrower one drops neighbors into `← +N` / `+N →`, then shortens those counts, then shrinks the active name. The visible width never exceeds the screen.

20 columns:

![This tab bar at 20 columns](shots/w20.svg)

40 columns:

![This tab bar at 40 columns](shots/w40.svg)

100 columns:

![This tab bar at 100 columns](shots/w100.svg)

Widths 1 through 100 are covered by `tests/widths.rs`. The pictures are SVG from [Zellij Plugin Snapshot](https://github.com/fulldecent/zellij-plugin-snapshot). The same scenes are in the [gallery](README-RESPONSIVE-GALLERY.md).

### Uninstall

Edit your `~/.config/zellij/config.kdl` again and replace the `tab-bar` line with:

```kdl
tab-bar location="zellij:tab-bar"
```

## Development

Thank you for taking an interest in improving Zellij Tab Bar Next-Generation.

You will need [rustup](https://rust-lang.github.io/rustup/), Git and your platform's native build tools. This project pins the toolchain in [rust-toolchain.toml](rust-toolchain.toml); rustup reads that file and installs matching `rustc`, Cargo, rustfmt, Clippy and the `wasm32-wasip1` target.

A packaged `rustc` or `cargo` from apt, dnf or Homebrew does not apply `rust-toolchain.toml`. Install rustup, then invoke the toolchain rustup selected:

```sh
"$(rustup which rustc)" --version
"$(rustup which cargo)" --version
```

`rustup which` prints the binary for the active toolchain (from `rust-toolchain.toml` in this directory). Quoting the command substitution runs that binary even when another `rustc` or `cargo` is earlier on `PATH`. That Cargo still searches `PATH` for `rustc`. A packaged `rustc` has no `wasm32-wasip1` standard library, so the build stops with E0463, `can't find crate for core`. [.cargo/config.toml](.cargo/config.toml) sets Cargo's [`rustc-wrapper`](https://doc.rust-lang.org/cargo/reference/config.html#buildrustc-wrapper) to [tools/rustc-wrapper](tools/rustc-wrapper), which runs the channel in `rust-toolchain.toml` when that happens. Doctests use [tools/rustdoc](tools/rustdoc) ([`build.rustdoc`](https://doc.rust-lang.org/cargo/reference/config.html#buildrustdoc)). A packaged `rustdoc` reports E0514 for rlibs that toolchain built.

### Linux

On Ubuntu 22.04+ or Debian 12+:

```sh
sudo apt update
sudo apt install git build-essential rustup
```

On Fedora:

```sh
sudo dnf install git gcc rustup
```

If your distribution has no `rustup` package, follow [other rustup installation methods](https://rust-lang.github.io/rustup/installation/other.html).

### macOS

```sh
xcode-select --install
brew install git rustup
```

Homebrew's `rust` formula is a standalone compiler. It does not honor `rust-toolchain.toml`. Use `rustup` instead.

### Windows

```powershell
winget install --exact --id Git.Git
winget install --exact --id Microsoft.VisualStudio.2022.BuildTools --override "--wait --passive --norestart --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
winget install --exact --id Rustlang.Rustup
```

Clone the project and build:

```sh
git clone https://github.com/fulldecent/zellij-tab-bar-ng.git
cd zellij-tab-bar-ng
"$(rustup which rustc)" --version
"$(rustup which cargo)" --version
"$(rustup which cargo)" check --locked
"$(rustup which cargo)" build --locked
```

> [!NOTE]
> Run those lines from this repository. `rust-toolchain.toml` is how the version is selected. If a check fails with “compiled by an incompatible version of rustc”, another compiler already wrote `target/`. That directory is gitignored. Remove it (`rm -rf target`) and run the same four lines again.

Run this directly with:

```sh
zellij --layout layout/dev.kdl
```

### Testing

```sh
"$(rustup which cargo)" test --locked --target "$("$(rustup which rustc)" -vV | sed -n 's/^host: //p')"
"$(rustup which cargo)" fmt --all -- --check
"$(rustup which cargo)" clippy --all-targets --locked -- -D warnings
"$(rustup which cargo)" build --release --locked
```

`cargo test` runs the Rust tests in `src/` and `tests/widths.rs`. Pass `--target` with the host triple from `rustup which rustc`. The target in [.cargo/config.toml](.cargo/config.toml) is `wasm32-wasip1`, and that wasm cannot run these tests. [tests/snapshot.sh](tests/snapshot.sh) builds the release wasm, runs every script in [shots/](shots/), and exits non-zero unless those bytes match the committed `shots/*.ansi.txt` files. SVG is traced. `diff` does not compare it. The 80-column picture is committed as [screenshot.svg](screenshot.svg) at the repository root. The other pictures are committed beside their YAML scripts.

The host is [Zellij Plugin Snapshot](https://github.com/fulldecent/zellij-plugin-snapshot) 0.2.2. Install that published crate once. `cargo install` compiles it and puts `zellij-plugin-snapshot` on `PATH`. `[dependencies]` and `[dev-dependencies]` link a library into the plugin wasm or into `cargo test`. This host is a command, so those fields leave it uninstalled.

```sh
"$(rustup which cargo)" install zellij-plugin-snapshot --version 0.2.2 --locked
sh tests/snapshot.sh
```

```sh
npx prettier@latest --check . --write
npx markdownlint-cli@latest "**/*.md" --fix
```

### Releases

Use `fix:`, `feat:` or `BREAKING CHANGE:` in your commit messages. When you push commits to `main`, the release workflow will create a pull request which bumps the version and kicks off a release when you merge.

The [release workflow](.github/workflows/release.yml) uses Release Please's `rust` release type. That pull request bumps the version in [Cargo.toml](Cargo.toml) and [Cargo.lock](Cargo.lock). The package name is `plugin`. `include-component-in-tag: false` keeps the tag `vMAJOR.MINOR.PATCH`. A tag shaped `plugin-v1.2.3` does not pass the publish step.

The released binary is named `com.github.fulldecent.zellij-tab-bar-ng.wasm`.

### Maintenance

The project administrator completes these maintenance tasks each month.

1. Identify external Actions in [.github/workflows](./.github/workflows) and update them when it is safe.
1. Review the Rust toolchain in `rust-toolchain.toml`.
1. Review [zellij-plugin-snapshot](https://github.com/fulldecent/zellij-plugin-snapshot) releases. Update every `cargo install zellij-plugin-snapshot --version` line. Regenerate each `shots/*.ansi.txt` and its SVG when the host output changes. Commit [screenshot.svg](screenshot.svg) at the repository root. Commit the other SVG files beside their YAML scripts.

## Project scope

This tab bar is intended to replace the built-in Zellij tab bar ([`default-plugins/tab-bar` at v0.45.1](https://github.com/zellij-org/zellij/tree/v0.45.1/default-plugins/tab-bar)). In this interest, we limit this project only to the following features which we consider to be the highest priority.

* Ribbon tabs, with the active tab kept on screen
* Responsive display that never exceeds the screen
* Theme colors from the active Zellij session for the bar, the placard, and the `+` button

Out of scope:

* No separate color scheme file. The ribbon fills stay the classic green and gray
* No format-string language or extra widgets
* It does not replace the status bar
* Layouts do not point at an HTTPS plugin URL

## References

1. We use "Title Case" only for proper nouns, this includes the name of our project.
1. This project is built based on [best practices documented in zellij-plugin-template](https://github.com/fulldecent/zellij-plugin-template).
1. This project is built based on [best practices documented in rust-template](https://github.com/fulldecent/rust-template/).
1. This project is built based on [best practices documented in project-template](https://github.com/fulldecent/project-template).
1. Zellij offers another installation method that points your layout configuration to an HTTPS URL. We consider that feature wrong and deprecated.
1. We speak of the official rustup installation recommendation as ugly. [Reported upstream](https://github.com/rust-lang/rust/issues/163468).
1. Render snapshots follow [Zellij Plugin Snapshot](https://github.com/fulldecent/zellij-plugin-snapshot) 0.2.2. [tests/snapshot.sh](tests/snapshot.sh) compares every `shots/*.yaml` against `shots/{name}.ansi.txt`. [screenshot.svg](screenshot.svg) is the picture in this README. Other SVG files stay beside their YAML scripts.
1. The fitting rule follows [Zellij’s `default-plugins/tab-bar`](https://github.com/zellij-org/zellij/tree/v0.45.1/default-plugins/tab-bar) at tag `v0.45.1` (MIT): keep the active tab, add neighbors toward the shorter side, and collapse the rest to `← +N` / `+N →`. Column width uses [`unicode-width`](https://docs.rs/unicode-width), as that tab bar does.
1. Zellij 0.45.1 expands `~` in a `file:` plugin location. [Confirmed while shipping zellij-status-bar-ng](https://github.com/fulldecent/zellij-status-bar-ng).
1. Release Please's [`rust` release type](https://github.com/googleapis/release-please/blob/main/docs/customizing.md) bumps [Cargo.toml](Cargo.toml) and [Cargo.lock](Cargo.lock). [`include-component-in-tag: false`](https://github.com/googleapis/release-please/blob/main/docs/manifest-releaser.md) keeps the GitHub tag `vMAJOR.MINOR.PATCH`.
1. [tests/snapshot.sh](tests/snapshot.sh) runs Cargo with `rustup run` and the active toolchain. `rustup which cargo` is the inner toolchain binary. On GitHub Actions that binary was the default stable toolchain, which did not have `wasm32-wasip1`. `rustup run` does not put `rustc` on `PATH`. [tools/rustc-wrapper](tools/rustc-wrapper) is Cargo's `rustc-wrapper`. It runs the channel in [rust-toolchain.toml](rust-toolchain.toml) when `PATH`'s `rustc` has no `wasm32-wasip1` sysroot. [tools/rustdoc](tools/rustdoc) is `build.rustdoc`, so doctests load those rlibs. On macOS that compiler's `rust-lld` does not find `libLLVM.dylib` on its recorded rpath, so the wrapper sets `DYLD_LIBRARY_PATH` for wasm links ([rust-lang/rust#76921](https://github.com/rust-lang/rust/issues/76921)).
1. The Rust ignore rules in [.gitignore](.gitignore) come from [GitHub's Rust gitignore](https://github.com/github/gitignore/blob/main/Rust.gitignore).
1. This project is released under the [MIT license](./LICENSE.md).
