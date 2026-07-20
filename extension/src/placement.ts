export type InsertionPosition =
  | "beginning"
  | "before_selected"
  | "after_selected"
  | "end";

export function sdkInsertionIndex(
  position: InsertionPosition,
  deviceCount: number,
): number {
  if (position === "beginning") return 0;
  if (position === "end") return deviceCount;
  throw new Error(
    "The official Extensions SDK does not expose the selected device; use the Live bridge for before/after placement",
  );
}
