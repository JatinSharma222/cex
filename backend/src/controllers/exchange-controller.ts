import type { Request, Response } from "express";
import {
  orderBodySchema,
  orderIdParamSchema,
  symbolParamSchema,
} from "../types/exchange-schema";
import { sendValidationError } from "../utils/validation";
import { sendToEngine } from "../utils/engine-client";

function getUserId(req: Request): string {
  if (!req.userId) throw new Error("Missing authenticated user");
  return req.userId;
}

export async function createOrder(req: Request, res: Response): Promise<void> {
  const userId = getUserId(req);
  const parsedBody = orderBodySchema.safeParse(req.body);
  if (!parsedBody.success) {
    sendValidationError(res, parsedBody.error);
    return;
  }

  const { type, side, symbol, qty } = parsedBody.data;
  const price = type === "market" ? null : parsedBody.data.price;

  const engineResponse = await sendToEngine("create_order", {
    userId,
    type,
    side,
    symbol,
    qty,
    price: type === "market" ? null : price,
  });

  res
    .status(engineResponse.ok ? 200 : 400)
    .json(
      engineResponse.ok ? engineResponse.data : { error: engineResponse.error },
    );
}

export async function getDepth(req: Request, res: Response): Promise<void> {
  const parsedParams = symbolParamSchema.safeParse(req.params);
  if (!parsedParams.success) {
    sendValidationError(res, parsedParams.error);
    return;
  }

  const { symbol } = parsedParams.data;

  const engineResponse = await sendToEngine("get_depth", { symbol });

  res
    .status(engineResponse.ok ? 200 : 400)
    .json(
      engineResponse.ok ? engineResponse.data : { error: engineResponse.error },
    );
}

export async function getBalance(req: Request, res: Response): Promise<void> {
  const engineResponse = await sendToEngine("get_user_balance", {
    userId: getUserId(req),
  });

  res.status(engineResponse.ok ? 200 : 400).json(
    engineResponse.ok
      ? engineResponse.data
      : {
          error: engineResponse.error,
        },
  );
}

export function getOrder() {}

export function cancelOrder() {}
