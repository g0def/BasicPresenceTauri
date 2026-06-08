import { useMemo } from "react";
import { Plus, Trash2 } from "lucide-react";
import { useTranslation } from "react-i18next";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectLabel,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import type {
  CommuteCategory,
  CommuteSegment,
  EmissionFactor,
} from "@/features/commute/domain/entities/commute";
import {
  CATEGORY_ICONS,
  CATEGORY_LABEL_KEYS,
  CATEGORY_ORDER,
  isCarCategory,
} from "@/features/commute/presentation/components/commute-icons";
import { modeLabel } from "@/features/commute/presentation/mode-labels";

interface SegmentEditorProps {
  segments: CommuteSegment[];
  onChange: (next: CommuteSegment[]) => void;
  emissionFactors: EmissionFactor[];
  disabled?: boolean;
}

/** A controlled list of commute legs: per row a grouped transport-type select, a
 * one-way distance, and (for car modes) an occupants field. Shared by the
 * commute form and the custom-trip step of the presence dialog. */
export function SegmentEditor({
  segments,
  onChange,
  emissionFactors,
  disabled,
}: SegmentEditorProps) {
  const { t } = useTranslation();

  const factorById = useMemo(
    () => new Map(emissionFactors.map((f) => [f.modeId, f])),
    [emissionFactors],
  );

  const groups = useMemo(() => {
    const byCategory = new Map<CommuteCategory, EmissionFactor[]>();
    for (const f of emissionFactors) {
      const list = byCategory.get(f.category) ?? [];
      list.push(f);
      byCategory.set(f.category, list);
    }
    return CATEGORY_ORDER.filter((c) => byCategory.has(c)).map((category) => ({
      category,
      factors: byCategory.get(category) ?? [],
    }));
  }, [emissionFactors]);

  const update = (index: number, patch: Partial<CommuteSegment>) =>
    onChange(segments.map((s, i) => (i === index ? { ...s, ...patch } : s)));
  const remove = (index: number) =>
    onChange(segments.filter((_, i) => i !== index));
  const add = () =>
    onChange([...segments, { modeId: "", distanceKm: 0, occupants: 1 }]);

  return (
    <div className="grid gap-3">
      {segments.map((seg, index) => {
        const showOccupants = isCarCategory(
          factorById.get(seg.modeId)?.category,
        );
        return (
          <div
            key={index}
            className="grid gap-2 rounded-md border border-border/60 p-3"
          >
            {/* Transport type on its own full-width line so long mode names
                truncate gracefully instead of crushing the fields beside them. */}
            <div className="grid gap-1.5">
              <Label className="text-xs text-muted-foreground">
                {t("commute.segment.type")}
              </Label>
              <div className="flex items-center gap-2">
                <Select
                  value={seg.modeId}
                  onValueChange={(value) => update(index, { modeId: value })}
                  disabled={disabled}
                >
                  <SelectTrigger className="min-w-0 flex-1">
                    <SelectValue
                      placeholder={t("commute.segment.typePlaceholder")}
                    />
                  </SelectTrigger>
                  <SelectContent position="popper">
                    {groups.map((group) => {
                      const Icon = CATEGORY_ICONS[group.category];
                      return (
                        <SelectGroup key={group.category}>
                          <SelectLabel className="flex items-center gap-2">
                            <Icon className="size-3.5" />
                            {t(CATEGORY_LABEL_KEYS[group.category])}
                          </SelectLabel>
                          {group.factors.map((f) => (
                            <SelectItem key={f.modeId} value={f.modeId}>
                              {modeLabel(f.modeId, f.label, t)}
                            </SelectItem>
                          ))}
                        </SelectGroup>
                      );
                    })}
                  </SelectContent>
                </Select>
                <Button
                  type="button"
                  variant="ghost"
                  size="icon-sm"
                  onClick={() => remove(index)}
                  disabled={disabled}
                  aria-label={t("commute.segment.remove")}
                  className="shrink-0"
                >
                  <Trash2 />
                </Button>
              </div>
            </div>

            <div className="flex items-end gap-2">
              <div className="grid flex-1 gap-1.5">
                <Label className="text-xs text-muted-foreground">
                  {t("commute.segment.distance")}
                </Label>
                <Input
                  type="number"
                  min={0}
                  step="0.1"
                  value={seg.distanceKm || ""}
                  onChange={(e) =>
                    update(index, { distanceKm: e.target.valueAsNumber || 0 })
                  }
                  disabled={disabled}
                />
              </div>

              {showOccupants && (
                <div className="grid w-24 gap-1.5">
                  <Label className="text-xs text-muted-foreground">
                    {t("commute.segment.occupants")}
                  </Label>
                  <Input
                    type="number"
                    min={1}
                    step="1"
                    placeholder="1"
                    // Allow the field to be cleared while typing (shown empty);
                    // an empty/invalid value is treated as 1 at compute time.
                    value={seg.occupants || ""}
                    onChange={(e) => {
                      const n = Math.floor(e.target.valueAsNumber);
                      update(index, {
                        occupants: Number.isFinite(n) && n > 0 ? n : 0,
                      });
                    }}
                    disabled={disabled}
                  />
                </div>
              )}
            </div>
          </div>
        );
      })}

      <Button
        type="button"
        variant="outline"
        size="sm"
        onClick={add}
        disabled={disabled}
        className="justify-self-start"
      >
        <Plus />
        {t("commute.segment.add")}
      </Button>
    </div>
  );
}
