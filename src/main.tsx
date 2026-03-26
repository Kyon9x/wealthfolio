import { RUN_ENV, getRunEnv } from "@/adapters";
import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./styles.css";

const runEnv = getRunEnv();

if (runEnv === RUN_ENV.DESKTOP && !import.meta.env.DEV) {
  void import("./lockdown").then(({ installLockdown }) => {
    installLockdown();
  });
}

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
