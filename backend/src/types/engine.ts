export type EngineCommandType =
  | "place_order"
  | "get_depth"
  | "get_balance"        
  | "get_order"
  | "cancel_order";

export interface EngineRequest {
  correlationId: string;
  responseQueue: string;
  type: EngineCommandType;
  payload: Record<string, unknown>;
}

export interface EngineResponse {
  correlationId: string;
  success: boolean;      
  data?: unknown;
  error?: string;
}