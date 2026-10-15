import { useMemo, useState } from "react";
import type { FormEvent } from "react";
import { useTranslation } from "react-i18next";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useContainer } from "@/core/di";
import { errorMessage, isSessionExpired, useCommand } from "@/core/query";
import {
  MIN_PASSWORD_LENGTH,
  makeChangePasswordUseCase,
} from "@/features/auth/domain/use-cases/change-password";

/**
 * Change the owner's password. The vault is not re-encrypted: the backend
 * re-wraps its keys under the new password, so this is instant.
 */
export function ChangePasswordForm() {
  const { t } = useTranslation();
  const { authRepository } = useContainer();
  const changePassword = useMemo(
    () => makeChangePasswordUseCase(authRepository),
    [authRepository],
  );
  const { run, isPending } = useCommand();
  const [current, setCurrent] = useState("");
  const [next, setNext] = useState("");
  const [confirm, setConfirm] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [done, setDone] = useState(false);

  const reset = () => {
    setError(null);
    setDone(false);
  };

  const onSubmit = async (e: FormEvent) => {
    e.preventDefault();
    reset();
    if (next.length < MIN_PASSWORD_LENGTH) {
      setError(t("auth.changePassword.tooShort", { min: MIN_PASSWORD_LENGTH }));
      return;
    }
    if (next !== confirm) {
      setError(t("auth.changePassword.mismatch"));
      return;
    }
    try {
      await run(() => changePassword(current, next));
      setCurrent("");
      setNext("");
      setConfirm("");
      setDone(true);
    } catch (err) {
      if (!isSessionExpired(err)) {
        setError(errorMessage(err, t("errors.generic")));
      }
    }
  };

  const field = (
    id: string,
    label: string,
    value: string,
    set: (v: string) => void,
    autoComplete: string,
  ) => (
    <div className="grid gap-1.5">
      <Label htmlFor={id} className="text-xs text-muted-foreground">
        {label}
      </Label>
      <Input
        id={id}
        type="password"
        value={value}
        autoComplete={autoComplete}
        onChange={(e) => {
          set(e.currentTarget.value);
          reset();
        }}
      />
    </div>
  );

  return (
    <form
      className="flex max-w-sm flex-col gap-3"
      onSubmit={(e) => void onSubmit(e)}
    >
      {field(
        "current-password",
        t("auth.changePassword.current"),
        current,
        setCurrent,
        "current-password",
      )}
      {field(
        "new-password",
        t("auth.changePassword.new"),
        next,
        setNext,
        "new-password",
      )}
      {field(
        "confirm-password",
        t("auth.changePassword.confirm"),
        confirm,
        setConfirm,
        "new-password",
      )}
      {error && (
        <p className="text-sm text-destructive" role="alert">
          {error}
        </p>
      )}
      {done && (
        <p className="text-sm text-success" role="status">
          {t("auth.changePassword.done")}
        </p>
      )}
      <div>
        <Button
          type="submit"
          disabled={isPending || !current || !next || !confirm}
        >
          {isPending ? t("auth.submitting") : t("auth.changePassword.submit")}
        </Button>
      </div>
    </form>
  );
}
