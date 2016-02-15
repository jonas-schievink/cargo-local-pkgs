# Run cargo subcommands on all local packages

A tiny cargo subcommand for executing other subcommands on all "local" packages (ie. packages in the same repository). This allows running `cargo test` on all crates in a repo with a single command.

## Example

This will run tests of all local crates (but see the notes below):
```
cargo local-pkgs test
```

## Notes

* The main package must have a dependency on all other crates in the repo (local crates not in the dependency graph of the main crate are skipped)
* A `Cargo.lock` must exist (you can either check it in, or run `cargo build` or `cargo update` before using this subcommand)
* You can use this to invoke external subcommands, but they must support specifying a package via `-p <pkg>`
