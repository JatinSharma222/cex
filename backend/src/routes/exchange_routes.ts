import { Router } from "express";
import {
  cancelOrder,
  createOrder,
  getBalance,
  getDepth,
  getOrder,
} from "../controllers/exchange-controller";
import { requireAuth } from "../middlewares/auth.js";

export const exchangeRouter = Router();

exchangeRouter.post("/order", requireAuth, createOrder);
exchangeRouter.get("/depth/:symbol", requireAuth, getDepth);
exchangeRouter.get("/balance", requireAuth, getBalance);
exchangeRouter.get("/order/:orderId", requireAuth, getOrder);
exchangeRouter.delete("/order/:orderId", requireAuth, cancelOrder);