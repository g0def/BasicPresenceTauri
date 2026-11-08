//! Category selection of the profile-bundle export/import wizards (ported
//! from `features/profile-transfer/domain/entities/bundle.ts`).

use crate::application::dto::profile_bundle_dto::{
    BundleExportOptionsDto, BundleImportSelectionDto, BundleManifestDto,
};

/// A bundle category. `Trips`, `WorkHours` and `Notes` are children of `Days`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BundleCategory {
    Days,
    Trips,
    WorkHours,
    Notes,
    TaskPresets,
    Commutes,
    Settings,
}

impl BundleCategory {
    /// Every category, in display order.
    pub const ALL: [BundleCategory; 7] = [
        BundleCategory::Days,
        BundleCategory::Trips,
        BundleCategory::WorkHours,
        BundleCategory::Notes,
        BundleCategory::TaskPresets,
        BundleCategory::Commutes,
        BundleCategory::Settings,
    ];

    /// Whether the category only makes sense alongside the presence days.
    pub fn is_day_child(self) -> bool {
        matches!(
            self,
            BundleCategory::Trips | BundleCategory::WorkHours | BundleCategory::Notes
        )
    }
}

/// Which categories to include. Defaults to everything.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BundleSelection {
    pub days: bool,
    pub trips: bool,
    pub work_hours: bool,
    pub notes: bool,
    pub task_presets: bool,
    pub commutes: bool,
    pub settings: bool,
}

impl Default for BundleSelection {
    fn default() -> Self {
        Self::all(true)
    }
}

impl BundleSelection {
    pub fn all(value: bool) -> Self {
        Self {
            days: value,
            trips: value,
            work_hours: value,
            notes: value,
            task_presets: value,
            commutes: value,
            settings: value,
        }
    }

    pub fn get(&self, category: BundleCategory) -> bool {
        match category {
            BundleCategory::Days => self.days,
            BundleCategory::Trips => self.trips,
            BundleCategory::WorkHours => self.work_hours,
            BundleCategory::Notes => self.notes,
            BundleCategory::TaskPresets => self.task_presets,
            BundleCategory::Commutes => self.commutes,
            BundleCategory::Settings => self.settings,
        }
    }

    pub fn set(&mut self, category: BundleCategory, value: bool) {
        let field = match category {
            BundleCategory::Days => &mut self.days,
            BundleCategory::Trips => &mut self.trips,
            BundleCategory::WorkHours => &mut self.work_hours,
            BundleCategory::Notes => &mut self.notes,
            BundleCategory::TaskPresets => &mut self.task_presets,
            BundleCategory::Commutes => &mut self.commutes,
            BundleCategory::Settings => &mut self.settings,
        };
        *field = value;
    }

    /// Children are dropped when the days are not included.
    pub fn effective(self) -> Self {
        if self.days {
            self
        } else {
            Self {
                trips: false,
                work_hours: false,
                notes: false,
                ..self
            }
        }
    }

    /// Initial import selection: exactly the categories the file contains.
    pub fn from_manifest(manifest: &BundleManifestDto) -> Self {
        let mut selection = Self::all(false);
        for (category, _) in present_categories(manifest) {
            selection.set(category, true);
        }
        selection
    }
}

/// The categories a bundle file actually contains, with their item counts
/// (`None` for the settings, a single row). Absent categories are omitted.
pub fn present_categories(m: &BundleManifestDto) -> Vec<(BundleCategory, Option<u32>)> {
    let counted = [
        (BundleCategory::Days, m.days),
        (BundleCategory::Trips, m.trips),
        (BundleCategory::WorkHours, m.work_entries),
        (BundleCategory::Notes, m.notes),
        (BundleCategory::TaskPresets, m.task_presets),
        (BundleCategory::Commutes, m.commutes),
    ];
    let mut out: Vec<_> = counted
        .into_iter()
        .filter(|(_, n)| *n > 0)
        .map(|(c, n)| (c, Some(n)))
        .collect();
    if m.has_settings {
        out.push((BundleCategory::Settings, None));
    }
    out
}

impl From<BundleSelection> for BundleExportOptionsDto {
    fn from(s: BundleSelection) -> Self {
        let s = s.effective();
        Self {
            include_days: s.days,
            include_trips: s.trips,
            include_work_hours: s.work_hours,
            include_notes: s.notes,
            include_task_presets: s.task_presets,
            include_commutes: s.commutes,
            include_settings: s.settings,
        }
    }
}

impl From<BundleSelection> for BundleImportSelectionDto {
    fn from(s: BundleSelection) -> Self {
        let s = s.effective();
        Self {
            include_days: s.days,
            include_trips: s.trips,
            include_work_hours: s.work_hours,
            include_notes: s.notes,
            include_task_presets: s.task_presets,
            include_commutes: s.commutes,
            include_settings: s.settings,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest() -> BundleManifestDto {
        BundleManifestDto {
            version: 1,
            compatible: true,
            exported_at: 0,
            app: "BasicPresence".into(),
            profile_first_name: "Ada".into(),
            profile_last_name: "Lovelace".into(),
            profile_enterprise: "AE".into(),
            profile_poste: None,
            days: 10,
            trips: 0,
            work_entries: 4,
            notes: 0,
            task_presets: 2,
            commutes: 0,
            has_settings: true,
            unknown_mode_ids: vec![],
        }
    }

    #[test]
    fn present_categories_lists_only_what_the_file_holds() {
        assert_eq!(
            present_categories(&manifest()),
            [
                (BundleCategory::Days, Some(10)),
                (BundleCategory::WorkHours, Some(4)),
                (BundleCategory::TaskPresets, Some(2)),
                (BundleCategory::Settings, None),
            ]
        );
    }

    #[test]
    fn selection_from_manifest_ticks_present_categories() {
        let s = BundleSelection::from_manifest(&manifest());
        assert!(s.days && s.work_hours && s.task_presets && s.settings);
        assert!(!s.trips && !s.notes && !s.commutes);
    }

    #[test]
    fn effective_drops_day_children_without_days() {
        let mut s = BundleSelection::default();
        assert_eq!(s.effective(), s);
        s.set(BundleCategory::Days, false);
        let e = s.effective();
        assert!(!e.trips && !e.work_hours && !e.notes);
        assert!(e.task_presets && e.commutes && e.settings);
        let dto: BundleExportOptionsDto = s.into();
        assert!(!dto.include_trips && dto.include_settings);
    }

    #[test]
    fn categories_get_set_and_children() {
        let mut s = BundleSelection::all(false);
        for c in BundleCategory::ALL {
            assert!(!s.get(c));
            s.set(c, true);
            assert!(s.get(c));
        }
        let children: Vec<_> = BundleCategory::ALL
            .into_iter()
            .filter(|c| c.is_day_child())
            .collect();
        assert_eq!(
            children,
            [
                BundleCategory::Trips,
                BundleCategory::WorkHours,
                BundleCategory::Notes
            ]
        );
    }
}
