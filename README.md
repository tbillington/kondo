# Kondo 🧹

![Kondo Lints](https://github.com/tbillington/kondo/workflows/Kondo%20Lints/badge.svg)

Cleans unneeded directories and files from your system.

![kondo gui](https://user-images.githubusercontent.com/2771466/76697113-f52b7a80-66e6-11ea-8ea1-4e1b6eb3f798.png)

![kondo command line](https://user-images.githubusercontent.com/2771466/89015432-5c765e00-d35a-11ea-8e67-193f2688d660.png)

It will identify the disk space savings you would get from deleting temporary/unnecessary files from project directories, such as `target` from Cargo projects and `node_modules` from Node projects.

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
