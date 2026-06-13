import { WebSocket } from "ws";
import { sendToClient, broadcastToRoom } from "./server";
import { sendToEngine } from "../utils/engine-client";

const VALID_SYMBOLS = ["BTC_USDT", "ETH_USDT", "SOl_USDT"];

type WsMessage = {
  type: string;
  symbol?: string;
  payload?: Record<string, unknown>;
};

export async function handleWsMessage(
  ws: WebSocket,
  message: WsMessage,
  orderBookRooms: Map<string, Set<WebSocket>>,
  tradeRooms: Map<string, Set<WebSocket>>,
): Promise<void> {
  const { type, symbol } = message;

  switch (type) {
    case "subscribe_orderbook": {
      if (!symbol || !VALID_SYMBOLS.includes(symbol)) {
        sendToClient(ws, { type: "error", message: "invalid symbol" });
        return;
      }

      if (!orderBookRooms.has(symbol)) {
        orderBookRooms.set(symbol, new Set());
      }

      orderBookRooms.get(symbol)!.add(ws);

      const response = await sendToEngine("get_depth", { symbol });
      if (response.success) {
        sendToClient(ws, {
          type: "orderbook_snapshot",
          symbol,
          data: response.data,
        });
      }
      sendToClient(ws, {
        type: "unsubscribed",
        channel: "orderbook",
        symbol,
      });

      console.log(`Client unsubscribed from orderbook:${symbol}`);
      break;
    }

    case "subscribe_trades": {
      if (!symbol || !VALID_SYMBOLS.includes(symbol)) {
        sendToClient(ws, { type: "error", message: "invalid symbol" });
        return;
      }

      if (!tradeRooms.has(symbol)) {
        tradeRooms.set(symbol, new Set());
      }
      tradeRooms.get(symbol)!.add(ws);

      sendToClient(ws, {
        type: "subscribed",
        channel: "trades",
        symbol,
      });

      console.log(`Client subscribed to trades:${symbol}`);
      break;
    }

    case "unsubscribe_trades": {
      if (!symbol) {
        sendToClient(ws, { type: "error", message: "missing symbol" });
        return;
      }

      const room = tradeRooms.get(symbol);
      if (room) {
        room.delete(ws);
        if (room.size === 0) {
          tradeRooms.delete(symbol);
        }
      }

      sendToClient(ws, {
        type: "unsubscribed",
        channel: "trades",
        symbol,
      });

      console.log(`Client unsubscribed from trades:${symbol}`);
      break;
    }

    case "ping": {
      sendToClient(ws, { type: "pong" });
      break;
    }

    default: {
      sendToClient(ws, {
        type: "error",
        message: `unknown message type: ${type}`,
      });
    }
  }
}
