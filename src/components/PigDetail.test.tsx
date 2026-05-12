import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useState } from "react";
import { describe, expect, it, vi } from "vitest";
import type { Focus } from "../types/focus";
import { PigDetail } from "./PigDetail";

const baseFocus: Focus = {
  id: "pig-a",
  title: "Ship it",
  description: "",
  created_at: "",
  tasks: [],
};

function renderDetail(overrides?: Partial<React.ComponentProps<typeof PigDetail>>) {
  const props = {
    focus: baseFocus,
    pigX: 100,
    pigY: 100,
    viewportW: 1920,
    viewportH: 1080,
    confirmDelete: true,
    onClose: vi.fn(),
    onClearTask: vi.fn(),
    onAddTask: vi.fn(),
    onRenameFocus: vi.fn(),
    onUpdateTask: vi.fn(),
    onToggleTask: vi.fn(),
    onDeleteFocus: vi.fn(),
    onStartTimer: vi.fn(),
    ...overrides,
  };
  render(<PigDetail {...props} />);
  return props;
}

describe("PigDetail add-task input", () => {
  it("renders an add task input with placeholder", () => {
    renderDetail();
    expect(screen.getByPlaceholderText("Add task…")).toBeInTheDocument();
  });

  it("Enter calls onAddTask with trimmed text", async () => {
    const { onAddTask } = renderDetail();
    const input = screen.getByPlaceholderText("Add task…");
    await userEvent.type(input, "fix the thing{Enter}");
    expect(onAddTask).toHaveBeenCalledWith("fix the thing");
  });

  it("Enter clears the input field", async () => {
    renderDetail();
    const input = screen.getByPlaceholderText("Add task…");
    await userEvent.type(input, "some task{Enter}");
    expect(input).toHaveValue("");
  });

  it("Enter with empty text does not call onAddTask", async () => {
    const { onAddTask } = renderDetail();
    const input = screen.getByPlaceholderText("Add task…");
    await userEvent.type(input, "   {Enter}");
    expect(onAddTask).not.toHaveBeenCalled();
  });

  it("Escape closes the card", async () => {
    const { onClose } = renderDetail();
    await userEvent.keyboard("{Escape}");
    expect(onClose).toHaveBeenCalled();
  });
});

describe("PigDetail title editing", () => {
  it("renders title as editable input", () => {
    renderDetail();
    expect(screen.getByLabelText("focus title")).toHaveValue("Ship it");
  });

  it("commits new title on blur", async () => {
    const { onRenameFocus } = renderDetail();
    const input = screen.getByLabelText("focus title");
    await userEvent.clear(input);
    await userEvent.type(input, "New Title");
    input.blur();
    expect(onRenameFocus).toHaveBeenCalledWith("pig-a", "New Title");
  });

  it("empty title reverts and shows error", async () => {
    const { onRenameFocus } = renderDetail();
    const input = screen.getByLabelText("focus title") as HTMLInputElement;
    await userEvent.clear(input);
    await userEvent.tab();
    expect(onRenameFocus).not.toHaveBeenCalled();
    expect(await screen.findByText("Title cannot be empty")).toBeInTheDocument();
  });
});

describe("PigDetail task editing", () => {
  const focusWithTasks: Focus = {
    id: "pig-a",
    title: "Ship it",
    description: "",
    created_at: "",
    tasks: [
      { id: "t1", text: "alpha", done: false },
      { id: "t2", text: "beta", done: true },
    ],
  };

  it("renders each task as an editable input", () => {
    renderDetail({ focus: focusWithTasks });
    expect(screen.getByLabelText("task text: alpha")).toHaveValue("alpha");
    expect(screen.getByLabelText("task text: beta")).toHaveValue("beta");
  });

  it("commits new task text on Enter", async () => {
    const { onUpdateTask } = renderDetail({ focus: focusWithTasks });
    const input = screen.getByLabelText("task text: alpha");
    await userEvent.clear(input);
    await userEvent.type(input, "alpha-renamed{Enter}");
    expect(onUpdateTask).toHaveBeenCalledWith("pig-a", 0, "alpha-renamed");
  });

  it("checkbox toggle calls onToggleTask", async () => {
    const { onToggleTask } = renderDetail({ focus: focusWithTasks });
    await userEvent.click(screen.getByLabelText("toggle task: alpha"));
    expect(onToggleTask).toHaveBeenCalledWith("pig-a", 0, true);
  });

  it("done task checkbox starts checked", () => {
    renderDetail({ focus: focusWithTasks });
    expect(screen.getByLabelText("toggle task: beta")).toBeChecked();
  });
});

describe("PigDetail delete focus", () => {
  it("with confirmDelete=true shows inline confirm before deleting", async () => {
    const { onDeleteFocus } = renderDetail({ confirmDelete: true });
    await userEvent.click(screen.getByLabelText("delete focus Ship it"));
    expect(onDeleteFocus).not.toHaveBeenCalled();
    expect(screen.getByTestId("pig-detail-delete-confirm")).toBeInTheDocument();
    await userEvent.click(screen.getByText("Delete"));
    expect(onDeleteFocus).toHaveBeenCalledWith("pig-a");
  });

  it("with confirmDelete=true Cancel hides confirm without deleting", async () => {
    const { onDeleteFocus } = renderDetail({ confirmDelete: true });
    await userEvent.click(screen.getByLabelText("delete focus Ship it"));
    await userEvent.click(screen.getByText("Cancel"));
    expect(screen.queryByTestId("pig-detail-delete-confirm")).not.toBeInTheDocument();
    expect(onDeleteFocus).not.toHaveBeenCalled();
  });

  it("with confirmDelete=false deletes immediately", async () => {
    const { onDeleteFocus, onClose } = renderDetail({ confirmDelete: false });
    await userEvent.click(screen.getByLabelText("delete focus Ship it"));
    expect(onDeleteFocus).toHaveBeenCalledWith("pig-a");
    expect(onClose).toHaveBeenCalled();
  });
});

describe("PigDetail timer picker", () => {
  it("renders Start button + preset select", () => {
    renderDetail();
    expect(screen.getByRole("button", { name: "Start" })).toBeInTheDocument();
    expect(screen.getByTestId("timer-preset-select")).toBeInTheDocument();
  });

  it("Start with named preset calls onStartTimer with preset literal", async () => {
    const { onStartTimer } = renderDetail();
    await userEvent.selectOptions(screen.getByTestId("timer-preset-select"), "ThirtyTwo");
    await userEvent.click(screen.getByRole("button", { name: "Start" }));
    expect(onStartTimer).toHaveBeenCalledWith("pig-a", "ThirtyTwo");
  });

  it("Start with custom preset wraps minutes", async () => {
    const { onStartTimer } = renderDetail();
    await userEvent.selectOptions(screen.getByTestId("timer-preset-select"), "custom");
    const input = screen.getByTestId("custom-timer-input");
    await userEvent.clear(input);
    await userEvent.type(input, "25");
    await userEvent.click(screen.getByRole("button", { name: "Start" }));
    expect(onStartTimer).toHaveBeenCalledWith("pig-a", { Custom: 25 });
  });

  it("rejects custom < 1 minute", async () => {
    const { onStartTimer } = renderDetail();
    await userEvent.selectOptions(screen.getByTestId("timer-preset-select"), "custom");
    const input = screen.getByTestId("custom-timer-input");
    await userEvent.clear(input);
    await userEvent.type(input, "0");
    await userEvent.click(screen.getByRole("button", { name: "Start" }));
    expect(onStartTimer).not.toHaveBeenCalled();
    expect(screen.getByText(/at least 1 minute/i)).toBeInTheDocument();
  });

  it("button reads Restart when timer is Running", () => {
    renderDetail({
      focus: {
        ...baseFocus,
        timer: { duration_secs: 600, started_at: 0, status: "Running" },
      },
    });
    expect(screen.getByRole("button", { name: "Restart" })).toBeInTheDocument();
  });

  it("resets picker state when focus changes", async () => {
    function Harness() {
      const [focusId, setFocusId] = useState("pig-a");
      return (
        <>
          <button type="button" data-testid="swap" onClick={() => setFocusId("pig-b")}>
            swap
          </button>
          <PigDetail
            focus={{ ...baseFocus, id: focusId }}
            pigX={100}
            pigY={100}
            viewportW={1920}
            viewportH={1080}
            confirmDelete={true}
            onClose={vi.fn()}
            onClearTask={vi.fn()}
            onAddTask={vi.fn()}
            onRenameFocus={vi.fn()}
            onUpdateTask={vi.fn()}
            onToggleTask={vi.fn()}
            onDeleteFocus={vi.fn()}
            onStartTimer={vi.fn()}
          />
        </>
      );
    }
    render(<Harness />);

    await userEvent.selectOptions(screen.getByTestId("timer-preset-select"), "Sixteen");
    expect(screen.getByTestId("timer-preset-select")).toHaveValue("Sixteen");

    await userEvent.click(screen.getByTestId("swap"));

    expect(screen.getByTestId("timer-preset-select")).toHaveValue("Eight");
  });
});
