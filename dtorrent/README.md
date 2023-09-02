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
    - It can be installed with:
      ```bash
      curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
      ```
- [GTK3](https://gtk.org/) (for the UI)
    - It can be installed with:
      ```bash
      sudo apt-get update && sudo apt-get install -y libgtk-3-dev
      ```

## Running

To run the program there needs to be a `config.cfg` file in the root of the project. We provide one with default values as an example inside the *dtorrent* folder.

Then run the program with `cargo` followed by the directory containing the .torrent files:

```bash
cargo run --bin dtorrent ./torrents
```

On startup the client gets all the .torrent files on the specified directory and immediately starts the download & upload.

## Tests

Run tests with `cargo`:

```bash
cargo test --package dtorrent
```
