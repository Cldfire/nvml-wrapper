# This script takes care of testing your crate

set -ex

# TODO This is the "test phase", tweak it as you see fit
main() {
    cross build --target $TARGET
    cross build --target $TARGET --release
    cross build --target $TARGET --features serde
    cross build --target $TARGET --release --features serde

    cross test --target $TARGET --no-run
    cross test --target $TARGET --release --no-run

    cross build --target $TARGET --examples
    cross build --target $TARGET --examples --release
}

# we don't run the "test phase" when doing deploys
if [ -z $TRAVIS_TAG ]; then
    main
fi
