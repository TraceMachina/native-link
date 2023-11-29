RUST_EDITION = "2021"

PACKAGES = {
    "prost": {
        "version": "0.11.9",
    },
    "prost-types": {
        "version": "0.11.9",
    },
    "hex": {
        "version": "0.4.3",
    },
    "async-trait": {
        "version": "0.1.71",
    },
    "fixed-buffer": {
        "version": "0.2.3",
    },
    "futures": {
        "version": "0.3.28",
    },
    "tokio": {
        "version": "1.29.1",
        "features": ["macros", "io-util", "fs", "rt-multi-thread", "parking_lot"],
    },
    "tokio-stream": {
        "version": "0.1.14",
        "features": ["fs", "sync"],
    },
    "tokio-util": {
        "version": "0.7.8",
        "features": ["io", "io-util", "codec"],
    },
    "tonic": {
        "version": "0.9.2",
        "features": ["gzip"],
    },
    "tls-listener": {
        "version": "0.7.0",
        "features": ["hyper-h2", "rustls"],
    },
    "tokio-rustls": {
        "version": "0.24.1",
    },
    "rustls-pemfile": {
        "version": "1.0.3",
    },
    "rcgen": {
        "version": "0.11.3",
    },
    "log": {
        "version": "0.4.19",
    },
    "env_logger": {
        "version": "0.10.0",
    },
    "serde": {
        "version": "1.0.167",
    },
    "json5": {
        "version": "0.4.1",
    },
    "sha2": {
        "version": "0.10.7",
    },
    "lru": {
        "version": "0.10.1",
    },
    "rand": {
        "version": "0.8.5",
    },
    "http": {
        "version": "^0.2",
    },
    "pin-project-lite": {
        "version": "0.2.10",
    },
    "async-lock": {
        "version": "2.7.0",
    },
    "lz4_flex": {
        "version": "0.11.1",
    },
    "bincode": {
        "version": "1.3.3",
    },
    "bytes": {
        "version": "1.4.0",
    },
    "shellexpand": {
        "version": "3.1.0",
    },
    "byteorder": {
        "version": "1.4.3",
    },
    "filetime": {
        "version": "0.2.21",
    },
    "nix": {
        "version": "0.26.2",
    },
    "clap": {
        "version": "4.3.11",
        "features": ["derive"],
    },
    "uuid": {
        "version": "1.4.0",
        "features": ["v4"],
    },
    "shlex": {
        "version": "1.1.0",
    },
    "relative-path": {
        "version": "1.8.0",
    },
    "parking_lot": {
        "version": "0.12.1",
    },
    "hashbrown": {
        "version": "0.14",
    },
    "hyper": {
        "version": "0.14.27",
        "features": ["tcp", "client"],
    },
    "hyper-rustls": {
        "version": "0.24.2",
        "features": ["webpki-tokio"],
    },
    "axum": {
        "version": "0.6.18",
    },
    "tower": {
        "version": "0.4.13",
    },
    "prometheus-client": {
        "version": "0.21.2",
    },
    "blake3": {
        "version": "1.4.1",
    },
    "scopeguard": {
        "version": "1.2.0",
    },
    "stdext": {
        "version": "0.3.1",
    },
    "prost-build": {
        "version": "0.11.9",
    },
    "tonic-build": {
        "version": "0.9.2",
    },
    "pretty_assertions": {
        "version": "1.4.0",
    },
    "maplit": {
        "version": "1.0.2",
    },
    "mock_instant": {
        "version": "0.3.1",
    },
    "ctor": {
        "version": "0.2.3",
    },
    "memory-stats": {
        "version": "1.1.0",
    },
    "formatx": {
        "version": "0.2.1",
    },
    "once_cell": {
        "version": "1.18.0",
    },
    "aws-config": {
        "version": "0.57.1",
    },
    "aws-sdk-config": {
        "version": "0.35.0",
    },
    "aws-sdk-s3": {
        "version": "0.35.0",
        # TODO(aaronmondal): Make "test-util" a dev-dependency feature.
        "features": ["rt-tokio", "test-util"],
    },
    "aws-smithy-runtime": {
        "version": "0.57.1",
        "features": ["client", "connector-hyper-0-14-x"],
    },
    "aws-smithy-types": {
        "version": "0.57.1",
    },
}
