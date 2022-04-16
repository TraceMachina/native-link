# Turbo Cache

[![CI](https://github.com/allada/turbo-cache/actions/workflows/main.yml/badge.svg)](https://github.com/allada/turbo-cache/actions/workflows/main.yml)

An extremely fast and efficient bazel cache service (CAS) written in rust.

The goals of this project are:
1. Stability - Things should work out of the box as expected
2. Efficiency - Don't waste time on inefficiencies & low resource usage
3. Tested - Components should have plenty of tests & each bug should be regression tested
4. Customers First - Design choices should be optimized for what customers want

## Overview

Turbo Cache is a project that implements the [Bazel Remote Execution protocol](https://github.com/bazelbuild/remote-apis) (the CAS/cache portion).

When properly configured this project will provide extremely fast and efficient build cache for any systems that communicate using the [BRE protocol](https://github.com/bazelbuild/remote-apis/blob/main/build/bazel/remote/execution/v2/remote_execution.proto).

## Status

This project is still under active development, but has passed into the "Alpha" stage of development. All major components do work as expected and no major API changes are expected.

As of November 18th, 2021 the following things are in need of fixing or should be noted by any users:
* Uses nightly build of the Rust compiler version: `2021-11-01`. This is because this project is sufficiently tested and any bugs found in the rust compiler could be reported back to the community. As this project reaches maturity this will likely be phased out and we'll start using a stable version of the rust compiler.
* Although all the configurations do work from unit test and local testing perspective, they have not been tested heavily in production environments yet. Please report any bugs that are discovered.


## History

This project was first created due to frustration with similar projects not working or being extremely inefficient. Rust was chosen as the language to write it in because at the time rust was going through a revolution in the new-ish feature `async-await`. This made making multi-threading extremely simple when paired with a runtime (like [tokio](https://github.com/tokio-rs/tokio)) while still giving all the lifetime and other protections that Rust gives. This pretty much guarantees that we will never have crashes due to race conditions. This kind of project seemed perfect, since there is so much asynchronous activity happening and running them on different threads is most preferable. Other languages like `Go` are good candidates, but other similar projects rely heavily channels and mutex locks which are cumbersome and have to be carefully designed by the developer. Rust doesn't have these issues, since the compiler will always tell you when the code you are writing might introduce undefined behavior. The last major reason is because Rust is extremely fast, +/- a few percent of C++ and has no garbage collection (like C++, but unlike `Java`, `Go`, or `Typescript`).

## Build Requirements
* Linux (most recent versions) (untested on Windows, but might work)
* Bazel 3.0.0+

### Building
```
$ bazel build //cas
```

This will place an executable in `./bazel-bin/cas/cas` that will start the service.

### Configure

Configuration is done via a JSON file that is passed in as the first parameter to the `cas` program. See [here](https://github.com/allada/turbo-cache/tree/master/config) for more details and examples.

## How to update external rust deps

Install `cargo` and then run: `cargo install cargo-raze`.
From now on you can use: 
```
$ cargo generate-lockfile  # This will generate a new Cargo.lock file.
$ cargo raze  # This will code-gen the bazel rules.
```

# License

Copyright 2020-2022 Nathan (Blaise) Bruer
