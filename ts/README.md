# TypeScript Node client

Run `vp install --frozen-lockfile` and `vp run start` with Node 24.20+ and the Zenoh
remote-API bridge running. No browser or TypeScript compilation is required.
The client loads `../proto/demo.proto` with protobufjs, publishes a sum of 6,
and decodes binary results from all four languages. Run `vp check`, `vp test`,
and `vp run build` for the package checks. See the root README for setup and
Rust orchestration.
