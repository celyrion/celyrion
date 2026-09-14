# celyrion-workers (Python)

Python-side worker implementations for Celyrion: model adapters and Triton
kernels. Mirrors the contract in `crates/celyrion-workers`.

```sh
uv sync
uv run python -c "import celyrion_workers; print(celyrion_workers.__version__)"
```
