# Celyrion

A Rust-based AI inference engine. This repository is at the scaffolding stage:
every crate below exists with its responsibility documented and a handful of
stub types, but nothing serves a model yet.

```sh
cargo run -p celyrion -- serve examples/model.yaml
```

## Layout

```
crates/               Cargo workspace (one crate per module)
examples/model.yaml   sample config for `celyrion serve`
workers/              Python / C++ / CUDA / Triton worker implementations (not built by cargo)
```

## Crates

| Crate | Responsibility |
|-------|----------------|
| `celyrion` | CLI; `celyrion serve model.yaml` |
| `celyrion-runtime` | **Core crate.** Loads `model.yaml`, wires every module, re-exports them |
| `celyrion-types` | Versioned identities, units, records, digests, errors |
| `celyrion-protocol` | Versioned messages exchanged between components |
| `celyrion-core` | State transitions, serialized ownership |
| `celyrion-cell` | Bounded mailboxes, effect dispatch |
| `celyrion-state` | Typed state bundles |
| `celyrion-memory` | Cache reuse, reservations, pins, reclamation |
| `celyrion-scheduler` | Eligibility, fair ranking, deadlines, batch construction, cost estimates |
| `celyrion-workers` | Rust side of Python/native workers: proposals, prepared variants, verified completion events |
| `celyrion-transport` | State movement |
| `celyrion-groups` | Execution-group preparation, collective coordination |
| `celyrion-gateway` | Authenticated API behaviour, replay |
| `celyrion-authority` | Declared authoritative transactions |
| `celyrion-pipeline` | Bounded stages |
| `celyrion-registry` | Immutable packages, revisions, artifact references |
| `celyrion-observe` | Bounded telemetry, generation health |
| `celyrion-supervisor` | Safe teardown |
| `celyrion-sim` | Deterministic effects, faults, trace replay using the real core |

## Development

```sh
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
```

The toolchain is pinned by `rust-toolchain.toml`; rustup installs it on first use.
Worker packages have their own instructions under [`workers/`](workers/README.md).

## License

MIT — see [LICENSE](LICENSE).

## Contributing

Formatting and lint checks run through [pre-commit](https://pre-commit.com):

```sh
uv tool install pre-commit
pre-commit install
pre-commit run --all-files
```

The same hooks run in CI on every pull request and must pass before merging to `main`.
