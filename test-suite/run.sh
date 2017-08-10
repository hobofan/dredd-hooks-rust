#!/bin/bash

cargo build --manifest-path ../Cargo.toml --features binaries
cargo build

# working directory is tmp/aruba
export PATH=../../../target/debug:$PATH # for dredd-hooks-rust
export PATH=../target/debug:$PATH # for test binaries

# Individual files to run
# bundle exec cucumber features/execution_order.feature
# bundle exec cucumber features/failing_transaction.feature
# bundle exec cucumber features/hook_handlers.feature
# bundle exec cucumber features/multiple_hookfiles.feature
# bundle exec cucumber features/tcp_server.feature

# Run all
bundle exec cucumber
