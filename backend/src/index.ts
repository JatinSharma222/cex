import cors from "cors";
import express from "express";
import { appRouter } from "./routes/index"

const app = express();

app.use(cors());
app.use(express.json());

app.use(appRouter);


app.listen(3000, () => {
    console.log("Backend started at 3000")
})