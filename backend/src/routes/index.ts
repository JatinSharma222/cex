import { Router } from "express";
import { authRouter } from "./auth_routes";
import { exchangeRouter } from "./exchange_routes";

export const appRouter = Router();

appRouter.use("/auth", authRouter);
appRouter.use("/exchange", exchangeRouter);