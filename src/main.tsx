import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { AppProvider } from "./AppContext";
import '@mantine/core/styles.css';
import '@mantine/tiptap/styles.css';
import "./styles.css"
import '@mantine/notifications/styles.css';

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <AppProvider>
      <App />
    </AppProvider>
  </React.StrictMode>,
);
