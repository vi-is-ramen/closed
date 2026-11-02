check: # full feature matrix for clippy
    # CHECK: ALL
    @cargo +stable  clippy --workspace --all-features --all-targets
    @cargo +nightly clippy --workspace --all-features --all-targets
    # CHECK: MACROS
    @cargo +stable  clippy --workspace --no-default-features --all-targets --features macros
    @cargo +nightly clippy --workspace --no-default-features --all-targets --features macros
    # CHECK: MACROS + STD
    @cargo +stable  clippy --workspace --no-default-features --all-targets --features std,macros
    @cargo +nightly clippy --workspace --no-default-features --all-targets --features std,macros
    # CHECK: STD + UNSTABLE
    @cargo +stable  clippy --workspace --no-default-features               --features std,unstable
    @cargo +nightly clippy --workspace --no-default-features               --features std,unstable
    # CHECK: MACROS + UNSTABLE
    @cargo +stable  clippy --workspace --no-default-features --all-targets --features macros,unstable
    @cargo +nightly clippy --workspace --no-default-features --all-targets --features macros,unstable
    # CHECK: UNSTABLE
    @cargo +stable  clippy --workspace --no-default-features               --features unstable
    @cargo +nightly clippy --workspace --no-default-features               --features unstable
    # CHECK: STD
    @cargo +stable  clippy --workspace --no-default-features               --features std
    @cargo +nightly clippy --workspace --no-default-features               --features std
    # CHECK: STD + MACROS + UNSTABLE
    @cargo +stable  clippy --workspace --no-default-features --all-targets --features std,macros,unstable
    @cargo +nightly clippy --workspace --no-default-features --all-targets --features std,macros,unstable

test: # full feature matrix for tests. tests explicitly depend on `macros` feature!
    # TEST: ALL
    @cargo +stable    test --workspace --all-features --all-targets
    @cargo +nightly   test --workspace --all-features --all-targets
    # TEST: MACROS
    @cargo +stable    test --workspace --no-default-features --all-targets --features macros
    @cargo +nightly   test --workspace --no-default-features --all-targets --features macros
    # TEST: MACROS + STD
    @cargo +stable    test --workspace --no-default-features --all-targets --features macros,std
    @cargo +nightly   test --workspace --no-default-features --all-targets --features macros,std
    # TEST: MACROS + STD + UNSTABLE
    @cargo +stable    test --workspace --no-default-features --all-targets --features macros,std,unstable
    @cargo +nightly   test --workspace --no-default-features --all-targets --features macros,std,unstable
    # TEST: MACROS + UNSTABLE
    @cargo +stable    test --workspace --no-default-features --all-targets --features macros,unstable
    @cargo +nightly   test --workspace --no-default-features --all-targets --features macros,unstable

dry-pub:
	# important: dry run must publish both packages in one invocation.
	@cargo publish --allow-dirty --dry-run -p closed-proc-macros -p closed

pub:
	# important: proc macro crate must be published before main crate, otherwise
	# publication may fail!
	@cargo publish -p closed-proc-macros
	@cargo publish -p closed

expand:
	@cargo expand -p closed > expand.rs

verify: check test dry-pub
