# YADB

`yadb` is the unmodified release asset of [YADB](https://github.com/ysbing/YADB)
v1.1.2, the device-side helper behind the core input channel
(`src-tauri/src/workspace/input/yadb.rs`). The app embeds it, pushes it to
`/data/local/tmp` and runs it with `app_process`; nothing is installed on the
device.

- Release: <https://github.com/ysbing/YADB/releases/tag/v1.1.2>
- Download: <https://github.com/ysbing/YADB/releases/download/v1.1.2/yadb>
- SHA-256: `f1d12e49c94fd227b900b7cfcf74ca9dcff57dbf4aa3c43591ebe742306b1796`
- Source: <https://github.com/ysbing/YADB/tree/v1.1.2>
- License: GNU LGPL v3, see `LICENSE` in this directory.

To update, replace `yadb` with a newer release asset and change the version and
digest in `yadb.rs`; a unit test checks the embedded file against the digest.
