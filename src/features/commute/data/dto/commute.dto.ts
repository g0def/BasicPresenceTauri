/** Wire shapes returned by the Rust commute commands (serialized camelCase). */

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
}

export interface CommuteSegmentDto {
  id: string;
  modeId: string;
  distanceKm: number;
  occupants: number;
  position: number;
}

export interface CommuteDto {
  id: string;
  profileId: string;
  name: string;
  roundTrip: boolean;
  segments: CommuteSegmentDto[];
  co2Kg: number | null;
  createdAt: number;
  updatedAt: number;
}
