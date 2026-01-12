# Cross-Compilation Setup

## On Development Machine

```bash
# Add target
rustup target add aarch64-unknown-linux-gnu

# Option A: cross (Docker-based)
cargo install cross
cross build --target aarch64-unknown-linux-gnu --release

# Option B: cargo-zigbuild (faster, no Docker)
cargo install cargo-zigbuild
cargo zigbuild --target aarch64-unknown-linux-gnu --release
```

## Comparison

| Method         | Pros                          | Cons                    |
| -------------- | ----------------------------- | ----------------------- |
| cross          | Handles all dependencies      | Requires Docker         |
| cargo-zigbuild | Faster, no Docker needed      | May need manual setup   |
