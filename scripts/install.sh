#!/usr/bin/env bash

npm run build:native && \
  cp src/rust-bf/target/release/bf /usr/local/bin && \
  cp src/ripple-asm/target/release/rasm /usr/local/bin && \
  cp src/ripple-asm/target/release/rlink /usr/local/bin && \
  cp rbt/target/release/rbt /usr/local/bin && \
  cp src/bf-macro-expander/target/release/bfm /usr/local/bin