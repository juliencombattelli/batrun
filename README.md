# batrun 🦇

Bash Test Runner, a framework to write and execute tests in bash.

## Features

## Installing

## Usage

## Writing tests

## Building

Batrun is written in Rust. You will need a working `Rust` and `Cargo` setup.
[Rustup](https://rustup.rs/) is the simplest way to set this up on either Windows, Mac or Linux.

To build Batrun:

```bash
git clone https://github.com/juliencombattelli/batrun
cd batrun
cargo build --release
./target/release/batrun --version
```

## Running the internal validation test suite

The internal test suite in tests/ is used to validate the behaviour of batrun.
To run it execute the following command:
```bash
batrun ./tests --out-dir out --target test other
```
