cargo_artefact_name := "libobsidian_sqlite_vtab"
artefact_ext := if os() == "macos" { ".dylib" } else if os() == "linux" { ".so" } else { ".dll" }
artefact_name := "obsidian_vtab" + artefact_ext

# The default installation of sqlite on mac doesn't
# allow laoding extensions. Override this with a version
# that does (e.g. homebrew with `--set sqlite_bin /opt/homebrew/opt/sqlite/bin/sqlite3`)
sqlite_bin := "sqlite3"

_default:
  @just --listf


build target="test":
  cargo build {{if target == "release" { "--release" } else { "" } }}

clean_dist:
  rm -rf dist
  mkdir dist

clean_targets:
  rm -f target/release/{{cargo_artefact_name}}.*
  rm -f target/debug/{{cargo_artefact_name}}.*

clean: clean_dist clean_targets

[macos]
test_dist: build clean_dist
  cp target/debug/{{cargo_artefact_name}}.dylib dist/obsidian_vtab

[linux]
test_dist: build clean_dist
  cp target/debug/{{cargo_artefact_name}}.so dist/obsidian_vtab

# Runs cargo test building the artefact first so it can be loaded by integration tests.
test: test_dist
  cargo test --features rlib

sql dirname="test_data": build
  echo "select obsidian_version(); \
  CREATE VIRTUAL TABLE vt \
  USING obsidian_notes(dirname='{{dirname}}'); \
  select * from vt;" \
  | {{sqlite_bin}} -box -cmd ".load target/debug/{{cargo_artefact_name}}"
