import { useTranslation } from "react-i18next";

import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { supportedLngs } from "@/core/i18n/resources";

/**
 * Minimal language picker. Changing the value persists the choice through
 * i18next's localStorage cache (see core/i18n).
 */
export function LanguageSwitcher() {
  const { t, i18n } = useTranslation();

  return (
    <Select
      value={i18n.resolvedLanguage ?? "fr"}
      onValueChange={(value) => void i18n.changeLanguage(value)}
    >
      <SelectTrigger
        size="sm"
        aria-label={t("language.label")}
        className="w-auto gap-2"
      >
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        {supportedLngs.map((lng) => (
          <SelectItem key={lng} value={lng}>
            {t(`language.${lng}`)}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  );
}
