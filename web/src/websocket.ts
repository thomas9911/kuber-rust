import React, { useEffect, useState, useRef, MutableRefObject } from "react";
import { ConstantBackoff, Websocket, WebsocketBuilder } from "websocket-ts";
import { WS_ENDPOINT } from "./config";
import { WebSocketData } from "./types";

export const onWebSocketMessage =
  (
    setLines: React.Dispatch<React.SetStateAction<string[]>>,
    setContexts: React.Dispatch<React.SetStateAction<string[]>>,
    setApps: React.Dispatch<React.SetStateAction<string[]>>
  ) =>
  (data: WebSocketData | null) => {
    const addLines = (extraLines: string[]) => {
      setLines((prevState) => prevState.concat(extraLines));
    };

    const newLines = data?.message.split("\n");

    if (data?.meta === "ctx-dropdown") {
      // addLines(["from dropdown!!!"]);
      if (newLines) {
        setContexts(newLines);
      }
      return;
    }

    if (data?.meta === "apps-dropdown") {
      // addLines(["from dropdown!!!"]);
      if (newLines) {
        setApps(newLines);
      }
      return;
    }

    if (data?.type === "empty") {
      return;
    }

    if (newLines) {
      addLines(newLines);
    }
  };

const parseMessage = (
  data: unknown,
  errorLog: (msg: string) => void
): WebSocketData | null => {
  if (typeof data === "string") {
    try {
      let parsed: WebSocketData = JSON.parse(data);
      if (parsed.type === "error") {
        errorLog(`invalid websocket data: ${parsed.message}`);
        return null;
      }
      return parsed;
    } catch (error) {
      errorLog(`invalid websocket data: ${error}`);
      return null;
    }
  }
  errorLog("invalid websocket data");
  return null;
};

export const sendOnSocket = (
  socket: Websocket,
  message: string,
  type: string = "echo",
  streaming: boolean = true,
  meta?: string
) => {
  let data: WebSocketData = {
    message,
    type,
    meta,
    streaming,
  };
  socket.send(JSON.stringify(data));
};

export const createSocket = (
  setActive: (active: boolean) => void,
  errorLog: (msg: string) => void,
  onMessage: (data: WebSocketData | null) => void
): Websocket => {
  const socket = new WebsocketBuilder(WS_ENDPOINT)
    .withBackoff(new ConstantBackoff(100))
    .onOpen((soc, event) => {
      setActive(true);
      sendOnSocket(soc, "Connected to server");
    })
    .onMessage((soc, event) => {
      let data = parseMessage(event.data, errorLog);
      onMessage(data);
    })
    .build();

  return socket;
};

export const useWebSocket = () => {
  const [active, setActive] = useState(false);
  const [contexts, setContexts] = useState<string[]>([]);
  const [apps, setApps] = useState<string[]>([]);
  const boxRef = useRef(null);
  const socketRef: MutableRefObject<Websocket | null> = useRef(null);
  const [lines, setLines] = useState<string[]>([]);

  const [socket, setSocket] = useState(() =>
    createSocket(
      setActive,
      console.error,
      onWebSocketMessage(setLines, setContexts, setApps)
    )
  );
  useEffect(() => {
    if (active && socket) {
      // sendOnSocket(socket, "hallo from hook!!!");
      sendOnSocket(socket, "ctx", "sh", false, "ctx-dropdown");
      sendOnSocket(socket, "apps", "sh", false, "apps-dropdown");
      socketRef.current = socket;
    }
  }, [active]);

  return { socketRef, lines, apps, clearLines: () => setLines([]), contexts };
};
