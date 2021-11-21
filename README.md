# Simple Mail Reader

A simple mail reader CLI made with Rust to view unix mails (from `/var/mail` for example)

Supports Unix (because of termion dependency)

_**WIP:** Bugs may be present but they solely impact reading mails_

# Quick start

Install: `cargo install --git "https://github.com/elias123tre/simple-mail-reader"`  
Run: `simple-mail-reader` (this shows all mails in `/var/mail`)

# Manual

```man
Simple mail reader 0.1.0
A simple mail reader CLI made with Rust to view unix mails.

USAGE:
    simple-mail-reader [OPTIONS] [--] [USER]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -p, --path <path>       Path to the directory containing mail [default: /var/mail]
    -s, --skip <skip>...    Users to skip reading mail from

ARGS:
    <USER>    User to read mail from
```
