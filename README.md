# Kondo 🧹

![Kondo Tests](https://github.com/tbillington/kondo/workflows/Kondo%20Tests/badge.svg) ![Kondo Lints](https://github.com/tbillington/kondo/workflows/Kondo%20Lints/badge.svg)

Cleans unneeded directories and files from your system.

![image](https://user-images.githubusercontent.com/2771466/76697113-f52b7a80-66e6-11ea-8ea1-4e1b6eb3f798.png)

It will identify the disk space savings you would get from deleting temporary/unnecessary files from project directories, such as `target` from Cargo projects and `node_modules` from Node projects.

Supports:

- [Cargo](https://doc.rust-lang.org/cargo/) projects
- [Node](https://nodejs.org/) projects
- [Unity](https://unity.com/) Projects
- [SBT](https://www.scala-sbt.org/) projects
- [Haskell Stack](https://docs.haskellstack.org/) projects
- [Maven](https://maven.apache.org/) projects
- [Unreal Engine](https://www.unrealengine.com/) projects

## Installation

### Graphic User Interface

Windows and Mac builds are available on the [Releases](https://github.com/tbillington/kondo/releases) page as `kondo-ui`.

You can install `kondo-ui` via [Cargo](https://doc.rust-lang.org/cargo/) with `cargo install kondo-ui`. Note you'll still need [druid's platform specific dependencies](https://github.com/xi-editor/druid#platform-notes) on mac and linux.

### Command line

Windows, Mac, and Linux builds are available on the [Releases](https://github.com/tbillington/kondo/releases) page as `kondo`.

You can install `kondo` via [Cargo](https://doc.rust-lang.org/cargo/) with `cargo install kondo`.

## Operation

### Graphic User Interface

Launch `kondo-ui`, select a directory to be scanned, evaluate & clean directories as needed.

### Command Line Interface

Running `kondo` without a directory specified will run in the current directory.

```
$ kondo
```

Supplying an argument will tell `kondo` where to start.

```
$ kondo code/my_project
```

## Example Output

```
$ kondo ~
Scanning "C:/Users/Trent"
3 projects found
Calculating savings per project
  (redacted 1000~ lines)
  385.6MB UnityTestApp (Unity) C:\Users\Trent\code\UnityTestApp
  458.7MB tokio (Cargo) C:\Users\Trent\code\tokio
    1.5GB ui-testing (Node) C:\Users\Trent\code\ui-testing
    4.0GB rust-analyzer (Cargo) C:\Users\Trent\code\rust-analyzer
9.5GB possible savings
```

## Options/Flags

### Artifact Dirs

`kondo -a` will output a line-separated list of artifact directories you can delete to reclaim space.

```
$ kondo test_dir -a
C:\Users\Trent\code\kondo\test_dir\node_project\node_modules
C:\Users\Trent\code\kondo\test_dir\rust_project\target
C:\Users\Trent\code\kondo\test_dir\health-dots\Temp
C:\Users\Trent\code\kondo\test_dir\health-dots\Obj
C:\Users\Trent\code\kondo\test_dir\health-dots\MemoryCaptures
C:\Users\Trent\code\kondo\test_dir\health-dots\Build
```

### Command

`kondo -c <COMMAND>` will run your supplied command for each artifact directory.

```
$ kondo test_dir -c echo
C:\Users\Trent\code\kondo\test_dir\node_project\node_modules
C:\Users\Trent\code\kondo\test_dir\rust_project\target
C:\Users\Trent\code\kondo\test_dir\health-dots\Temp
C:\Users\Trent\code\kondo\test_dir\health-dots\Obj
C:\Users\Trent\code\kondo\test_dir\health-dots\MemoryCaptures
C:\Users\Trent\code\kondo\test_dir\health-dots\Build
```

## Building/Development

To build `kondo` you can run `cargo build` from the projects root directory.

To build `kondo-ui` you must first navigate into the `kondo-ui` directory, then you can run `cargo build`. Because we use [druid](https://github.com/xi-editor/druid) for the interface you'll need to satisfy druid's [platform specific dependencies](https://github.com/xi-editor/druid#platform-notes).

## Similar Projects

- [The Tin Summer](https://github.com/vmchale/tin-summer)
