import Redis from "ioredis";
import { env } from "./env";
import type {
  EngineCommandType,
  EngineRequest,
  EngineResponse,
} from "../types/engine";

import {
  resolveEngineResponse,
  waitForEngineResponse,
} from "../store/pending-responses";

const publisher = new Redis(env.redisUrl);
const subscriber = new Redis(env.redisUrl);

publisher.on("error", (error) => {
    console.error("Redis publisher error", error)
});

subscriber.on("error", (error) => {
    console.error("Redis subscriber error", error);
});

export async function connectRedis(): Promise<void> {
    await Promise.all([publisher.ping(), subscriber.ping()]);
}

export async function pingRedis(): Promise<string>  {
    return publisher.ping();
}

export function subscribeToEngineUpdates(
  callback: (channel: string, data: unknown) => void
): void {
  subscriber.psubscribe("orderbook:*", "trades:*");

  subscriber.on("pmessage", (_pattern, channel, message) => {
    try {
      const data = JSON.parse(message);
      callback(channel, data);
    } catch (e) {
      console.error("Failed to parse engine update:", e);
    }
  });
}

export async function sendToEngine(
  type: EngineCommandType,
  payload: Record<string, unknown>,
): Promise<EngineResponse> {
    const correlationId = crypto.randomUUID();
    const responsePromise = waitForEngineResponse(correlationId, env.engineTimeoutMs);

    const message: EngineRequest = {
      correlationId,
      responseQueue: env.responseQueue,
      type,
      payload,
    };

    await publisher.lpush(env.incomingQueue, JSON.stringify(message));
    return responsePromise;
}

export async function listenForEngineResponse(): Promise<void> {
    console.log(`Listening for engine responses on ${env.responseQueue}`);

    for (;;) {
        const response = await subscriber.brpop(env.responseQueue, 0);
        if (!response) continue;

        try {
            const parsedResponse = JSON.parse(response[1]) as EngineResponse;
            resolveEngineResponse(parsedResponse);
        } catch (error) {
            console.error("Invalid engine response", error);
        }
    }
}