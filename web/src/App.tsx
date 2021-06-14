import React from "react";
import { Box, Button, Menu, Grommet, Header, Footer, Clock } from "grommet";

// black theme from https://grommet-theme-builder.netlify.app/
import defaultBlackTheme from "./theme.json";
import { useWebSocket } from "./websocket";
import { SocketRef } from "./types";
import {
  flush,
  requestContext,
  requestHelp,
  requestLogApp,
  requestLogin,
  requestLs,
  requestLsApp,
} from "./api";
import { CommandOutput } from "./CommandOutput";

// const myTheme = {
//   global: {
//     font: {
//       family: 'Helvetica',
//     },
//   },
// };

const setContext = (ctx: string, socketRef: SocketRef) => {
  return () => requestContext(socketRef, ctx);
};

const reconnectWebsocket = (socketRef: SocketRef) => {
  if (socketRef.current) {
    socketRef.current.underlyingWebsocket?.close(1000);
  }
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
            <Button
              onClick={() => {
                flush().then(() => reconnectWebsocket(socketRef));
              }}
              label="Cancel"
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
          <Menu
            label="Log app"
            dropAlign={{ bottom: "top" }}
            items={apps.map((app) => ({
              label: app,
              onClick: () => requestLogApp(app, socketRef),
            }))}
          />
        </Footer>
      </Box>
    </Grommet>
  );
};
