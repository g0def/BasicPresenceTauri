import { useMemo } from "react";
import { Link } from "@tanstack/react-router";
import { ChevronLeft, ExternalLink } from "lucide-react";
import { useTranslation } from "react-i18next";

import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { openExternal } from "@/core/external-link";
import { modeLabel } from "@/features/commute/presentation/mode-labels";
import {
  CATEGORY_ORDER,
  type Co2ReferentialFactor,
} from "@/features/methodology/domain/entities/co2-referential";
import { useCo2Referential } from "@/features/methodology/presentation/hooks/use-co2-referential";
import { SOURCE_GROUPS } from "@/features/methodology/presentation/sources";

const TRANSPARENCY_ITEMS = [1, 2, 3, 4, 5] as const;

export function MethodologyPage() {
  const { t, i18n } = useTranslation();
  const { referential, isLoading, isError } = useCo2Referential();
  const lang = i18n.resolvedLanguage ?? "fr";

  const locale = lang === "en" ? "en-US" : "fr-FR";
  const nf = useMemo(
    () => new Intl.NumberFormat(locale, { maximumFractionDigits: 5 }),
    [locale],
  );
  const nf2 = useMemo(
    () => new Intl.NumberFormat(locale, { maximumFractionDigits: 2 }),
    [locale],
  );

  const factorsByCategory = useMemo(() => {
    const map = new Map<string, Co2ReferentialFactor[]>();
    for (const f of referential?.factors ?? []) {
      const list = map.get(f.category) ?? [];
      list.push(f);
      map.set(f.category, list);
    }
    return map;
  }, [referential]);

  const examples = useMemo(() => {
    if (!referential) return [];
    const byId = new Map(referential.factors.map((f) => [f.modeId, f]));
    const out: {
      key: "car" | "carpool" | "train" | "plane";
      km: number;
      factor: number;
      total: number;
    }[] = [];
    const carPetrol = byId.get("car_petrol");
    if (carPetrol) {
      out.push({
        key: "car",
        km: 20,
        factor: carPetrol.value,
        total: 20 * 2 * carPetrol.value,
      });
      out.push({
        key: "carpool",
        km: 20,
        factor: carPetrol.value,
        total: (20 * 2 * carPetrol.value) / 3,
      });
    }
    const tgv = byId.get("train_hs_fr");
    if (tgv)
      out.push({
        key: "train",
        km: 400,
        factor: tgv.value,
        total: 400 * 2 * tgv.value,
      });
    const plane = byId.get("plane_short");
    if (plane)
      out.push({
        key: "plane",
        km: 500,
        factor: plane.value,
        total: 500 * plane.value,
      });
    return out;
  }, [referential]);

  return (
    <section className="mt-6 mx-auto w-full max-w-3xl flex flex-col gap-8 pb-12">
      <div className="flex items-center gap-3">
        <Button variant="ghost" size="icon" asChild>
          <Link to="/" aria-label={t("methodology.back")}>
            <ChevronLeft />
          </Link>
        </Button>
        <h2 className="text-xl font-semibold">{t("methodology.title")}</h2>
      </div>

      {/* Intro */}
      <p className="text-sm text-muted-foreground">
        {referential
          ? t("methodology.intro", {
              year: referential.factorYear,
              rf: nf.format(referential.radiativeForcing),
            })
          : t("methodology.introGeneric")}
      </p>

      {/* Formulas */}
      <Card>
        <CardHeader>
          <CardTitle>{t("methodology.formula.title")}</CardTitle>
        </CardHeader>
        <CardContent className="flex flex-col gap-4 text-sm">
          <div className="flex flex-col gap-1">
            <span className="font-medium">
              {t("methodology.formula.tripTitle")}
            </span>
            <code className="block rounded-md bg-muted px-3 py-2 text-xs">
              {t("methodology.formula.trip")}
            </code>
            <p className="text-muted-foreground">
              {t("methodology.formula.tripNote")}
            </p>
          </div>
          <div className="flex flex-col gap-1">
            <span className="font-medium">
              {t("methodology.formula.dayTitle")}
            </span>
            <code className="block rounded-md bg-muted px-3 py-2 text-xs">
              {t("methodology.formula.day")}
            </code>
            <p className="text-muted-foreground">
              {t("methodology.formula.dayNote")}
            </p>
          </div>
        </CardContent>
      </Card>

      {/* Parameters */}
      <Card>
        <CardHeader>
          <CardTitle>{t("methodology.params.title")}</CardTitle>
        </CardHeader>
        <CardContent>
          <dl className="flex flex-col gap-3 text-sm">
            {(
              [
                "radiativeForcing",
                "carpool",
                "roundTrip",
                "building",
                "gridCountry",
                "year",
              ] as const
            ).map((key) => (
              <div key={key} className="flex flex-col">
                <dt className="font-medium">
                  {t(`methodology.params.${key}`)}
                </dt>
                <dd className="text-muted-foreground">
                  {key === "radiativeForcing" && referential
                    ? t("methodology.params.radiativeForcingDesc", {
                        rf: nf.format(referential.radiativeForcing),
                        year: referential.factorYear,
                      })
                    : t(`methodology.params.${key}Desc`)}
                </dd>
              </div>
            ))}
          </dl>
        </CardContent>
      </Card>

      {/* Emission-factor table (live) */}
      <Card>
        <CardHeader>
          <CardTitle>{t("methodology.table.title")}</CardTitle>
          <CardDescription>
            {referential
              ? t("methodology.table.subtitle", {
                  year: referential.factorYear,
                })
              : ""}
          </CardDescription>
        </CardHeader>
        <CardContent>
          {isLoading && (
            <p className="text-sm text-muted-foreground">
              {t("methodology.loading")}
            </p>
          )}
          {isError && (
            <p className="text-sm text-destructive">{t("methodology.error")}</p>
          )}
          {!isLoading && !isError && !referential && (
            <p className="text-sm text-muted-foreground">
              {t("methodology.noProfile")}
            </p>
          )}
          {referential &&
            CATEGORY_ORDER.filter((c) => factorsByCategory.has(c)).map(
              (category) => (
                <div key={category} className="mb-6 last:mb-0">
                  <h3 className="mb-2 text-sm font-semibold">
                    {t(`methodology.table.categories.${category}`)}
                  </h3>
                  <div className="overflow-x-auto">
                    <table className="w-full text-left text-sm">
                      <thead>
                        <tr className="border-b text-xs text-muted-foreground">
                          <th className="py-1 pr-3 font-medium">
                            {t("methodology.table.mode")}
                          </th>
                          <th className="py-1 pr-3 font-medium">
                            {t("methodology.table.value")}
                          </th>
                          <th className="py-1 pr-3 font-medium">
                            {t("methodology.table.scope")}
                          </th>
                          <th className="py-1 font-medium">
                            {t("methodology.table.source")}
                          </th>
                        </tr>
                      </thead>
                      <tbody>
                        {(factorsByCategory.get(category) ?? []).map((f) => (
                          <tr
                            key={f.modeId}
                            className="border-b last:border-0 align-top"
                          >
                            <td className="py-2 pr-3">
                              {modeLabel(f.modeId, f.label, t)}
                            </td>
                            <td className="py-2 pr-3 whitespace-nowrap">
                              <span className="font-medium">
                                {nf.format(f.value)}
                              </span>{" "}
                              <span className="text-xs text-muted-foreground">
                                {f.unit}
                              </span>
                              {f.gridVariants.length > 0 && (
                                <div className="mt-1 text-xs text-muted-foreground">
                                  {f.gridVariants
                                    .map(
                                      (v) =>
                                        `${v.country} ${nf.format(v.value)}`,
                                    )
                                    .join(" · ")}
                                </div>
                              )}
                            </td>
                            <td className="py-2 pr-3 text-xs text-muted-foreground">
                              {f.scope ?? "—"}
                            </td>
                            <td className="py-2 text-xs text-muted-foreground">
                              {f.source ?? "—"}
                            </td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>
                </div>
              ),
            )}
        </CardContent>
      </Card>

      {/* Worked examples (computed from the live factors) */}
      {examples.length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle>{t("methodology.examples.title")}</CardTitle>
          </CardHeader>
          <CardContent>
            <ul className="flex flex-col gap-2 text-sm">
              {examples.map((ex) => (
                <li key={ex.key} className="text-muted-foreground">
                  {t(`methodology.examples.${ex.key}`, {
                    km: ex.km,
                    factor: nf.format(ex.factor),
                    total: nf2.format(ex.total),
                  })}
                </li>
              ))}
            </ul>
          </CardContent>
        </Card>
      )}

      {/* Sources */}
      <Card>
        <CardHeader>
          <CardTitle>{t("methodology.sources.title")}</CardTitle>
          <CardDescription>{t("methodology.sources.intro")}</CardDescription>
        </CardHeader>
        <CardContent>
          <ul className="flex flex-col gap-3 text-sm">
            {SOURCE_GROUPS.map((group) => (
              <li key={group.org} className="flex flex-col gap-1">
                <span className="font-medium">{group.org}</span>
                <div className="flex flex-wrap gap-x-4 gap-y-1">
                  {group.links.map((link) => (
                    <button
                      key={link.url}
                      type="button"
                      onClick={() => void openExternal(link.url)}
                      className="inline-flex items-center gap-1 text-primary hover:underline"
                    >
                      {link.label}
                      <ExternalLink className="size-3" />
                    </button>
                  ))}
                </div>
              </li>
            ))}
          </ul>
        </CardContent>
      </Card>

      {/* Transparency / limitations */}
      <details className="rounded-xl border bg-card px-6 py-4 text-sm">
        <summary className="cursor-pointer font-semibold">
          {t("methodology.transparency.title")}
        </summary>
        <p className="mt-2 text-muted-foreground">
          {t("methodology.transparency.intro")}
        </p>
        <ul className="mt-2 flex list-disc flex-col gap-1 pl-5 text-muted-foreground">
          {TRANSPARENCY_ITEMS.map((n) => (
            <li key={n}>{t(`methodology.transparency.item${n}`)}</li>
          ))}
        </ul>
      </details>
    </section>
  );
}
