# scurry-rs
Standalone or embeddable forward-only SQL migration tool

For documentation on embedding Scurry, see the library documentation [here](http://bsundsrud.github.io/scurry-rs/scurry).

```
Scurry CLI
Forward-only migrations for Postgres and Sqlite

USAGE:
    scurry [FLAGS] [OPTIONS] [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information


OPTIONS:
    -d, --dir <MIGRATIONS>      Path to migrations.  Default: ./migrations
    -r, --revision <VERSION>    Version to migrate to.  Defaults to latest

SUBCOMMANDS:
    help         Prints this message or the help of the given subcommand(s)
    postgres     Migrate Postgres DB
    revisions    List available versions
    sqlite       Migrate Sqlite DB
```

Postgres subcommand:
```
scurry-postgres
Migrate Postgres DB

USAGE:
    scurry postgres [FLAGS] --connect <CONNECTION_STRING> [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information

    -V, --version    Prints version information

OPTIONS:
    -c, --connect <CONNECTION_STRING>    Connection string for Postgres DB

SUBCOMMANDS:
    help       Prints this message or the help of the given subcommand(s)
    history    List installed versions
    mark       Set schema version without running migrations
    migrate    Migrate schema
```

SQLite subcommand:

```
scurry-sqlite
Migrate Sqlite DB

USAGE:
    scurry sqlite [FLAGS] --path <PATH> [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information

    -V, --version    Prints version information

OPTIONS:
    -p, --path <PATH>    Path to Sqlite DB

SUBCOMMANDS:
    help       Prints this message or the help of the given subcommand(s)
    history    List installed versions
    mark       Set schema version without running migrations
    migrate    Migrate schema
```

# License

This project is dual licensed under MIT or Apache 2.0 at your option.
