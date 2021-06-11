import { MutableRefObject } from "react";
import { Websocket } from "websocket-ts";

export interface WebSocketData {
  type: string;
  message: string;
  streaming: boolean;
  meta?: string;
}

export type SocketRef = MutableRefObject<Websocket | null>;
