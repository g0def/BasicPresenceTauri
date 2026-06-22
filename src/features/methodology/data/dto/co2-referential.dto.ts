/** Wire shapes returned by the `list_co2_referential` command (camelCase). */

export interface GridVariantDto {
  country: string;
  value: number;
}

export interface EmissionFactorDto {
  modeId: string;
  label: string;
  value: number;
  unit: string;
  category: string;
  isParam: boolean;
  gridVariants: GridVariantDto[];
  scope: string | null;
  source: string | null;
}

export interface Co2ReferentialDto {
  factorYear: number;
  radiativeForcing: number;
  factors: EmissionFactorDto[];
}
