import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { NewFocusForm } from "./NewFocusForm";

describe("NewFocusForm", () => {
  it("starts collapsed and expands on click", () => {
    render(<NewFocusForm onCreate={() => Promise.resolve()} />);
    expect(screen.queryByTestId("new-focus-form")).not.toBeInTheDocument();
    fireEvent.click(screen.getByTestId("new-focus-toggle"));
    expect(screen.getByTestId("new-focus-form")).toBeInTheDocument();
  });

  it("blocks submit when title is empty", async () => {
    const onCreate = vi.fn();
    render(<NewFocusForm onCreate={onCreate} />);
    fireEvent.click(screen.getByTestId("new-focus-toggle"));
    fireEvent.click(screen.getByText("Create"));
    await waitFor(() => {
      expect(screen.getByTestId("new-focus-error")).toHaveTextContent("title is required");
    });
    expect(onCreate).not.toHaveBeenCalled();
  });

  it("calls onCreate with trimmed title, description, and null timer by default", async () => {
    const onCreate = vi.fn().mockResolvedValue(undefined);
    render(<NewFocusForm onCreate={onCreate} />);
    fireEvent.click(screen.getByTestId("new-focus-toggle"));
    fireEvent.change(screen.getByLabelText("new focus title"), {
      target: { value: "  Customer X bug  " },
    });
    fireEvent.change(screen.getByLabelText("new focus description"), {
      target: { value: "  ship  " },
    });
    fireEvent.click(screen.getByText("Create"));
    await waitFor(() => {
      expect(onCreate).toHaveBeenCalledWith({
        title: "Customer X bug",
        description: "ship",
        timer_preset: null,
      });
    });
  });

  it("passes selected preset to onCreate", async () => {
    const onCreate = vi.fn().mockResolvedValue(undefined);
    render(<NewFocusForm onCreate={onCreate} />);
    fireEvent.click(screen.getByTestId("new-focus-toggle"));
    fireEvent.change(screen.getByLabelText("new focus title"), {
      target: { value: "Timer focus" },
    });
    fireEvent.change(screen.getByTestId("timer-preset-select"), {
      target: { value: "Eight" },
    });
    fireEvent.click(screen.getByText("Create"));
    await waitFor(() => {
      expect(onCreate).toHaveBeenCalledWith({
        title: "Timer focus",
        description: "",
        timer_preset: "Eight",
      });
    });
  });

  it("shows custom input when custom preset selected", () => {
    render(<NewFocusForm onCreate={() => Promise.resolve()} />);
    fireEvent.click(screen.getByTestId("new-focus-toggle"));
    expect(screen.queryByTestId("custom-timer-input")).not.toBeInTheDocument();
    fireEvent.change(screen.getByTestId("timer-preset-select"), {
      target: { value: "custom" },
    });
    expect(screen.getByTestId("custom-timer-input")).toBeInTheDocument();
  });

  it("passes Custom preset with minutes to onCreate", async () => {
    const onCreate = vi.fn().mockResolvedValue(undefined);
    render(<NewFocusForm onCreate={onCreate} />);
    fireEvent.click(screen.getByTestId("new-focus-toggle"));
    fireEvent.change(screen.getByLabelText("new focus title"), {
      target: { value: "Custom timer" },
    });
    fireEvent.change(screen.getByTestId("timer-preset-select"), {
      target: { value: "custom" },
    });
    fireEvent.change(screen.getByTestId("custom-timer-input"), {
      target: { value: "45" },
    });
    fireEvent.click(screen.getByText("Create"));
    await waitFor(() => {
      expect(onCreate).toHaveBeenCalledWith({
        title: "Custom timer",
        description: "",
        timer_preset: { Custom: 45 },
      });
    });
  });

  it("surfaces async errors inline", async () => {
    const onCreate = vi.fn().mockRejectedValue(new Error("focus already exists"));
    render(<NewFocusForm onCreate={onCreate} />);
    fireEvent.click(screen.getByTestId("new-focus-toggle"));
    fireEvent.change(screen.getByLabelText("new focus title"), {
      target: { value: "Customer X bug" },
    });
    fireEvent.click(screen.getByText("Create"));
    await waitFor(() => {
      expect(screen.getByTestId("new-focus-error")).toHaveTextContent("focus already exists");
    });
  });
});
