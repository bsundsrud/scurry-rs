# scurry-rs
Standalone or embeddable forward-only SQL migration tool

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
    help        Prints this message or the help of the given subcommand(s)
    postgres    Migrate Postgres DB
    sqlite      Migrate Sqlite DB
```

