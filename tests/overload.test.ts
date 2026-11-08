/**
 * A finer detail the device can't sustain steps down (ultra → high → standard) after a second of
 * render quanta over budget; a brief spike, or load under the threshold, changes nothing; the
 * standard detail is never stepped; anatomy is kept.
 */
import { describe, expect, test } from "bun:test";
import { OVERLOAD, OverloadGuard } from "../src/worklet/overload.ts";
import type { Detail } from "../src/wasm/types.ts";

const SECOND = 375; // render quanta of 128 samples at 48 kHz
const ULTRA: Detail = { resolution: "ultra", anatomy: true };

function run(g: OverloadGuard, load: number, detail: Detail, quanta: number): { at: number; to: Detail } | null {
  for (let i = 0; i < quanta; i++) {
    const to = g.observe(load, detail);
    if (to) return { at: i + 1, to };
  }
  return null;
}

describe("overload guard", () => {
  test("a second over budget steps the resolution down, keeping the anatomy", () => {
    const g = new OverloadGuard(SECOND);
    const first = run(g, 1.3, ULTRA, 2 * SECOND);
    expect(first).toEqual({ at: SECOND, to: { resolution: "high", anatomy: true } });
    const second = run(g, 1.1, first!.to, 2 * SECOND);
    expect(second).toEqual({ at: SECOND, to: { resolution: "standard", anatomy: true } });
    // the standard detail stays, however heavy
    expect(run(g, 2.0, second!.to, 3 * SECOND)).toBeNull();
  });

  test("load under the threshold, or a spike shorter than the window, changes nothing", () => {
    const g = new OverloadGuard(SECOND);
    expect(run(g, OVERLOAD - 0.01, ULTRA, 3 * SECOND)).toBeNull();
    for (let i = 0; i < 4; i++) {
      expect(run(g, 1.5, ULTRA, SECOND - 1)).toBeNull(); // nearly a second over…
      expect(run(g, 0.5, ULTRA, 1)).toBeNull(); // …then one quantum under: the count starts again
    }
    g.reset();
    expect(run(g, 1.5, ULTRA, SECOND)?.at).toBe(SECOND);
  });
});
