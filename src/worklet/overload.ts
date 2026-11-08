import type { Detail, Resolution } from "../wasm/types.ts";

/** Above this share of the render budget a quantum counts as strained. */
export const OVERLOAD = 0.9;

const LIGHTER: Record<Resolution, Resolution> = { ultra: "high", high: "standard", standard: "standard" };

/**
 * Steps a finer detail down when the device can't keep up with it. Over budget, the audio thread
 * underruns (the sound crackles and clips, then gives out), so after `window` strained render
 * quanta in a row this returns the next lighter resolution (ultra → high → standard), keeping the
 * anatomy. The standard detail is never stepped.
 */
export class OverloadGuard {
  private strained = 0;

  constructor(private readonly window: number) {}

  /** Forget the strain so far (a new voice). */
  reset(): void {
    this.strained = 0;
  }

  /** One render quantum at this (smoothed) load: the detail to step down to, or null. */
  observe(load: number, detail: Detail): Detail | null {
    if (detail.resolution === "standard") {
      this.strained = 0;
      return null;
    }
    this.strained = load > OVERLOAD ? this.strained + 1 : 0;
    if (this.strained < this.window) return null;
    this.strained = 0;
    return { ...detail, resolution: LIGHTER[detail.resolution] };
  }
}
