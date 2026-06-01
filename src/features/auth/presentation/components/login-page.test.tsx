import { mockIPC } from "@tauri-apps/api/mocks";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";

import App from "@/App";
import { AuthProvider } from "@/features/auth/presentation/providers/auth-provider";

describe("auth flow", () => {
  it("logs in an existing account and shows the username on the home screen", async () => {
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
        default:
          throw new Error(`unexpected command: ${cmd}`);
      }
    });

    const user = userEvent.setup();
    render(
      <AuthProvider>
        <App />
      </AuthProvider>,
    );

    // The backend reports an existing account → login screen.
    await screen.findByRole("heading", { name: "Connexion" });

    await user.type(screen.getByLabelText("Nom d'utilisateur"), "alice");
    await user.type(screen.getByLabelText("Mot de passe"), "secret123");
    await user.click(screen.getByRole("button", { name: "Se connecter" }));

    expect(await screen.findByText(/Bonjour alice/)).toBeInTheDocument();
  });
});
