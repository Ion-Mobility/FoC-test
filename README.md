How to use cross compile

1. Add target aarch64-unknown-linux-musl

   `rustup target add aarch64-unknown-linux-musl`

2. Download a musl cross-compiler

   `wget https://musl.cc/aarch64-linux-musl-cross.tgz`

3. Extract it

   `tar xvzf aarch64-linux-musl-cross.tgz`

4. Add it to your path -- this must be done every time you start a new shell

   `export PATH=$(pwd)/aarch64-linux-musl-cross/bin:$PATH`

5. Build the project

```
CFLAGS_aarch64_unknown_linux_musl=-mno-outline-atomics \
CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=aarch64-linux-musl-gcc \
cargo build \
--target aarch64-unknown-linux-musl
```
