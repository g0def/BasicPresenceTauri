import { mockIPC } from "@tauri-apps/api/mocks";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";

import App from "@/App";
import { AuthProvider } from "@/features/auth/presentation/providers/auth-provider";
import { UpdaterProvider } from "@/features/updater/presentation/providers/updater-provider";

describe("auth flow", () => {
  it("logs in an existing account and lands on the empty-profile home", async () => {
    mockIPC((cmd) => {
      switch (cmd) {
        case "account_exists":
          return true;
        case "login":
          return {
            token: "tok",
            expiresAt: Date.now() + 60_000,
            user: { id: "u1", username: "alice", createdAt: 1, updatedAt: 2 },
          };
        case "list_profiles":
          return { profiles: [], activeProfileId: null };
        default:
          throw new Error(`unexpected command: ${cmd}`);
      }
    });

    const user = userEvent.setup();
    render(
      <AuthProvider>
        <UpdaterProvider>
          <App />
        </UpdaterProvider>
      </AuthProvider>,
    );

    // The backend reports an existing account → login screen.
    await screen.findByLabelText("Nom d'utilisateur");

    await user.type(screen.getByLabelText("Nom d'utilisateur"), "alice");
    await user.type(screen.getByLabelText("Mot de passe"), "secret123");
    await user.click(screen.getByRole("button", { name: "Se connecter" }));

    // No profile yet → the centered "add profile" prompt is shown.
    expect(
      await screen.findByRole("button", { name: /Ajouter un profil/ }),
    ).toBeInTheDocument();
  });

  it("switches the UI language to English", async () => {
    mockIPC((cmd) => {
      switch (cmd) {
        case "account_exists":
          return true;
        default:
          throw new Error(`unexpected command: ${cmd}`);
      }
    });

    const user = userEvent.setup();
    render(
      <AuthProvider>
        <UpdaterProvider>
          <App />
        </UpdaterProvider>
      </AuthProvider>,
    );

    // Starts in French (pinned by the test setup).
    await screen.findByLabelText("Nom d'utilisateur");

    await user.click(screen.getByRole("combobox", { name: "Langue" }));
    await user.click(await screen.findByRole("option", { name: "English" }));

    expect(await screen.findByLabelText("Username")).toBeInTheDocument();
  });
});
