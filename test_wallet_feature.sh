#/bin/bash /Users/jinghuiliao/git/neo/scripts/install-requirements.sh
# Test that wallet feature alone can be compiled without transaction feature
cargo build --no-default-features --features="wallet"
