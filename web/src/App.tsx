import React, { useEffect, useState, useRef, MutableRefObject } from "react";
import { WebsocketBuilder, Websocket } from "websocket-ts";
import AutoScroll from "@brianmcallister/react-auto-scroll";
import { Box, Button, Menu, Grommet, Text } from "grommet";

interface WebSocketData {
  type: string;
  message: string;
  meta?: string;
}

type SocketRef = MutableRefObject<Websocket | null>;

const sendOnSocket = (
  socket: Websocket,
  message: string,
  type: string = "echo",
  meta?: string
) => {
  let data: WebSocketData = {
    message,
    type,
    meta,
  };
  socket.send(JSON.stringify(data));
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

const createSocket = (
  setActive: (active: boolean) => void,
  errorLog: (msg: string) => void,
  onMessage: (data: WebSocketData | null) => void
): Websocket => {
  const socket = new WebsocketBuilder("ws://localhost:9894/api")
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



const requestHelp = (socketRef: SocketRef) => {
  if (socketRef.current) {
    sendOnSocket(socketRef.current, "help", "sh");
  }
};

const requestLs = (socketRef: SocketRef) => {
  if (socketRef.current) {
    sendOnSocket(socketRef.current, "ls", "sh");
  }
};

const requestContext = (socketRef: SocketRef, ctx: string) => {
  if (socketRef.current) {
    sendOnSocket(socketRef.current, `ctx ${ctx}`, "sh");
  }
};

// const sendMessage = (socketRef: MutableRefObject<Websocket | null>) => {
//   if (socketRef.current) {
//     sendOnSocket(socketRef.current, "hallo from sendMessage");
//   }
// };

const onWebSocketMessage =
  (
    setLines: React.Dispatch<React.SetStateAction<string[]>>,
    setContexts: React.Dispatch<React.SetStateAction<string[]>>
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

    if (newLines) {
      addLines(newLines);
    }
  };

const useWebSocket = () => {
  const [active, setActive] = useState(false);
  const [contexts, setContexts] = useState<string[]>([]);
  const boxRef = useRef(null);
  const socketRef: MutableRefObject<Websocket | null> = useRef(null);
  const [lines, setLines] = useState<string[]>([]);

  const [socket, setSocket] = useState(() =>
    createSocket(
      setActive,
      console.error,
      onWebSocketMessage(setLines, setContexts)
    )
  );
  useEffect(() => {
    if (active && socket) {
      // sendOnSocket(socket, "hallo from hook!!!");
      sendOnSocket(socket, "ctx", "sh", "ctx-dropdown");
      socketRef.current = socket;
    }
  }, [active]);

  return { socketRef, lines, clearLines: () => setLines([]), contexts };
};

const CommandOutput = ({ lines }: { lines: string[] }) => {
  return (
    <AutoScroll height={800} showOption={false}>
      {lines.map((msg) => {
        return (
          <Text key={msg} as="pre" size="small" margin="xsmall">
            {msg}
          </Text>
        );
      })}
    </AutoScroll>
  );
};

const setContext = (ctx: string, socketRef: SocketRef) => {
  return () => requestContext(socketRef, ctx);
};

export const App = () => {
  const { socketRef, lines, clearLines, contexts } = useWebSocket();

  return (
    <Grommet plain>
      <Box width="xlarge">
        <Box direction="row-responsive" fill>
          <h1>Kuber</h1>
          <Menu
            label="Switch Context"
            items={contexts.map((ctx) => ({
              label: ctx,
              onClick: setContext(ctx, socketRef),
            }))}
          />
        </Box>

        <CommandOutput lines={lines} />
        <Box direction="row-responsive" fill>
          <Button onClick={clearLines} label="CLEAR" size="large"></Button>
          <Button onClick={() => requestHelp(socketRef)} label="Help"></Button>
          <Button onClick={() => requestLs(socketRef)} label="List"></Button>
        </Box>
      </Box>
    </Grommet>
  );
};
