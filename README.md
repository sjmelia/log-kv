log_kv
======

[![Build Status](https://travis-ci.org/sjmelia/log-kv.svg)](https://travis-ci.org/sjmelia/log-kv)
[![crates.io](https://img.shields.io/crates/v/log_kv.svg)](https://crates.io/crates/log_kv)

A `LogKv` backs a standard Rust `HashMap` with an log of inserts. The log is rebuilt
on initialisation by iterating over it. The log is serialised using `serde`.

This makes for a cheap and cheerful persistent key-value store. It is similar in
principal to a [bitcask](https://github.com/basho/bitcask) albeit without the
merging and hint files.

Usage
-----

See the examples in the doctests of `src/lib.rs`.
