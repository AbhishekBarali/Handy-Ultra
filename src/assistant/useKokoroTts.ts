import { useCallback, useEffect, useRef, useState } from "react";

export type TtsStatus = "off" | "loading" | "ready" | "speaking" | "error";

/** Minimal surface of the kokoro-js model we use (erases its strict voice
 *  union type so the voice id can come from settings). */
interface KokoroModel {
  generate(
    text: string,
    options: { voice?: string },
  ): Promise<{ toBlob(): Blob }>;
}

const KOKORO_MODEL_ID = "onnx-community/Kokoro-82M-v1.0-ONNX";

/**
 * Lazy-loaded local TTS via kokoro-js (82M params, q8, WASM). The model is
 * only downloaded/initialized once TTS is enabled; audio is fully local.
 */
export function useKokoroTts(enabled: boolean, voice: string) {
  const modelRef = useRef<KokoroModel | null>(null);
  const loadingRef = useRef<Promise<KokoroModel> | null>(null);
  const audioRef = useRef<HTMLAudioElement | null>(null);
  const [status, setStatus] = useState<TtsStatus>("off");

  const ensureLoaded = useCallback(async (): Promise<KokoroModel> => {
    if (modelRef.current) return modelRef.current;
    if (!loadingRef.current) {
      setStatus("loading");
      loadingRef.current = (async () => {
        const { KokoroTTS } = await import("kokoro-js");
        const model = (await KokoroTTS.from_pretrained(KOKORO_MODEL_ID, {
          dtype: "q8",
          device: "wasm",
        })) as unknown as KokoroModel;
        modelRef.current = model;
        setStatus("ready");
        return model;
      })().catch((e: unknown) => {
        loadingRef.current = null;
        setStatus("error");
        throw e;
      });
    }
    return loadingRef.current;
  }, []);

  // Preload as soon as TTS is switched on so the first answer speaks fast.
  useEffect(() => {
    if (enabled) {
      ensureLoaded().catch(() => {});
    } else {
      setStatus((s) => (s === "speaking" || s === "ready" ? "off" : s));
    }
  }, [enabled, ensureLoaded]);

  const stop = useCallback(() => {
    const el = audioRef.current;
    if (el) {
      el.pause();
      audioRef.current = null;
    }
    setStatus((s) => (s === "speaking" ? "ready" : s));
  }, []);

  const speak = useCallback(
    async (text: string) => {
      if (!enabled || !text.trim()) return;
      try {
        const model = await ensureLoaded();
        const audio = await model.generate(text, { voice });
        stop();
        const url = URL.createObjectURL(audio.toBlob());
        const el = new Audio(url);
        audioRef.current = el;
        setStatus("speaking");
        const finish = () => {
          URL.revokeObjectURL(url);
          if (audioRef.current === el) {
            audioRef.current = null;
          }
          setStatus((s) => (s === "speaking" ? "ready" : s));
        };
        el.onended = finish;
        el.onerror = finish;
        await el.play();
      } catch (e) {
        console.error("Kokoro TTS failed:", e);
        setStatus("error");
      }
    },
    [enabled, voice, ensureLoaded, stop],
  );

  return { status, speak, stop };
}
