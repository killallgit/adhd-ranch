export function useViewport() {
  return {
    screenW: document.documentElement.clientWidth || window.screen.width,
    screenH: document.documentElement.clientHeight || window.screen.height,
  };
}
