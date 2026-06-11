import React from "react";
import ReactDOM from "react-dom/client";

import "@/index.css";
import "@/core/i18n";
import App from "@/App";
import { AuthProvider } from "@/features/auth/presentation/providers/auth-provider";
import { UpdaterProvider } from "@/features/updater/presentation/providers/updater-provider";
import { UpdateDialog } from "@/features/updater/presentation/components/update-dialog";
import { applyStoredTheme } from "@/shared/theme/use-theme";

// Apply the persisted theme before the first paint to avoid a flash of light.
applyStoredTheme();

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <AuthProvider>
      <UpdaterProvider>
        <App />
        <UpdateDialog />
      </UpdaterProvider>
    </AuthProvider>
  </React.StrictMode>,
);
