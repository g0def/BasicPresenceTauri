import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import type {
  CommuteSegment,
  EmissionFactor,
} from "@/features/commute/domain/entities/commute";
import { SegmentEditor } from "@/features/commute/presentation/components/segment-editor";

const FACTORS: EmissionFactor[] = [
  {
    modeId: "car_petrol",
    label: "Voiture — essence",
    category: "car",
    unit: "kgCO2e/veh.km",
    isParam: false,
  },
  {
    modeId: "train_sncb",
    label: "Train Intercity (SNCB)",
    category: "rail",
    unit: "kgCO2e/passenger.km",
    isParam: false,
  },
];

describe("SegmentEditor", () => {
  it("shows the occupants field only for car-category modes", () => {
    const { rerender } = render(
      <SegmentEditor
        segments={[{ modeId: "car_petrol", distanceKm: 10, occupants: 2 }]}
        onChange={() => {}}
        emissionFactors={FACTORS}
      />,
    );
    expect(screen.getByText("Occupants")).toBeInTheDocument();

    // A passenger.km mode (rail) hides occupants.
    rerender(
      <SegmentEditor
        segments={[{ modeId: "train_sncb", distanceKm: 30, occupants: 1 }]}
        onChange={() => {}}
        emissionFactors={FACTORS}
      />,
    );
    expect(screen.queryByText("Occupants")).not.toBeInTheDocument();
  });

  it("appends a blank segment when clicking add", async () => {
    const onChange = vi.fn<(next: CommuteSegment[]) => void>();
    const segments: CommuteSegment[] = [
      { modeId: "car_petrol", distanceKm: 10, occupants: 1 },
    ];
    const user = userEvent.setup();
    render(
      <SegmentEditor
        segments={segments}
        onChange={onChange}
        emissionFactors={FACTORS}
      />,
    );

    await user.click(
      screen.getByRole("button", { name: "Ajouter un segment" }),
    );

    expect(onChange).toHaveBeenCalledWith([
      ...segments,
      { modeId: "", distanceKm: 0, occupants: 1 },
    ]);
  });
});
