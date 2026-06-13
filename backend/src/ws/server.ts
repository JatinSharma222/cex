import { WebSocket, WebSocketServer } from "ws";
import { IncomingMessage } from "http";
import { handleWsMessage } from "./handlers";
import { subscribeToEngineUpdates } from "../utils/engine-client";

const orderBookRooms = new Map<string, Set<WebSocket>>();

const tradeRooms = new Map<string, Set<WebSocket>>();

export function createWsServer(port: number): WebSocketServer {
  const wss = new WebSocketServer({ port });

  console.log(`Websocket server started on port ${port}`);

  wss.on("connection", (ws: WebSocket, req: IncomingMessage) => {
    console.log(`New WebSocket connection from ${req.socket.remoteAddress}`);

    ws.on("message", (data) => {
      try {
        const message = JSON.parse(data.toString());
        handleWsMessage(ws, message, orderBookRooms, tradeRooms);
      } catch (e) {
        sendToClient(ws, { type: "error", message: "invalid message format" });
      }
    });

    ws.on("close", () => {
      removeFromAllRooms(ws, orderBookRooms);
      removeFromAllRooms(ws, tradeRooms);
      console.log("Client disconnected");
    });

    ws.on("error", (err) => {
      console.error("Websocket error: ", err);
    });

    sendToClient(ws, {
      type: "connected",
      message: "connected to CEX Websocket",
    });
  });

  subscribeToEngineUpdates((channel, data) => {
    if (channel.startsWith("orderbook:")) {
      const symbol = channel.replace("orderbook:", "");
      broadcastToRoom(orderBookRooms, symbol, {
        type: "orderbook_update",
        symbol,
        data,
      });
    } else if (channel.startsWith("trades:")) {
      const symbol = channel.replace("trades:", "");
      broadcastToRoom(tradeRooms, symbol, {
        type: "trade_update",
        symbol,
        data,
      });
    }
  });

  return wss;
}

export function sendToClient(ws: WebSocket, message: object): void {
  if (ws.readyState === WebSocket.OPEN) {
    ws.send(JSON.stringify(message));
  }
}

export function broadcastToRoom(
  rooms: Map<string, Set<WebSocket>>,
  symbol: string,
  message: object,
): void {
  const room = rooms.get(symbol);
  if (!room) return;

  const payload = JSON.stringify(message);
  for (const client of room) {
    if (client.readyState === WebSocket.OPEN) {
      client.send(payload);
    } else {
      room.delete(client);
    }
  }
}

function removeFromAllRooms(
  ws: WebSocket,
  rooms: Map<string, Set<WebSocket>>,
): void {
  for (const [symbol, clients] of rooms.entries()) {
    clients.delete(ws);
    if (clients.size === 0) {
      rooms.delete(symbol);
    }
  }
}
