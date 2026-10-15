import { mockIPC } from "@tauri-apps/api/mocks";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";

import App from "@/App";
import { AppProviders } from "@/test/app-providers";

describe("security notice after login", () => {
  it("reports what happened since the last visit, once", async () => {
    mockIPC((cmd) => {
      switch (cmd) {
        case "account_exists":
          return true;
        case "login":
          return {
            token: "tok",
            expiresAt: Date.now() + 60_000,
            user: { id: "u1", username: "alice", createdAt: 1, updatedAt: 2 },
            failedAttemptsSinceLastLogin: 3,
            securityEvents: [
              { kind: "account_locked", at: 1_717_200_000_000 },
              { kind: "unclean_shutdown", at: 1_717_300_000_000 },
            ],
          };
        case "list_profiles":
          return { profiles: [], activeProfileId: null };
        default:
          throw new Error(`unexpected command: ${cmd}`);
      }
    });

    const user = userEvent.setup();
    render(
      <AppProviders>
        <App />
      </AppProviders>,
    );
    await user.type(await screen.findByLabelText("Nom d'utilisateur"), "alice");
    await user.type(screen.getByLabelText("Mot de passe"), "secret123");
    await user.click(screen.getByRole("button", { name: "Se connecter" }));

    expect(
      await screen.findByText("Depuis votre dernière connexion"),
    ).toBeInTheDocument();
    expect(
      screen.getByText("3 tentatives de connexion échouées"),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/Compte verrouillé après trop de tentatives/),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/L'application s'est arrêtée sans fermeture propre/),
    ).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "J'ai compris" }));
    expect(
      screen.queryByText("Depuis votre dernière connexion"),
    ).not.toBeInTheDocument();
  });
});
