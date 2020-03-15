# Kondo 🧹

![Kondo Tests](https://github.com/tbillington/kondo/workflows/Kondo%20Tests/badge.svg) ![Kondo Lints](https://github.com/tbillington/kondo/workflows/Kondo%20Lints/badge.svg)

Cleans unneeded directories and files from your system.

It will identify the disk space savings you would get from deleting temporary/unnecessary files from project directories, such as `target` from Cargo projects and `node_modules` from Node projects. Currently `kondo` doesn't actually delete any files.

Supports:

- [Cargo](https://doc.rust-lang.org/cargo/) projects
- [Node](https://nodejs.org/) projects
- [Unity](https://unity.com/) Projects
- [SBT](https://www.scala-sbt.org/) projects
- [Haskell Stack](https://docs.haskellstack.org/) projects
- [Maven](https://maven.apache.org/) projects

## Installation

You can install `kondo` via Cargo with `cargo install kondo`.

If you don't have Cargo installed you can download a binary for your platform from the [Releases](https://github.com/tbillington/kondo/releases) page.

## Screenshot

![image](https://user-images.githubusercontent.com/2771466/75639044-00ac8980-5c83-11ea-9cf8-5e30e0b933b0.png)

**GUI**

![image](https://user-images.githubusercontent.com/2771466/76200757-45698f00-6246-11ea-81a2-48baf5b84ba3.png)

## Roadmap

- Actually delete (with prompt)
- Handle Unity cache, editor cache

## Operation

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

## Similar Projects

- [The Tin Summer](https://github.com/vmchale/tin-summer)
