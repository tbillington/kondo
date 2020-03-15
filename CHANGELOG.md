# 0.3 2020-03-15

- Add basic graphic user interface 🎉 ([#19](https://github.com/tbillington/kondo/pull/19))
- Rewrite project discovery phase for a massive 97.5% runtime reduction 🎉 This includes a correctness fix, projects within the artifacts of other projects will not be listed and therefore will not be included more than once in the size total (previously they were). ([#20](https://github.com/tbillington/kondo/pull/20))
- Break project into cargo workspace ([#18](https://github.com/tbillington/kondo/pull/18))
- Improve path handling, skip folders that don't exist ([#17](https://github.com/tbillington/kondo/pull/17))

# 0.2 2020-02-21

- Added Haskell Stack project support
- Added Github actions for testing, linting, and deployment
- Added SBT project support
- Add command line options
  - Support passing multiple paths to scan
  - Support outputting just a list of artifact directories, this list is usually used to pipe into another program to delete
  - Support running a command for each artifact directory
- Add Maven project support
