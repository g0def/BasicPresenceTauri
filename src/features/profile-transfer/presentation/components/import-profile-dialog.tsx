import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import {
  effective,
  presentCategories,
  selectionFromManifest,
  type BundleImportSummary,
  type BundleManifest,
  type BundleSelection,
  type ConflictStrategy,
  type ImportTargetKind,
} from "@/features/profile-transfer/domain/entities/bundle";
import {
  ConflictStrategyPicker,
  ImportCategoryPicker,
  ImportTargetPicker,
  type NewProfileForm,
} from "@/features/profile-transfer/presentation/components/import-review-steps";
import { useProfileTransfer } from "@/features/profile-transfer/presentation/hooks/use-profile-transfer";
import { useProfile } from "@/entities/profile/model/use-profile";
import { usePresence } from "@/entities/presence/model/use-presence";

interface ImportProfileDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

type Stage = "idle" | "review" | "result";

/**
 * Import a shared profile bundle. Flow: pick a file → the backend inspects it →
 * review (only the categories the file actually contains, pick a destination,
 * resolve day conflicts when merging) → import → summary. Replaces the old
 * presences-only migration import.
 */
export function ImportProfileDialog({
  open,
  onOpenChange,
}: ImportProfileDialogProps) {
  const { t } = useTranslation();
  const { profiles, activeProfile, refresh } = useProfile();
  const { reload } = usePresence();
  const {
    inspectBundle,
    importBundle,
    isInspecting,
    isImporting,
    error,
    clearError,
  } = useProfileTransfer();

  const [stage, setStage] = useState<Stage>("idle");
  const [localError, setLocalError] = useState<string | null>(null);
  const [handle, setHandle] = useState<string | null>(null);
  const [manifest, setManifest] = useState<BundleManifest | null>(null);
  const [selection, setSelection] = useState<BundleSelection | null>(null);
  const [targetKind, setTargetKind] = useState<ImportTargetKind>("new");
  const [targetProfileId, setTargetProfileId] = useState<string>("");
  const [newProfile, setNewProfile] = useState<NewProfileForm>({
    firstName: "",
    lastName: "",
    enterprise: "",
    poste: "",
  });
  const [conflict, setConflict] = useState<ConflictStrategy>("skip");
  const [summary, setSummary] = useState<BundleImportSummary | null>(null);

  // Reset everything whenever the dialog (re)opens.
  useEffect(() => {
    if (!open) return;
    setStage("idle");
    setLocalError(null);
    setHandle(null);
    setManifest(null);
    setSelection(null);
    setTargetKind("new");
    setTargetProfileId("");
    setNewProfile({ firstName: "", lastName: "", enterprise: "", poste: "" });
    setConflict("skip");
    setSummary(null);
    clearError();
  }, [open, clearError]);

  const chooseFile = async () => {
    setLocalError(null);
    const res = await inspectBundle();
    if (!res) return; // cancelled, or a backend error already surfaced
    const { handle: picked, manifest: m } = res;
    if (!m.compatible) {
      setLocalError(t("transfer.import.incompatible"));
      return;
    }
    setHandle(picked);
    setManifest(m);
    setSelection(selectionFromManifest(m));
    setNewProfile({
      firstName: m.profileFirstName,
      lastName: m.profileLastName,
      enterprise: m.profileEnterprise,
      poste: m.profilePoste ?? "",
    });
    setTargetKind("new");
    setTargetProfileId(activeProfile?.id ?? profiles[0]?.id ?? "");
    setConflict("skip");
    setStage("review");
  };

  const toggle = (key: keyof BundleSelection) =>
    setSelection((prev) => (prev ? { ...prev, [key]: !prev[key] } : prev));

  const runImport = async () => {
    if (!handle || !selection) return;
    const sel = effective(selection);
    const target =
      targetKind === "new"
        ? {
            kind: "new" as const,
            firstName: newProfile.firstName.trim(),
            lastName: newProfile.lastName.trim(),
            enterprise: newProfile.enterprise.trim(),
            poste: newProfile.poste.trim() || null,
          }
        : { kind: "existing" as const, profileId: targetProfileId };
    const result = await importBundle(handle, sel, target, conflict);
    if (result) {
      // Surface the imported/changed profile + refresh the calendar if we merged
      // into the active one.
      await refresh();
      await reload();
      setSummary(result);
      setStage("result");
    }
  };

  const categories = manifest ? presentCategories(manifest) : [];
  const sel = selection;
  const nothingSelected =
    !sel ||
    !categories.some((c) => {
      const e = effective(sel);
      return e[c.key];
    });
  const newProfileIncomplete =
    targetKind === "new" &&
    (!newProfile.firstName.trim() ||
      !newProfile.lastName.trim() ||
      !newProfile.enterprise.trim());
  const existingMissing = targetKind === "existing" && !targetProfileId;
  const importDisabled =
    isImporting || nothingSelected || newProfileIncomplete || existingMissing;

  const feedback = localError ?? error;

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-h-[85vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>{t("transfer.import.title")}</DialogTitle>
          {stage === "idle" && (
            <DialogDescription>
              {t("transfer.import.description")}
            </DialogDescription>
          )}
        </DialogHeader>

        {stage === "review" && manifest && sel && (
          <div className="grid gap-4 text-sm">
            <p className="text-muted-foreground">
              {t("transfer.import.fileFrom", {
                name: `${manifest.profileFirstName} ${manifest.profileLastName}`,
              })}
            </p>

            {manifest.unknownModeIds.length > 0 && (
              <p className="rounded-md bg-muted px-3 py-2 text-muted-foreground">
                {t("transfer.import.unknownModes", {
                  count: manifest.unknownModeIds.length,
                })}
              </p>
            )}

            {/* Destination */}
            <ImportTargetPicker
              targetKind={targetKind}
              setTargetKind={setTargetKind}
              newProfile={newProfile}
              setNewProfile={setNewProfile}
              targetProfileId={targetProfileId}
              setTargetProfileId={setTargetProfileId}
              profiles={profiles}
            />

            {/* Categories present in the file */}
            <ImportCategoryPicker
              categories={categories}
              sel={sel}
              toggle={toggle}
            />

            {/* Conflict strategy — only when merging days into an existing profile */}
            {targetKind === "existing" && sel.includeDays && (
              <ConflictStrategyPicker
                conflict={conflict}
                setConflict={setConflict}
              />
            )}
          </div>
        )}

        {stage === "result" && summary && (
          <div className="grid gap-2 text-sm">
            <p>
              {t("transfer.import.result", {
                imported: summary.daysImported,
                replaced: summary.daysReplaced,
                skipped: summary.daysSkipped,
              })}
            </p>
          </div>
        )}

        {feedback && <p className="text-sm text-destructive">{feedback}</p>}

        <DialogFooter>
          {stage === "idle" && (
            <>
              <Button variant="outline" onClick={() => onOpenChange(false)}>
                {t("transfer.import.cancel")}
              </Button>
              <Button disabled={isInspecting} onClick={() => void chooseFile()}>
                {t("transfer.import.chooseFile")}
              </Button>
            </>
          )}
          {stage === "review" && (
            <>
              <Button variant="outline" onClick={() => onOpenChange(false)}>
                {t("transfer.import.cancel")}
              </Button>
              <Button
                disabled={importDisabled}
                onClick={() => void runImport()}
              >
                {isImporting
                  ? t("transfer.import.importing")
                  : t("transfer.import.importButton")}
              </Button>
            </>
          )}
          {stage === "result" && (
            <Button onClick={() => onOpenChange(false)}>
              {t("transfer.import.done")}
            </Button>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
