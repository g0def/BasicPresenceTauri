import { useTranslation } from "react-i18next";

import { supportedLngs } from "@/core/i18n/resources";

/**
 * Minimal language picker. Changing the value persists the choice through
 * i18next's localStorage cache (see core/i18n).
 */
export function LanguageSwitcher() {
  const { t, i18n } = useTranslation();

  return (
    <select
      className="language-switcher"
      aria-label={t("language.label")}
      value={i18n.resolvedLanguage ?? "fr"}
      onChange={(e) => void i18n.changeLanguage(e.currentTarget.value)}
    >
      {supportedLngs.map((lng) => (
        <option key={lng} value={lng}>
          {t(`language.${lng}`)}
        </option>
      ))}
    </select>
  );
}
