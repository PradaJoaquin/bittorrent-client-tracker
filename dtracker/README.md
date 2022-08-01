<p align="center">
<img src="../docs/logo_dtracker.png" height="150">
</p>

<h1 align="center">
dTracker
</h1>
<p align="center">
A BitTorrent tracker made in Rust.
<p>

## Requirements

To build the program it needs:

- [Rust](https://www.rust-lang.org/) (and cargo)

## Running

Run the program with `cargo` followed by the port on which to run the tracker:

```bash
$ cargo run --bin dtracker 8080
```

## Tests

Run tests with `cargo`:

```bash
$ cargo test --package dtracker
```
