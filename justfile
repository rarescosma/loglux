build: clean
    RUSTFLAGS="-Zlocation-detail=none -Zfmt-debug=none" cargo +nightly build \
      -Z build-std=std,panic_abort \
      -Z build-std-features="panic_immediate_abort,optimize_for_size" \
      --target x86_64-unknown-linux-gnu --release

bloat: clean
    RUSTFLAGS="-Zlocation-detail=none -Zfmt-debug=none" cargo +nightly bloat \
      -Z build-std=std,panic_abort \
      -Z build-std-features="panic_immediate_abort,optimize_for_size" \
      --target x86_64-unknown-linux-gnu --release

clean: 
    cargo clean

fmt:
    cargo +nightly fmt
