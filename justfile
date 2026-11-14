rustc_lib_dir := `rustc --print sysroot` + "/lib/rustlib/" + `rustc -vV | sed -n 's/^host: //p'` + "/lib"

build:
    cargo build --bins

fmt:
    cargo fmt

clippy:
    cargo clippy --all-targets

test: build
    cargo test
    env LD_LIBRARY_PATH="{{rustc_lib_dir}}:${LD_LIBRARY_PATH}" ./target/debug/forge-test

bless: build
    env LD_LIBRARY_PATH="{{rustc_lib_dir}}:${LD_LIBRARY_PATH}" ./target/debug/forge-test --bless
