export function useViewport() {
  return {
    screenW: window.innerWidth || window.screen.width,
    screenH: window.innerHeight || window.screen.height,
  };
}
