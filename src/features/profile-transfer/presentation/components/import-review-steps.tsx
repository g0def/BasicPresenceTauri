import { useTranslation } from "react-i18next";

import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { fullName } from "@/entities/profile/model/profile";
import type { Profile } from "@/entities/profile/model/profile";
import {
  DAY_CHILD_KEYS,
  type BundleSelection,
  type ConflictStrategy,
  type ImportTargetKind,
  type presentCategories as PresentCategories,
} from "@/features/profile-transfer/domain/entities/bundle";

export interface NewProfileForm {
  firstName: string;
  lastName: string;
  enterprise: string;
  poste: string;
}

/** Review step — where the imported data goes: a new profile (identity
 *  prefilled from the file) or an existing one. */
export function ImportTargetPicker({
  targetKind,
  setTargetKind,
  newProfile,
  setNewProfile,
  targetProfileId,
  setTargetProfileId,
  profiles,
}: {
  targetKind: ImportTargetKind;
  setTargetKind: (kind: ImportTargetKind) => void;
  newProfile: NewProfileForm;
  setNewProfile: (update: (p: NewProfileForm) => NewProfileForm) => void;
  targetProfileId: string;
  setTargetProfileId: (id: string) => void;
  profiles: Profile[];
}) {
  const { t } = useTranslation();
  return (
    <div className="grid gap-2">
      <span className="font-medium">{t("transfer.import.target.heading")}</span>
      <div className="flex flex-wrap gap-2">
        {(["new", "existing"] as ImportTargetKind[]).map((kind) => (
          <button
            key={kind}
            type="button"
            onClick={() => setTargetKind(kind)}
            className={`rounded-md border px-3 py-1.5 text-sm transition-colors ${
              targetKind === kind
                ? "border-primary bg-primary text-primary-foreground"
                : "bg-card hover:bg-accent"
            }`}
          >
            {t(`transfer.import.target.${kind}`)}
          </button>
        ))}
      </div>

      {targetKind === "new" ? (
        <div className="grid gap-2 sm:grid-cols-2">
          <div className="grid gap-1">
            <Label htmlFor="import-first-name">{t("profile.firstName")}</Label>
            <Input
              id="import-first-name"
              value={newProfile.firstName}
              onChange={(e) =>
                setNewProfile((p) => ({
                  ...p,
                  firstName: e.target.value,
                }))
              }
            />
          </div>
          <div className="grid gap-1">
            <Label htmlFor="import-last-name">{t("profile.lastName")}</Label>
            <Input
              id="import-last-name"
              value={newProfile.lastName}
              onChange={(e) =>
                setNewProfile((p) => ({
                  ...p,
                  lastName: e.target.value,
                }))
              }
            />
          </div>
          <div className="grid gap-1 sm:col-span-2">
            <Label htmlFor="import-enterprise">{t("profile.enterprise")}</Label>
            <Input
              id="import-enterprise"
              value={newProfile.enterprise}
              onChange={(e) =>
                setNewProfile((p) => ({
                  ...p,
                  enterprise: e.target.value,
                }))
              }
            />
          </div>
        </div>
      ) : (
        <Select value={targetProfileId} onValueChange={setTargetProfileId}>
          <SelectTrigger className="w-full">
            <SelectValue
              placeholder={t("transfer.import.target.selectProfile")}
            />
          </SelectTrigger>
          <SelectContent>
            {profiles.map((p) => (
              <SelectItem key={p.id} value={p.id}>
                {fullName(p)}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      )}
    </div>
  );
}

/** Review step — the categories the file contains, to include or not. */
export function ImportCategoryPicker({
  categories,
  sel,
  toggle,
}: {
  categories: ReturnType<typeof PresentCategories>;
  sel: BundleSelection;
  toggle: (key: keyof BundleSelection) => void;
}) {
  const { t } = useTranslation();
  return (
    <div className="grid gap-2">
      <span className="font-medium">
        {t("transfer.import.categoriesHeading")}
      </span>
      {categories.map(({ key, count }) => {
        const child = DAY_CHILD_KEYS.includes(key);
        const disabled = child && !sel.includeDays;
        return (
          <label
            key={key}
            className={`flex items-center gap-2 ${child ? "pl-6" : ""} ${
              disabled ? "opacity-50" : ""
            }`}
          >
            <input
              type="checkbox"
              className="size-4"
              disabled={disabled}
              checked={!disabled && sel[key]}
              onChange={() => toggle(key)}
            />
            {t(`transfer.fields.${key}`)}
            {count !== undefined ? ` (${count})` : ""}
          </label>
        );
      })}
    </div>
  );
}

/** Review step — what to do with days that already exist when merging. */
export function ConflictStrategyPicker({
  conflict,
  setConflict,
}: {
  conflict: ConflictStrategy;
  setConflict: (s: ConflictStrategy) => void;
}) {
  const { t } = useTranslation();
  return (
    <div className="grid gap-2">
      <span className="text-muted-foreground">
        {t("transfer.import.conflict.heading")}
      </span>
      <div className="flex flex-wrap gap-2">
        {(["skip", "replace"] as ConflictStrategy[]).map((s) => (
          <button
            key={s}
            type="button"
            onClick={() => setConflict(s)}
            className={`rounded-md border px-3 py-1.5 text-sm transition-colors ${
              conflict === s
                ? "border-primary bg-primary text-primary-foreground"
                : "bg-card hover:bg-accent"
            }`}
          >
            {t(`transfer.import.conflict.${s}`)}
          </button>
        ))}
      </div>
    </div>
  );
}
