import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { DayKeyField } from "@/features/export/presentation/components/day-key-field";

const NOV_1 = Date.UTC(2026, 10, 1);

describe("DayKeyField", () => {
  it("shows the day as dd/mm/yyyy", () => {
    render(<DayKeyField id="d" value={NOV_1} onChange={vi.fn()} />);
    expect(screen.getByRole("textbox")).toHaveValue("01/11/2026");
  });

  it("commits a valid date on blur", () => {
    const onChange = vi.fn();
    render(<DayKeyField id="d" value={NOV_1} onChange={onChange} />);
    const input = screen.getByRole("textbox");
    fireEvent.change(input, { target: { value: "30/11/2026" } });
    fireEvent.blur(input);
    expect(onChange).toHaveBeenCalledWith(Date.UTC(2026, 10, 30));
  });

  it("reverts an invalid entry without committing", () => {
    const onChange = vi.fn();
    render(<DayKeyField id="d" value={NOV_1} onChange={onChange} />);
    const input = screen.getByRole("textbox");
    fireEvent.change(input, { target: { value: "11/31/2026" } });
    fireEvent.keyDown(input, { key: "Enter" });
    expect(onChange).not.toHaveBeenCalled();
    expect(input).toHaveValue("01/11/2026");
  });
});
