# 0.8 2023-12-19

- Add .NET (C# and F#) support by @mrivnak in https://github.com/tbillington/kondo/pull/106
- Add Godot 4 support by @anosatsuk124 in https://github.com/tbillington/kondo/commit/764a00db7a257cdeade77ac3d5be66e80eadcd43
- Add support for elixir language server artifact directories by @aschiavon91 in https://github.com/tbillington/kondo/pull/112
- Add shell completion generation by @NickyMeuleman in https://github.com/tbillington/kondo/pull/100
- Add CLion support for CMake by @mrivnak in https://github.com/tbillington/kondo/pull/107
- Add .tox support to python projects by @jwodder in https://github.com/tbillington/kondo/pull/104
- Add support for cleaning xwin-cache in cargo projects by @tbillington in https://github.com/tbillington/kondo/commit/936e2a6844462e11c1038cfc4725f8fa7e879f44
- Fix date formatting for years, more accurate day duration by @tbillington

# 0.7 2023-07-06

- Add support for Gradle projects by @s-aditya-k and @Lipen in https://github.com/tbillington/kondo/pull/81, https://github.com/tbillington/kondo/pull/85
- Upgrade from structopt to clap by @s-aditya-k in @https://github.com/tbillington/kondo/pull/82
- Support default clean operation by @s-aditya-k in https://github.com/tbillington/kondo/pull/83
- Add Arch Linux install instructions by @orhun in https://github.com/tbillington/kondo/pull/84
- Support more types of Python artifact dirs by @trag1c in https://github.com/tbillington/kondo/pull/88
- Support ignoring specified dirs by @tbillington in https://github.com/tbillington/kondo/pull/90
- Add support for Zig projects by @orhun in https://github.com/tbillington/kondo/pull/92

# 0.6 2022-12-25

- add support for cmake projects by @sassman in https://github.com/tbillington/kondo/pull/56
- add support for composer php projects by @WesleyKlop in https://github.com/tbillington/kondo/pull/58
- add support for Pub (Dart) projects by @Desdaemon in https://github.com/tbillington/kondo/pull/62
- default to not following symlinks, thank you @cuviper for raising in https://github.com/tbillington/kondo/issues/61, by @tbillington in https://github.com/tbillington/kondo/pull/63
- show last modified date on project in https://github.com/tbillington/kondo/pull/63
- invalid directories are now filtered out when supplied to the cli in https://github.com/tbillington/kondo/commit/725f7ec72ff95a32b9f09ce834ab917c892915aa
- allow passing time filter to only show projects not mofidied in some time range by @gabrielztk in https://github.com/tbillington/kondo/pull/66
- add support for Elixir projects by @aschiavon91 in https://github.com/tbillington/kondo/pull/69

# 0.5 2022-01-05

### Major

Support for pycache and jupyter-notebook checkpoints by @Stunkymonkey in https://github.com/tbillington/kondo/pull/33

Support for "quiet" and "all" modes allowing you to clean all projects found and doing it without any noise! Implemented in https://github.com/tbillington/kondo/pull/53 and thanks to @danieljb for the suggestion.

### Changes

* update various dependencies by @striezel in https://github.com/tbillington/kondo/pull/40
* add keywords for kondo and kondo-ui by @striezel in https://github.com/tbillington/kondo/pull/41
* Add directories to delete for Python by @pawamoy in https://github.com/tbillington/kondo/pull/47
* Disable the console on the Windows platform by @Aursen in https://github.com/tbillington/kondo/pull/49
* kondo-lib: don't panic in `path_canonicalise` by @vrmiguel in https://github.com/tbillington/kondo/pull/50
* add basic error handling to scan by @tbillington in https://github.com/tbillington/kondo/pull/54

# 0.4 2020-07-31

- Remove all options and subcommands to re-focus the intent of Kondo 🧹
- Default is now interactive mode, allowing users to choose options on a per-project basis
- Update `druid` dependency to 0.6, this means we no longer rely on `cairo` on MacOS 🎉

# 0.3 2020-03-15

- Add basic graphic user interface 🎉 ([#19](https://github.com/tbillington/kondo/pull/19))
- Rewrite project discovery phase for a massive 97.5% runtime reduction 🎉 This includes a correctness fix, projects within the artifacts of other projects will not be listed and therefore will not be included more than once in the size total (previously they were). ([#20](https://github.com/tbillington/kondo/pull/20))
- Break project into cargo workspace ([#18](https://github.com/tbillington/kondo/pull/18))
- Improve path handling, skip folders that don't exist ([#17](https://github.com/tbillington/kondo/pull/17))
- Add Unreal 4 project support ([#597efd9](https://github.com/tbillington/kondo/commit/597efd9a9100272f408ebd1f531113ea11da3192))

# 0.2 2020-02-21

- Added Haskell Stack project support
- Added Github actions for testing, linting, and deployment
- Added SBT project support
- Add command line options
  - Support passing multiple paths to scan
  - Support outputting just a list of artifact directories, this list is usually used to pipe into another program to delete
  - Support running a command for each artifact directory
- Add Maven project support
