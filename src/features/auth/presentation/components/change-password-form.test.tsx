import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { AppError } from "@/core/errors";
import { ChangePasswordForm } from "@/features/auth/presentation/components/change-password-form";
import { withContainer } from "@/test/app-providers";
import type { AuthRepository } from "@/features/auth/domain/repositories/auth-repository";

function setup(changePassword: AuthRepository["changePassword"]) {
  const authRepository = { changePassword } as unknown as AuthRepository;
  render(<ChangePasswordForm />, {
    wrapper: withContainer({ authRepository }),
  });
  return userEvent.setup();
}

async function fill(
  user: ReturnType<typeof userEvent.setup>,
  current: string,
  next: string,
  confirm: string,
) {
  await user.type(screen.getByLabelText("Mot de passe actuel"), current);
  await user.type(screen.getByLabelText("Nouveau mot de passe"), next);
  await user.type(
    screen.getByLabelText("Confirmer le nouveau mot de passe"),
    confirm,
  );
  await user.click(
    screen.getByRole("button", { name: "Changer le mot de passe" }),
  );
}

describe("ChangePasswordForm", () => {
  it("refuses a confirmation that does not match, without calling the backend", async () => {
    const changePassword = vi.fn();
    const user = setup(changePassword);
    await fill(user, "old-password", "new-password-1", "new-password-2");
    expect(
      screen.getByText("Les deux nouveaux mots de passe ne correspondent pas."),
    ).toBeInTheDocument();
    expect(changePassword).not.toHaveBeenCalled();
  });

  it("changes the password and confirms", async () => {
    const changePassword = vi.fn(() => Promise.resolve());
    const user = setup(changePassword);
    await fill(user, "old-password", "new-password-1", "new-password-1");
    expect(await screen.findByText("Mot de passe changé.")).toBeInTheDocument();
    expect(changePassword).toHaveBeenCalledWith(
      "old-password",
      "new-password-1",
    );
  });

  it("shows the backend's message on a wrong current password", async () => {
    const changePassword = vi.fn(() =>
      Promise.reject(
        new AppError("INVALID_CREDENTIALS", "Identifiants invalides"),
      ),
    );
    const user = setup(changePassword);
    await fill(user, "wrong-password", "new-password-1", "new-password-1");
    expect(
      await screen.findByText("Identifiants invalides"),
    ).toBeInTheDocument();
  });
});
