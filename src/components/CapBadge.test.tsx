import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { CapBadge } from "./CapBadge";

describe("CapBadge", () => {
  it("renders nothing when under cap", () => {
    const { container } = render(
      <CapBadge
        maxFocuses={5}
        capState={{
          focusesOver: false,
          focusCount: 3,
          overTaskFocusIds: [],
          anyOver: false,
        }}
      />,
    );
    expect(container).toBeEmptyDOMElement();
  });

  it("shows focuses-over message when over", () => {
    render(
      <CapBadge
        maxFocuses={5}
        capState={{
          focusesOver: true,
          focusCount: 6,
          overTaskFocusIds: [],
          anyOver: true,
        }}
      />,
    );
    expect(screen.getByTestId("cap-badge")).toBeInTheDocument();
    expect(screen.getByTestId("cap-badge-focuses")).toHaveTextContent(
      "6 focuses (max 5) — trim one",
    );
  });

  it("lists task-over focus ids", () => {
    render(
      <CapBadge
        maxFocuses={5}
        capState={{
          focusesOver: false,
          focusCount: 2,
          overTaskFocusIds: ["bad"],
          anyOver: true,
        }}
      />,
    );
    expect(screen.getByTestId("cap-badge-tasks")).toHaveTextContent("tasks over: bad");
  });
});
