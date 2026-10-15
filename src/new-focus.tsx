import React from "react";
import ReactDOM from "react-dom/client";
import { createTauriFocusWriter } from "./api/focusWriter";
import { NewFocusWindow } from "./components/NewFocusWindow";
import { useNewFocusWindow } from "./hooks/useNewFocusWindow";
import "./styles.css";

const focusWriter = createTauriFocusWriter();

function NewFocusApp() {
  const {
    title,
    description,
    selectedMinutes,
    savedTimers,
    submitting,
    error,
    setTitle,
    setDescription,
    setSelectedMinutes,
    handleSubmit,
    handleCancel,
  } = useNewFocusWindow(focusWriter);

  return (
    <NewFocusWindow
      title={title}
      description={description}
      selectedMinutes={selectedMinutes}
      savedTimers={savedTimers}
      submitting={submitting}
      error={error}
      onTitleChange={setTitle}
      onDescriptionChange={setDescription}
      onSelectMinutes={setSelectedMinutes}
      onSubmit={handleSubmit}
      onCancel={() => void handleCancel()}
    />
  );
}

const rootEl = document.getElementById("root");
if (!rootEl) throw new Error("missing #root");

ReactDOM.createRoot(rootEl).render(
  <React.StrictMode>
    <NewFocusApp />
  </React.StrictMode>,
);
