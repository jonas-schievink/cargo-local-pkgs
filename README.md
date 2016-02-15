# Run cargo subcommands on all local packages

A tiny cargo subcommand for executing other subcommands on all "local" packages (ie. packages in the same repository). This allows running `cargo test` on all crates in a repo with a single command.

## Installation

As usual, this subcommand can be installed with `cargo install`:
```
cargo install cargo-local-pkgs
```

## Examples

### Test all local crates

This will run tests of all local crates (but see the notes below):
```
cargo local-pkgs test
```

### With Travis

Travis integration is easy, thanks to `cargo install`:
```yml
language: rust
before_script:
  - |
      cargo install cargo-local-pkgs --vers 0.1 &&
      export PATH=$HOME/.cargo/bin:$PATH
script:
  - cargo local-pkgs test
```

Libraries aren't supposed to check their `Cargo.lock` into git, so it doesn't exist when running `cargo local-pkgs` via Travis. However, we can generate it using `cargo generate-lockfile`. Replace the `script` section above with this:
```yml
script:
  - |
      cargo generate-lockfile &&
      cargo local-pkgs test
```

## Notes

* The main package must have a dependency on all other crates in the repo (local crates not in the dependency graph of the main crate are skipped)
* A `Cargo.lock` must exist (you can either check it in, or run `cargo build` or `cargo update` before using this subcommand)
* You can use this to invoke external subcommands, but they must support specifying a package via `-p <pkg>`
