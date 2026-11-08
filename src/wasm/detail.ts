import type { Detail, Resolution } from "./types.ts";

/** The calibrated model: the default voice. */
export const STANDARD_DETAIL: Detail = { resolution: "standard", anatomy: false };

/** The C ABI's encoding of a resolution (engine_set_detail). */
export const RESOLUTION_LEVEL: Record<Resolution, number> = { standard: 0, high: 1, ultra: 2 };

export const sameDetail = (a: Detail, b: Detail): boolean => a.resolution === b.resolution && a.anatomy === b.anatomy;

export const detailKey = (d: Detail): string => `${d.resolution}${d.anatomy ? "+anatomy" : ""}`;
