/** Wire shape of profile settings (flat camelCase, matches the Rust DTO). */
export interface ProfileSettingsDto {
  defaultStartMinutes: number;
  noteFont: string;
  cellDisplayMode: string;
  gridCountry: string;
  defaultCarOccupancy: number;
  includeRadiativeForcing: boolean;
  countBuildingEnergy: boolean;
  workingDaysPerYear: number;
  factorYear: number;
}
