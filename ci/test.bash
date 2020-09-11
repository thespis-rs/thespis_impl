#!/usr/bin/bash

# fail fast
#
set -e

# print each command before it's executed
#
set -x

export RUSTFLAGS="-D warnings"

cargo check --all-features

# Currently doc tests in readme will fail without all features, because we have no way of turning on
# the features for the doctest.
#
cargo test --all-features

# checking with rustup for when not running on travis.
#
if [[ "$TRAVIS_RUST_VERSION" == nightly ]] || [[ $(rustup default) =~ "nightly" ]]
then

	# will run doc tests which requires nightly.
	#
	cargo doc --all-features --no-deps

fi
