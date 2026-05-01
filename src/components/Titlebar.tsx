interface TitlebarProps {
  onClose: () => void;
}

export function Titlebar({ onClose }: TitlebarProps) {
  return (
    <div className="titlebar" data-tauri-drag-region>
      <button type="button" className="titlebar-close" onClick={onClose} aria-label="Close">
        ×
      </button>
    </div>
  );
}
