import { SocketRef } from "./types";
import { sendOnSocket } from "./websocket";
import { API_ENDPOINT } from "./config";

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

export const requestLogApp = (app: string, socketRef: SocketRef) => {
  if (socketRef.current) {
    sendOnSocket(socketRef.current, `log ${app}`, "sh", true, "batching");
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

export const flush = () => {
  return fetch(`${API_ENDPOINT}/flush`, { method: "POST" })
};
