import React, { useEffect, useState, useRef, MutableRefObject } from "react";
import { WebsocketBuilder, Websocket } from "websocket-ts";
import AutoScroll from "@brianmcallister/react-auto-scroll";
import {
  Anchor,
  Box,
  Button,
  Menu,
  Grommet,
  Text,
  Header,
  Footer,
  Main,
  Clock,
} from "grommet";
// black theme from https://grommet-theme-builder.netlify.app/
import defaultBlackTheme from "./theme.json";

interface WebSocketData {
  type: string;
  message: string;
  streaming: boolean;
  meta?: string;
}

type SocketRef = MutableRefObject<Websocket | null>;

// const myTheme = {
//   global: {
//     font: {
//       family: 'Helvetica',
//     },
//   },
// };

const sendOnSocket = (
  socket: Websocket,
  message: string,
  type: string = "echo",
  streaming: boolean = true,
  meta?: string,
) => {
  let data: WebSocketData = {
    message,
    type,
    meta,
    streaming
  };
  socket.send(JSON.stringify(data));
};

// regex from https://github.com/sindresorhus/linkify-urls
const urlRegex =
  /((?<!\+)(?:https?(?::\/\/))(?:www\.)?(?:[a-zA-Z\d-_.]+(?:(?:\.|@)[a-zA-Z\d]{2,})|localhost)(?:(?:[-a-zA-Z\d:%_+.~#*$!?&//=@]*)(?:[,](?![\s]))*)*)/g;

function linkify(text: string) {
  return text
    .split(urlRegex)
    .map((x, index) =>
      index % 2 === 1 ? <Anchor target="_blank" href={x} label={x} /> : x
    );
}

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

const requestLsApp = (app: string, socketRef: SocketRef) => {
  if (socketRef.current) {
    sendOnSocket(socketRef.current, `ls ${app}`, "sh");
  }
};

const requestLogin = (socketRef: SocketRef) => {
  if (socketRef.current) {
    sendOnSocket(socketRef.current, "login", "sh");
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
      return
    }

    if (newLines) {
      addLines(newLines);
    }
  };

const useWebSocket = () => {
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

const CommandOutput = ({ lines }: { lines: string[] }) => {
  return (
    <Main>
      <AutoScroll height={800} showOption={false}>
        {lines.map((msg) => {
          return (
            <Text key={msg} as="pre" size="small" margin="xsmall">
              {linkify(msg)}
            </Text>
          );
        })}
      </AutoScroll>
    </Main>
  );
};

const setContext = (ctx: string, socketRef: SocketRef) => {
  return () => requestContext(socketRef, ctx);
};

export const App = () => {
  const { socketRef, lines, apps, clearLines, contexts } = useWebSocket();

  return (
    <Grommet background={{ color: "dark-1" }} theme={defaultBlackTheme as any}>
      <Box width="xlarge" pad="medium">
        <Header direction="row-responsive" fill>
          <h1>Kuber</h1>
          <Clock type="digital" />
          <Menu
            label="Switch Context"
            items={contexts.map((ctx) => ({
              label: ctx,
              onClick: setContext(ctx, socketRef),
            }))}
          />
        </Header>

        <CommandOutput lines={lines} />
        <Footer>
          <Box direction="row-responsive">
            <Button onClick={clearLines} label="CLEAR" size="large"></Button>
            <Button
              onClick={() => requestHelp(socketRef)}
              label="Help"
            ></Button>
            <Button onClick={() => requestLs(socketRef)} label="List"></Button>
            <Button
              onClick={() => requestLogin(socketRef)}
              label="Login"
            ></Button>
          </Box>
          <Menu
            label="List app"
            dropAlign={{ bottom: "top" }}
            items={apps.map((app) => ({
              label: app,
              onClick: () => requestLsApp(app, socketRef),
            }))}
          />
        </Footer>
      </Box>
    </Grommet>
  );
};
