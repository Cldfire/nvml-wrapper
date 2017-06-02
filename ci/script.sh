# This script takes care of testing your crate

set -ex

# TODO This is the "test phase", tweak it as you see fit
main() {
      cross build --target $TARGET --features "ci"
#     echo "running script"
#     if [ $RUST_VER = nightly ]; then
#             echo "nightly"
#             cross build --target $TARGET --features "nightly ci"
#             cross build --target $TARGET --release --features "nightly ci"

#             cross build --target $TARGET --features "nightly ci serde"
#             cross build --target $TARGET --release --features "nightly ci serde"

#             cross test --target $TARGET --features "test-local ci nightly" --no-run
#             cross test --target $TARGET --release --features "test-local ci nightly" --no-run

#             cross test --target $TARGET --features "nightly ci" --no-run
#             cross test --target $TARGET --release --features "nightly ci" --no-run
#     else
#             echo "stable"
#             cross build --target $TARGET --features "ci"
#             cross build --target $TARGET --features "ci" --release

#             cross build --target $TARGET --features "ci serde"
#             cross build --target $TARGET --release --features "ci serde"

#             cross test --target $TARGET --features "ci test-local" --no-run
#             cross test --target $TARGET --release --features "ci test-local" --no-run

#             cross test --target $TARGET --features "ci" --no-run
#             cross test --target $TARGET --features "ci" --release --no-run
#     fi
}
