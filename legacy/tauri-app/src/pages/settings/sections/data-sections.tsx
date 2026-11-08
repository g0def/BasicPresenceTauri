import { useState } from "react";
import { Download } from "lucide-react";
import { useTranslation } from "react-i18next";

import { Button } from "@/components/ui/button";
import { ExportDialog } from "@/features/export/presentation/components/export-dialog";
import { ExportProfileDialog } from "@/features/profile-transfer/presentation/components/export-profile-dialog";
import { ImportProfileDialog } from "@/features/profile-transfer/presentation/components/import-profile-dialog";

/** Spreadsheet export (.ods) of the active profile's data. */
export function ExportSection({ profileId }: { profileId: string }) {
  const { t } = useTranslation();
  const [exportOpen, setExportOpen] = useState(false);

  return (
    <>
      <div className="flex flex-col gap-3">
        <h3 className="text-base font-medium">{t("export.title")}</h3>
        <p className="text-sm text-muted-foreground">
          {t("export.description")}
        </p>
        <div>
          <Button onClick={() => setExportOpen(true)}>
            <Download />
            {t("export.menuItem")}
          </Button>
        </div>
      </div>
      <ExportDialog
        open={exportOpen}
        onOpenChange={setExportOpen}
        profileId={profileId}
      />
    </>
  );
}

/** Profile transfer (shareable .json): export the whole profile, or import
 *  one (into a new profile or merged into an existing one). */
export function TransferSection({ profileId }: { profileId: string }) {
  const { t } = useTranslation();
  const [exportBundleOpen, setExportBundleOpen] = useState(false);
  const [importBundleOpen, setImportBundleOpen] = useState(false);

  return (
    <>
      <div className="flex flex-col gap-3">
        <h3 className="text-base font-medium">{t("transfer.section")}</h3>
        <p className="text-sm text-muted-foreground">
          {t("transfer.sectionHint")}
        </p>
        <div className="flex flex-wrap gap-2">
          <Button variant="outline" onClick={() => setExportBundleOpen(true)}>
            {t("transfer.export.menuItem")}
          </Button>
          <Button variant="outline" onClick={() => setImportBundleOpen(true)}>
            {t("transfer.import.menuItem")}
          </Button>
        </div>
      </div>
      <ExportProfileDialog
        open={exportBundleOpen}
        onOpenChange={setExportBundleOpen}
        profileId={profileId}
      />
      <ImportProfileDialog
        open={importBundleOpen}
        onOpenChange={setImportBundleOpen}
      />
    </>
  );
}
