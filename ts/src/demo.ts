import { setTimeout } from "node:timers/promises";
import { Config, Session, type Sample } from "@eclipse-zenoh/zenoh-ts";

const session = await Session.open(new Config("ws://127.0.0.1:10000"));
try {
  const subscriber = await session.declareSubscriber("demo/example/**", {
    handler: (sample: Sample) => {
      console.log(`RECV ${sample.keyexpr().toString()} ${sample.payload().toString()}`);
    },
  });
  console.log("READY typescript");
  const seconds = Number(process.env.DEMO_SECONDS ?? "30");
  for (let i = 0; i < seconds; i++) {
    await setTimeout(1000);
    await session.put("demo/example/typescript", `Hello from TypeScript! ${i}`);
  }
  await subscriber.undeclare();
} finally {
  await session.close();
}
