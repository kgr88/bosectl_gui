import { useEffect, useMemo, useRef, useState } from "react";
import { api } from "../api";
import type { DeviceSnapshot } from "../contracts";
import { LatestWriteQueue } from "../state/latestWrite";
import {
  card,
  eyebrow,
  heading,
  muted,
  secondaryButton,
  selectedButton,
} from "../styles/ui";

interface Props {
  snapshot: DeviceSnapshot;
  onSnapshot: (snapshot: DeviceSnapshot) => void;
  onError: (error: unknown) => void;
}

export function HomePage({ snapshot, onSnapshot, onError }: Props) {
  const capability = snapshot.capabilities.noise_control;
  const rawValue = snapshot.values.noise_control?.value ?? 0;
  const deviceValue: number | string =
    typeof rawValue === "boolean" ? Number(rawValue) : rawValue;
  const authoritative =
    capability?.kind === "cnc" &&
    capability.range &&
    typeof deviceValue === "number"
      ? capability.range.minimum + capability.range.maximum - deviceValue
      : deviceValue;
  const authoritativeRef = useRef(authoritative);
  authoritativeRef.current = authoritative;
  const writingRef = useRef(false);
  const [value, setValue] = useState<number | string>(authoritative);
  useEffect(() => {
    if (!writingRef.current) setValue(authoritative);
  }, [authoritative, snapshot.generation]);

  const queue = useMemo(
    () =>
      new LatestWriteQueue<number | string, DeviceSnapshot>(
        (next) =>
          api.setNoiseControl({
            generation: snapshot.generation,
            kind: capability?.kind ?? "cnc",
            value:
              capability?.kind === "cnc" &&
              capability.range &&
              typeof next === "number"
                ? capability.range.minimum + capability.range.maximum - next
                : next,
          }),
        (next) => {
          writingRef.current = false;
          onSnapshot(next);
        },
        (error) => {
          writingRef.current = false;
          setValue(authoritativeRef.current);
          onError(error);
        },
        300,
      ),
    [snapshot.generation, capability?.kind, onSnapshot, onError],
  );
  useEffect(() => {
    queue.start();
    return () => queue.dispose();
  }, [queue]);

  if (!capability) {
    return (
      <section className={card}>
        <h2 className="text-[19px] font-bold">Noise control</h2>
        <p className={muted}>
          This device does not report a supported noise-control capability.
        </p>
      </section>
    );
  }
  return (
    <section className={card} aria-labelledby="noise-title">
      <div className={heading}>
        <div>
          <p className={eyebrow}>Listening</p>
          <h2 className="text-[19px] font-bold" id="noise-title">
            Active noise cancellation
          </h2>
        </div>
        <output className="min-w-[42px] rounded-full bg-accent-soft px-2.5 py-[5px] text-center font-bold text-accent">
          {String(value)}
        </output>
      </div>
      {capability.range ? (
        <label className="mt-[22px] block">
          <span className="mb-3 block font-semibold">ANC strength</span>
          <input
            aria-label="Active noise cancellation strength"
            type="range"
            min={capability.range.minimum}
            max={capability.range.maximum}
            step={capability.range.step}
            value={Number(value)}
            onChange={(event) => {
              const next = Number(event.currentTarget.value);
              writingRef.current = true;
              setValue(next);
              queue.enqueue(next);
            }}
            onPointerUp={() => void queue.flush()}
            onKeyUp={() => void queue.flush()}
          />
          <span className="flex justify-between text-xs text-muted">
            <span>Awareness</span>
            <span>Maximum ANC</span>
          </span>
        </label>
      ) : (
        <div
          className="mt-5 flex flex-wrap gap-[7px]"
          role="group"
          aria-label="Noise control mode"
        >
          {capability.options.map((option) => (
            <button
              className={`${value === option.value ? selectedButton : secondaryButton} min-w-20 flex-1`}
              type="button"
              aria-pressed={value === option.value}
              key={option.value}
              onClick={() => {
                writingRef.current = true;
                setValue(option.value);
                queue.enqueue(option.value);
                void queue.flush();
              }}
            >
              {option.label}
            </button>
          ))}
        </div>
      )}
    </section>
  );
}
