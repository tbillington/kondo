# Kondo 🧹

Cleans `node_modules`, `target`, `build`, and friends from your projects.

Excellent if

- 💾 You want to back up your code but don't want to include GBs of dependencies
- 🧑‍🎨 You try out lots of projects but hate how much space they occupy
- ⚡️ You like keeping your disks lean and zippy

<br />

<p align="center">
    <strong>14 Supported Project Types</strong>
</p>
<p align="center">
<a href="https://doc.rust-lang.org/cargo/">Cargo</a>
- <a href="https://nodejs.org/">Node</a>
- <a href="https://unity.com/">Unity</a>
- <a href="https://www.scala-sbt.org/">Scala SBT</a>
- <a href="https://docs.haskellstack.org/">Haskell Stack</a>
- <a href="https://maven.apache.org/">Maven</a>
- <a href="https://www.unrealengine.com/">Unreal Engine</a>
- <a href="https://www.python.org/">Python</a>
</p>
<p align="center">
<a href="https://jupyter.org/">Jupyter Notebook</a>
- <a href="https://cmake.org">CMake</a>
- <a href="https://getcomposer.org/">Composer</a>
- <a href="https://dart.dev/">Pub</a>
- <a href="https://elixir-lang.org/">Elixir</a>
- <a href="https://swift.org/">Swift</a>
</p>
<p align="center">
Pull requests are welcome, it's <a href="https://github.com/tbillington/kondo/pull/76/files">easy to extend</a>!
</p>
<p align="center">
Current <a href="https://github.com/tbillington/kondo/blob/a7af95484d364bbb12eb3b40b0d860424dd1b714/kondo-lib/src/lib.rs#L22-L54">deleted directories config</a>.
</p>

<img width="972" alt="kondo cli cleaning projects" src="https://user-images.githubusercontent.com/2771466/222950622-475bc6cc-7b91-47c2-86b2-5948bee4fe8e.png">

<img width="1112" alt="kondo gui displaying projects" src="https://user-images.githubusercontent.com/2771466/222950846-964162a1-80c9-4cdf-a9a8-d818ba4cb34a.png">

<details>
<summary>CLI Video</summary>

[kondo-cli.webm](https://user-images.githubusercontent.com/2771466/222949617-0ed621bc-ac4e-495a-9165-036a3a597d34.webm)

</details>

<details>
<summary>GUI Video</summary>

[kondo-ui.webm](https://user-images.githubusercontent.com/2771466/222951044-13484711-6107-45d4-aaa3-3140bbbba898.webm)

</details>

## Installation

> **Warning**
>
> Kondo is [_essentially_](https://github.com/tbillington/kondo/blob/a7af95484d364bbb12eb3b40b0d860424dd1b714/kondo-lib/src/lib.rs#L236) `rm -rf` with a prompt. Use at your own discretion. Always have a backup of your projects.

### Command Line

**Homebrew**

```sh
brew install kondo
```

**Source**

Requires [rust](https://www.rust-lang.org/tools/install).

```sh
git clone https://github.com/tbillington/kondo.git
cargo install --path kondo/kondo
```

**Others**

Binaries available on the [releases page](https://github.com/tbillington/kondo/releases).

<a href="https://repology.org/project/kondo/versions">
    <img src="https://repology.org/badge/vertical-allrepos/kondo.svg" alt="Packaging status">
</a>

### Graphic User Interface

**Source**

Requires [rust](https://www.rust-lang.org/tools/install). You may need [platform specific dependencies on linux](https://github.com/xi-editor/druid#platform-notes).

```sh
git clone https://github.com/tbillington/kondo.git
cargo install --path kondo/kondo-ui
```

Binaries available on the [releases page](https://github.com/tbillington/kondo/releases).

<a href="https://repology.org/project/rust:kondo-ui/versions">
    <img src="https://repology.org/badge/vertical-allrepos/rust:kondo-ui.svg" alt="Packaging status">
</a>

## Usage

> **Warning**
>
> Kondo is [_essentially_](https://github.com/tbillington/kondo/blob/a7af95484d364bbb12eb3b40b0d860424dd1b714/kondo-lib/src/lib.rs#L236) `rm -rf` with a prompt. Use at your own discretion. Always have a backup of your projects.

### Command Line Interface

Running `kondo` without a directory specified will run in the current directory.

```sh
kondo
```

Supplying a path will tell `kondo` where to start. Multiple paths are supported.

```sh
kondo code/my_project code/my_project_2
```

Passing a time will filter projects to those that haven't been modified for at least the specified period. See `kondo --help` for the full list of options.

```sh
kondo --older 3M # only projects with last modified greater than 3 months
kondo -o3M # shorthand
```

More options such as quiet mode, following symlinks, and filesystem restriction are viewable with `kondo --help`.

## Building/Development

To build the cli `kondo` you can run `cargo build` and `cargo run` from the projects root directory.

To build the gui `kondo-ui` you must first navigate into the `kondo-ui` directory, then you can run `cargo build` and `cargo run`.

The output binaries will be located in `target/debug/` or `target/release/` per [Cargo](https://doc.rust-lang.org/cargo/index.html) defaults.

## Similar Projects

- [The Tin Summer](https://github.com/vmchale/tin-summer)
- [Detox](https://github.com/whitfin/detox)
- [Sweep](https://github.com/woubuc/sweep)
- [npkill](https://github.com/voidcosmos/npkill)
- [Cargo Cleanall](https://github.com/LeSnake04/cargo-cleanall)
- [Cargo Sweep](https://github.com/holmgr/cargo-sweep)
- [Cargo Wipe](https://github.com/mihai-dinculescu/cargo-wipe)
- [cargo-clean-recursive](https://github.com/IgaguriMK/cargo-clean-recursive)
