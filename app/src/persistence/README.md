# SQLite migrations

Warper applies pending SQLite migrations in a transaction at startup. Migration SQL lives under `crates/persistence/migrations/`; generated Rust bindings live in `crates/persistence/src/schema.rs`.

## Setup

`./script/bootstrap` installs Diesel CLI 2.3.8, the version used by CI. To install it directly:

```sh
cargo binstall --force -y diesel_cli@2.3.8
```

Keep the CLI pins in `.github/workflows/ci.yml` and `script/install_cargo_build_deps` in sync. CLI version changes can change generated Rust bindings even when the database schema is unchanged.

## Add a migration

Run from the repository root:

```sh
diesel migration generate migration_name --migration-dir crates/persistence/migrations
```

Edit the generated `up.sql` and `down.sql` files.

## Generate and review the schema

Use a disposable database for development:

```sh
migration_test_dir="$(mktemp -d)"
migration_test_db="$migration_test_dir/warper.sqlite"
diesel migration run --database-url "$migration_test_db" --migration-dir crates/persistence/migrations
git diff -- crates/persistence/src/schema.rs
```

Running migrations regenerates the bindings using `diesel.toml`. Commit the generated schema with the migration. CI replays every migration into a fresh database and compares the generated bindings with the checked-in file.

Test reversion and reapplication against the same disposable database:

```sh
diesel migration revert --database-url "$migration_test_db" --migration-dir crates/persistence/migrations
diesel migration run --database-url "$migration_test_db" --migration-dir crates/persistence/migrations
```

## Schema style

- Use `id` for integer primary keys.
- Use plural table names and singular Rust model names.
- Name foreign-key columns after the referenced model, for example `foo_id` for `foos.id`.
