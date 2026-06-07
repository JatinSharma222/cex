import express from "express";
import { appRouter } from "./routes/index"

const app = express();

app.use(express.json());



app.listen(3000, () => {
    console.log("Backend started at 3000")
})