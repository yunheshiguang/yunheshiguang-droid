import React from "react";
import ReactDOM from "react-dom/client";

import { App } from "@/app/app";
import { renderErrorReporting, reportUnhandledErrors } from "@/app/error-reporting";
import { isTauriHost } from "@/shared/api";
import "@/styles/globals.css";

reportUnhandledErrors();

async function installDevHost() {
  if (import.meta.env.DEV && !isTauriHost()) {
    const { installMockHost } = await import("@/shared/mock-host");

    installMockHost();
  }
}

await installDevHost();

ReactDOM.createRoot(document.getElementById("root") as HTMLElement, renderErrorReporting).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
