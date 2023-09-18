# Sofie: The Modern TV News Studio Automation System (ATEM connection image tools)

[![Node CI](https://github.com/julusian/atem-connection-image-tools/actions/workflows/CI.yaml/badge.svg)](https://github.com/julusian/atem-connection-image-tools/actions/workflows/CI.yaml)
[![npm](https://img.shields.io/npm/v/@atem-connection/image-tools)](https://www.npmjs.com/package/@atem-connection/image-tools)

## Usage

This library is not intended to be used directly, but as part of atem-connection [NPM](https://www.npmjs.com/package/atem-connection) [Github](https://github.com/nrkno/sofie-atem-connection). It provides some image utilities with both compiled (rust) and javascript implementations.

## Development

### Setting up

- Clone the repository
- Install a compatible version of nodejs and yarn
- Install the rust compiler with [rustup](https://rustup.rs/)
- Build the project with `yarn build`
- You can use [`yarn link`](https://yarnpkg.com/cli/link) to link this into `atem-connection`

### Modifying the rust code

To rebuild the native component you can run `yarn build:rs`. If you are changing the exposed api, you should instead run `yarn build`, so that typescript can check the new typings.

There are some rust unit tests, which can be run with `yarn unit:rs`, or you can run all the unit tests with `yarn unit` (make sure to rebuild the module first!).

### Modifying the js code

You can run `yarn build:js` to rebuild the typescript code, optionally with the `--watch` parameter to re-run upon saving a file.

There are some unit tests, which can be run with `yarn unit:js`. This command calls into jest, so any jest arguments can also be used
