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

  it("calls onCreate with trimmed title and description", async () => {
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
