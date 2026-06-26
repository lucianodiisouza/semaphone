export const DRAG_THRESHOLD_PX = 5;

export function shouldOpenSettingsOnDoubleClick(
  movedPx: number,
  threshold = DRAG_THRESHOLD_PX,
): boolean {
  return movedPx < threshold;
}

export function pointerDistance(
  x1: number,
  y1: number,
  x2: number,
  y2: number,
): number {
  const dx = x2 - x1;
  const dy = y2 - y1;
  return Math.sqrt(dx * dx + dy * dy);
}
