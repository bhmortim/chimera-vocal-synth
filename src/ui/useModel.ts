import { useEffect, useRef } from "react";
import { VoiceModel } from "./analysis.ts";
import { useStore } from "./store.ts";
import { engine } from "../audio/AudioEngine.ts";
import { bodyKey, type BodyMix } from "./body.ts";
import type { Detail } from "../wasm/types.ts";
import { detailKey, STANDARD_DETAIL } from "../wasm/detail.ts";

/**
 * Shared main-thread acoustic models, most recent last (rebuilt on body / detail / sample-rate
 * change). A body is continuous, so only the last few are kept; an evicted one is freed in the
 * engine too (components read the current body's model on every render, so none holds an evicted
 * one).
 */
const cache = new Map<string, VoiceModel>();
const KEEP = 8;

export function getModel(body: BodyMix, sampleRate: number, detail: Detail = STANDARD_DETAIL): VoiceModel {
  const key = `${bodyKey(body)}@${sampleRate}/${detailKey(detail)}`;
  let m = cache.get(key);
  if (m) cache.delete(key);
  else m = new VoiceModel(body, sampleRate, detail);
  cache.set(key, m);
  if (cache.size > KEEP) {
    const [oldest, old] = cache.entries().next().value!;
    cache.delete(oldest);
    old.dispose();
  }
  return m;
}

export function useModel(): React.MutableRefObject<VoiceModel | null> {
  const body = useStore((s) => s.body);
  const detail = useStore((s) => s.detail);
  const started = useStore((s) => s.started);
  const ref = useRef<VoiceModel | null>(null);
  ref.current = getModel(body, started ? engine.sampleRate : 48000, detail);
  useEffect(() => {
    ref.current = getModel(body, started ? engine.sampleRate : 48000, detail);
  }, [body, detail, started]);
  return ref;
}
