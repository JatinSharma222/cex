import Redis from "ioredis";
import { randomUUID } from "crypto";

const redis = new Redis("redis://localhost:6379");

const INCOMING_QUEUE = "backend-to-engine-broker";
const PAIRS = ["BTC_USDT", "ETH_USDT", "SOL_USDT"];
const ORDERS_PER_PAIR = 10_000;

function randomBetween(min: number, max: number): number {
  return Math.random() * (max - min) + min;
}

function generateOrder(symbol: string) {
  const side = Math.random() > 0.5 ? "buy" : "sell";
  const orderType = Math.random() > 0.3 ? "limit" : "market";

  const basePrice: Record<string, number> = {
    BTC_USDT: 50000,
    ETH_USDT: 3000,
    SOL_USDT: 100,
  };

  const base = basePrice[symbol];
  const price =
    orderType === "market"
      ? 0
      : parseFloat((base + randomBetween(-500, 500)).toFixed(2));

  return {
    correlationId: randomUUID(),
    responseQueue: `response-queue-loadtest`,
    type: "place_order",
    payload: {
      userId: `user_${Math.floor(Math.random() * 100)}`,
      symbol,
      side,
      orderType,
      price,
      quantity: parseFloat(randomBetween(0.01, 5).toFixed(4)),
    },
  };
}

async function runLoadTest() {
  console.log("Starting load test...\n");

  const totalOrders = PAIRS.length * ORDERS_PER_PAIR;
  let sent = 0;
  let received = 0;
  let errors = 0;

  const latencies: number[] = [];
  const timestamps = new Map<string, number>();

  const responseRedis = new Redis("redis://localhost:6379");
  const responseQueue = "response-queue-loadtest";
  let listening = true;

  const responseListener = async () => {
    while (listening) {
      const result = await responseRedis.brpop(responseQueue, 1);
      if (!result) continue;

      try {
        const response = JSON.parse(result[1]);
        const sentAt = timestamps.get(response.correlationId);
        if (sentAt) {
          latencies.push(Date.now() - sentAt);
          timestamps.delete(response.correlationId);
        }
        if (response.success) {
          received++;
        } else {
          errors++;
          console.error(`Error: ${response.error}`);
        }

        if ((received + errors) % 1000 === 0) {
          console.log(`Progress: ${received + errors}/${totalOrders}`);
        }

        if (received + errors >= totalOrders) {
          listening = false;
        }
      } catch (e) {
        errors++;
      }
    }
  };

  responseListener();

  console.log(`Sending ${totalOrders} orders across ${PAIRS.length} pairs...\n`);
  const startTime = Date.now();

  for (const pair of PAIRS) {
    const pipeline = redis.pipeline();

    for (let i = 0; i < ORDERS_PER_PAIR; i++) {
      const order = generateOrder(pair);
      timestamps.set(order.correlationId, Date.now());
      pipeline.lpush(INCOMING_QUEUE, JSON.stringify(order));
      sent++;
    }

    await pipeline.exec();
    console.log(`Sent ${ORDERS_PER_PAIR} orders for ${pair}`);
  }

  const sendDuration = Date.now() - startTime;
  console.log(`\nAll ${sent} orders sent in ${sendDuration}ms`);
  console.log(`Throughput: ${(sent / sendDuration * 1000).toFixed(0)} orders/sec\n`);

  console.log("Waiting for engine responses...");
  while (listening) {
    await new Promise((r) => setTimeout(r, 100));
  }

  const totalDuration = Date.now() - startTime;

  latencies.sort((a, b) => a - b);
  const avg = latencies.reduce((a, b) => a + b, 0) / latencies.length;
  const p50 = latencies[Math.floor(latencies.length * 0.50)];
  const p95 = latencies[Math.floor(latencies.length * 0.95)];
  const p99 = latencies[Math.floor(latencies.length * 0.99)];
  const min = latencies[0];
  const max = latencies[latencies.length - 1];

  console.log("\nResults:");
  console.log("─────────────────────────────");
  console.log(`Total orders:     ${totalOrders}`);
  console.log(`Sent:             ${sent}`);
  console.log(`Received:         ${received}`);
  console.log(`Errors:           ${errors}`);
  console.log(`Total time:       ${totalDuration}ms`);
  console.log(`Throughput:       ${(received / totalDuration * 1000).toFixed(0)} orders/sec`);
  console.log("─────────────────────────────");
  console.log("Latency (ms):");
  console.log(`  Min:            ${min}ms`);
  console.log(`  Avg:            ${avg.toFixed(2)}ms`);
  console.log(`  p50:            ${p50}ms`);
  console.log(`  p95:            ${p95}ms`);
  console.log(`  p99:            ${p99}ms`);
  console.log(`  Max:            ${max}ms`);
  console.log("─────────────────────────────");

  await redis.quit();
  await responseRedis.quit();
  process.exit(0);
}

runLoadTest().catch(console.error);