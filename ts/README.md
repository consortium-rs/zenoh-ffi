# TypeScript Zenoh example

Start the bridge described in the repository's root README, then run `vp run start`
in this directory. This uses Node 24.20+ to run `src/demo.ts` directly. Set
`DEMO_SECONDS` to change the default 30 seconds.

Install with `vp install --frozen-lockfile`. Validate with `vp check`, `vp test`,
and `vp run build`. The existing library entry point is separate from the runnable
Zenoh demo.
