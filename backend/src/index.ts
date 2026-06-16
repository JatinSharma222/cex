import cors from "cors";
import express from "express";
import { appRouter } from "./routes/index";
import { createWsServer } from "./ws/server";
import { connectRedis, listenForEngineResponse } from "./utils/engine-client";

const app = express();
app.use(cors());
app.use(express.json());
app.use(appRouter);
async function start() {
  await connectRedis();
  console.log("Redis connected");

  listenForEngineResponse(); 

  createWsServer(3001);

  app.listen(3000, () => {
    console.log("Backend started at 3000");
    console.log("WebSocket started at 3001");
  });
}

start().catch(console.error);
