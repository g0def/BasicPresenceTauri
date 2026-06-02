import React from "react";
import ReactDOM from "react-dom/client";

import "@/core/i18n";
import App from "@/App";
import { AuthProvider } from "@/features/auth/presentation/providers/auth-provider";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <AuthProvider>
      <App />
    </AuthProvider>
  </React.StrictMode>,
);
