# Regius Mark

Regius Mark is a cryptocurrency that is backed by physical gold assets. A single
token is backed by one physical gram of gold. Blockchain technology is used to
provide an immutable and cryptographically verified ledger. The system is
centralized allowing for global scalability that would otherwise be foregone in
a decentralized system.

[Website](https://regiusmark.io) |
[Whitepaper](https://regiusmark.io/whitepaper)

## Overview

This repository provides Regius Mark's core software and library implementations
and is to be used as a point of reference when developing software in other
languages.

[![Build Status](https://travis-ci.com/RegiusMark/regiusmark.svg?branch=master)](https://travis-ci.com/RegiusMark/regiusmark)

## Supported Rust Versions

Regius Mark is built against the latest stable version of the compiler. Any
previous versions are not guaranteed to compile.

## Project Layout

Each crate lives under the `crates` directory. Developers looking to use Regius
Mark in their software will want the library under `crates/regiusmark`.

- `crates/cli`: Provides a CLI for the wallet and other utilities.
- `crates/regiusmark`: Core Regius Mark library.
- `crates/server`: Core Regius Mark server daemon.
