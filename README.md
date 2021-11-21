# Simple Mail Reader

A simple mail reader CLI made with Rust to view unix mails (from `/var/mail` for example)

Set environment variable `MAIL_FOLDER` to the folder where user mail files are stored (defaults to `/var/mail`)

Supports Unix (because of termion dependency)

_**WIP:** Bugs may be present but they only impact reading_
