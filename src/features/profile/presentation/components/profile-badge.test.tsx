import { mockIPC } from "@tauri-apps/api/mocks";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";

import App from "@/App";
import { AuthProvider } from "@/features/auth/presentation/providers/auth-provider";
import { UpdaterProvider } from "@/features/updater/presentation/providers/updater-provider";

const ADA = {
  id: "p1",
  firstName: "Ada",
  lastName: "Lovelace",
  enterprise: "Analytical Engine",
  poste: null,
  createdAt: 1,
  updatedAt: 2,
};

describe("profile home", () => {
  it("shows the presence calendar and the active profile in the header badge", async () => {
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
          return { profiles: [ADA], activeProfileId: ADA.id };
        case "list_presences":
          return [];
        case "list_emission_factors":
          return [];
        case "list_commutes":
          return [];
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

    await screen.findByRole("heading", { name: "Connexion" });
    await user.type(screen.getByLabelText("Nom d'utilisateur"), "alice");
    await user.type(screen.getByLabelText("Mot de passe"), "secret123");
    await user.click(screen.getByRole("button", { name: "Se connecter" }));

    // The calendar is the main view (legend label proves it rendered) and the
    // header badge shows the active profile's full name.
    expect(await screen.findByText("Bureau")).toBeInTheDocument();
    expect(screen.getByText("Ada Lovelace")).toBeInTheDocument();
  });
});
