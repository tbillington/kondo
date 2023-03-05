# Kondo 🧹

Cleans unneeded directories and files from your system.

<p align="center">
<a href="https://doc.rust-lang.org/cargo/">Cargo</a>
- <a href="https://nodejs.org/">Node</a>
- <a href="https://unity.com/">Unity</a>
- <a href="https://www.scala-sbt.org/">SBT</a>
- <a href="https://docs.haskellstack.org/">Haskell Stack</a>
- <a href="https://maven.apache.org/">Maven</a>
- <a href="https://www.unrealengine.com/">Unreal Engine</a>
- <a href="https://jupyter.org/">Jupyter Notebook</a>
- <a href="https://www.python.org/">Python</a>
- <a href="https://cmake.org">CMake</a>
- <a href="https://getcomposer.org/">Composer</a>
- <a href="https://dart.dev/">Pub</a>
- <a href="https://elixir-lang.org/">Elixir</a>
- <a href="https://swift.org/">Swift</a>
</p>

**Command line interface**

<img width="972" alt="kondo cli cleaning projects" src="https://user-images.githubusercontent.com/2771466/222950622-475bc6cc-7b91-47c2-86b2-5948bee4fe8e.png">

**Graphic user interface**

<img width="1112" alt="kondo gui displaying projects" src="https://user-images.githubusercontent.com/2771466/222950846-964162a1-80c9-4cdf-a9a8-d818ba4cb34a.png">

<details>
<summary>CLI Video</summary>

[kondo-cli.webm](https://user-images.githubusercontent.com/2771466/222949617-0ed621bc-ac4e-495a-9165-036a3a597d34.webm)

</details>

<details>
<summary>UI Video</summary>

[kondo-ui.webm](https://user-images.githubusercontent.com/2771466/222951044-13484711-6107-45d4-aaa3-3140bbbba898.webm)

</details>

## Supports:

- [Cargo](https://doc.rust-lang.org/cargo/) projects (Rust)
- [Node](https://nodejs.org/) projects (JavaScript)
- [Unity](https://unity.com/) projects (C#)
- [SBT](https://www.scala-sbt.org/) projects (Scala)
- [Haskell Stack](https://docs.haskellstack.org/) projects (Haskell)
- [Maven](https://maven.apache.org/) projects (Java)
- [Unreal Engine](https://www.unrealengine.com/) projects (C++)
- [Jupyter Notebook](https://jupyter.org/) projects (Python)
- [Python](https://www.python.org/) projects
- [CMake](https://cmake.org) projects
- [Composer](https://getcomposer.org/) projects (PHP)
- [Pub](https://dart.dev/) projects (Dart)
- [Elixir](https://elixir-lang.org/) projects
- [Swift](https://swift.org/) projects

## Installation

### Graphic User Interface

<a href="https://repology.org/project/rust:kondo-ui/versions">
    <img src="https://repology.org/badge/vertical-allrepos/rust:kondo-ui.svg" alt="Packaging status">
</a>

Windows and Mac builds are available on the [Releases](https://github.com/tbillington/kondo/releases) page as `kondo-ui`.

You can install `kondo-ui` via [Cargo](https://doc.rust-lang.org/cargo/) with `cargo install kondo-ui`. Note you'll still need [druid's platform specific dependencies](https://github.com/xi-editor/druid#platform-notes) on mac and linux.

### Command line

<a href="https://repology.org/project/kondo/versions">
    <img src="https://repology.org/badge/vertical-allrepos/kondo.svg" alt="Packaging status">
</a>

Windows, Mac, and Linux builds are available on the [Releases](https://github.com/tbillington/kondo/releases) page as `kondo`.

You can install `kondo` via [homebrew](https://formulae.brew.sh/formula/kondo) with `brew install kondo`.

## Operation

### Graphic User Interface

Launch `kondo-ui`, select a directory to be scanned, evaluate & clean directories as needed.

### Command Line Interface

Running `kondo` without a directory specified will run in the current directory.

```
$ kondo
```

Supplying a path will tell `kondo` where to start. Multiple paths are supported.

```
$ kondo code/my_project code/my_project_2
```

## Example Output

```
$ kondo ~/code
/Users/choc/code/unity Cargo project
  └─ target (489.1KiB)
  delete above artifact directories? ([y]es, [n]o, [a]ll, [q]uit): y
  deleted 489.1KiB

/Users/choc/code/multiplayer-kit/generator Cargo project
  └─ target (874.3KiB)
  delete above artifact directories? ([y]es, [n]o, [a]ll, [q]uit): n

/Users/choc/code/chat Cargo project
  └─ target (37.2MiB)
  delete above artifact directories? ([y]es, [n]o, [a]ll, [q]uit): q

Total bytes deleted: 489.1KiB
```

## Building/Development

To build `kondo` you can run `cargo build` from the projects root directory.

To build `kondo-ui` you must first navigate into the `kondo-ui` directory, then you can run `cargo build`.

## Similar Projects

- [The Tin Summer](https://github.com/vmchale/tin-summer)
- [Detox](https://github.com/whitfin/detox)
- [Sweep](https://github.com/woubuc/sweep)
- [npkill](https://github.com/voidcosmos/npkill)
