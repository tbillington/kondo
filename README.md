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

## Similar Projects

- [The Tin Summer](https://github.com/vmchale/tin-summer)
