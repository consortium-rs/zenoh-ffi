import { existsSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { setTimeout } from "node:timers/promises";
import { Config, Session, type Sample } from "@eclipse-zenoh/zenoh-ts";
import protobuf from "protobufjs";

const root = await protobuf.load(fileURLToPath(new URL("../../proto/demo.proto", import.meta.url)));
const Result = root.lookupType("demo.Result");
const seconds = Number(process.env.DEMO_SECONDS ?? "30");
if (!Number.isSafeInteger(seconds) || seconds < 1) {
  throw new Error("DEMO_SECONDS must be a positive integer");
}
const session = await Session.open(new Config("ws://127.0.0.1:10000"));
try {
  const subscriber = await session.declareSubscriber("demo/example/**", {
    handler: (sample: Sample) => {
      const result = Result.toObject(Result.decode(sample.payload().toBytes()));
      console.log(
        `RECV ${sample.keyexpr().toString()} ${String(result.language)} ${String(result.operation)} ${JSON.stringify(result.values)}`,
      );
    },
  });
  console.log("READY typescript");
  const gate = process.env.DEMO_START_FILE;
  const deadline = Date.now() + 60_000;
  while (gate && !existsSync(gate)) {
    if (Date.now() > deadline) throw new Error("Startup barrier timed out");
    await setTimeout(50);
  }
  for (let i = 0; i < seconds; i++) {
    await setTimeout(1000);
    const result = Result.create({
      language: "typescript",
      sequence: i,
      operation: "node.sum",
      values: [[1, 2, 3].reduce((sum, value) => sum + value, 0)],
    });
    await session.put("demo/example/typescript", Result.encode(result).finish());
  }
  await subscriber.undeclare();
} finally {
  await session.close();
}
