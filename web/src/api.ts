import { SocketRef } from "./types";
import { sendOnSocket } from "./websocket";

export const requestHelp = (socketRef: SocketRef) => {
  if (socketRef.current) {
    sendOnSocket(socketRef.current, "help", "sh");
  }
};

// const sendMessage = (socketRef: MutableRefObject<Websocket | null>) => {
//   if (socketRef.current) {
//     sendOnSocket(socketRef.current, "hallo from sendMessage");
//   }
// };

export const requestLs = (socketRef: SocketRef) => {
  if (socketRef.current) {
    sendOnSocket(socketRef.current, "ls", "sh");
  }
};

export const requestLsApp = (app: string, socketRef: SocketRef) => {
  if (socketRef.current) {
    sendOnSocket(socketRef.current, `ls ${app}`, "sh");
  }
};

export const requestLogin = (socketRef: SocketRef) => {
  if (socketRef.current) {
    sendOnSocket(socketRef.current, "login", "sh");
  }
};

export const requestContext = (socketRef: SocketRef, ctx: string) => {
  if (socketRef.current) {
    sendOnSocket(socketRef.current, `ctx ${ctx}`, "sh");
  }
};
