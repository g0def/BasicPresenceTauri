import React from "react";
import ReactDOM from "react-dom/client";

import "@/index.css";
import "@/core/i18n";
import App from "@/App";
import { AuthProvider } from "@/features/auth/presentation/providers/auth-provider";
import { UpdaterProvider } from "@/features/updater/presentation/providers/updater-provider";
import { UpdateDialog } from "@/features/updater/presentation/components/update-dialog";
import { applyStoredNoteFont } from "@/shared/note-font/use-note-font";
import { applyStoredTheme } from "@/shared/theme/use-theme";

// Apply persisted display prefs before the first paint (theme avoids a flash of
// light; the note font is set so the editor/preview render in it immediately).
applyStoredTheme();
applyStoredNoteFont();

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
