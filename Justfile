check:
    @cargo +stable check --workspace --all-features --all-targets
    @cargo +stable check --workspace --no-default-features --all-targets
    @cargo +nightly check --workspace --all-features --all-targets
    @cargo +nightly check --workspace --no-default-features --all-targets

test:
    @cargo +stable test --workspace --all-features
    @cargo +stable test --workspace --no-default-features
    @cargo +nightly test --workspace --all-features
    @cargo +nightly test --workspace --no-default-features

verify: check test
