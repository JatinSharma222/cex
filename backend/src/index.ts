import cors from "cors";
import express from "express";
import { appRouter } from "./routes/index"
import { createWsServer } from "./ws/server";

const app = express();

app.use(cors());
app.use(express.json());

app.use(appRouter);

createWsServer(3001);
app.listen(3000, () => {
    console.log("Backend started at 3000")
})