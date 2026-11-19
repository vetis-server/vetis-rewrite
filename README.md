# vetis-rewrite

[![Crates.io downloads](https://img.shields.io/crates/d/vetis-rewrite)](https://crates.io/crates/vetis-rewrite) [![crates.io](https://img.shields.io/crates/v/vetis-rewrite?style=flat-square)](https://crates.io/crates/vetis-rewrite) [![Build Status](https://github.com/vetis-server/vetis-rewrite/actions/workflows/rust.yml/badge.svg?event=push)](https://github.com/vetis-server/vetis-rewrite/actions/workflows/rust.yml) ![Crates.io MSRV](https://img.shields.io/crates/msrv/vetis-rewrite) [![Documentation](https://docs.rs/vetis-rewrite/badge.svg)](https://docs.rs/vetis-rewrite/latest/vetis-rewrite) [![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/vetis-server/vetis-rewrite/blob/main/LICENSE.md)  [![codecov](https://codecov.io/gh/vetis-server/vetis-rewrite/graph/badge.svg?token=T0HSBAPVSI)](https://codecov.io/gh/vetis-server/vetis-rewrite)

vetis-rewrite adds diffent rewrite types for your host path, choose the most appropriate one for your usecase!

## Available rewrite hooks

- query
- regex
- replace
- req_headers
- res_headers
- strip_prefix
- strip_suffix

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
vetis-rewrite = "0.1.0"
```

## Configuration

Below you can find instruction on how to configure many auth options availble
on vetis.

### stdout

#### Using API

```rust,ignore
use vetis_rewriten ::basic::HookConfig as BasicAuthHookConfig;

let config = BasicAuthHookConfig::builder()
    .build()?;
```

#### vetis.yaml

```yaml
name: "regex_rewrite"
matcher:
  type: "path"
  paths:
    - "/hello/world*"
target: "/(hello*)/(world*)"
replacement: "/$2/$1"
```

#### Configuration Options

- **matcher**: If set, this hook will be only called if request meet the criteria defined by him, otherwise, all requests will call this hook (optional)

### Runtime Features

Choose the appropriate runtime feature for your project:

- `runtime-tokio` (default): Use with Tokio runtime

## License

Licensed under either of

- Apache License, Version 2.0
  (LICENSE-APACHE or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license
  (LICENSE-MIT or <https://opensource.org/licenses/MIT>)

at your option.

## Author

Rogerio Pereira Araujo <rogerio.araujo@gmail.com>
