console.log("1")

import cors from "cors";
import express from "express";
import { appRouter } from "./routes/index"

console.log("2")
const app = express();

app.use(cors());
app.use(express.json());

app.use(appRouter);
console.log("3")


app.listen(3000, () => {
    console.log("Backend started at 3000")
})