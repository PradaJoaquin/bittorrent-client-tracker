<p align="center">
<img src="../docs/logo_dtorrent.png" height="150">
</p>

<h1 align="center">
dTorrent
</h1>
<p align="center">
A BitTorrent client made in Rust.
<p>

## Requirements

To build the program it needs:

- [Rust](https://www.rust-lang.org/) (and cargo)
- [gtk3](https://gtk.org/) (for the UI)

## Running

To run the program there needs to be a `config.cfg` file in the root of the project. We provide one with default values as an example.

Then run the program with `cargo` followed by the directory containing the .torrent files:

```bash
$ cargo run --bin dtorrent ./torrents
```

On startup the client gets all the .torrent files on the specified directory and immediately starts the download & upload.

## Tests

Run tests with `cargo`:

```bash
$ cargo test --package dtorrent
```
