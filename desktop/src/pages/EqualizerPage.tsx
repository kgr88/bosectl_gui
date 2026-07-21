import { useEffect, useMemo, useRef, useState } from "react";
import { api } from "../api";
import type { DeviceSnapshot } from "../contracts";
import { LatestWriteQueue } from "../state/latestWrite";

interface Props { snapshot: DeviceSnapshot; onSnapshot: (value: DeviceSnapshot) => void; onError: (error: unknown) => void; }
type EqValue = { bass: number; mid: number; treble: number };

function readEq(snapshot: DeviceSnapshot): EqValue {
  return { bass: snapshot.values.eq.bass ?? 0, mid: snapshot.values.eq.mid ?? 0, treble: snapshot.values.eq.treble ?? 0 };
}

export function EqualizerPage({ snapshot, onSnapshot, onError }: Props) {
  const authoritative = readEq(snapshot);
  const authoritativeRef = useRef(authoritative);
  authoritativeRef.current = authoritative;
  const writingRef = useRef(false);
  const [value, setValue] = useState(authoritative);
  useEffect(() => {
    if (!writingRef.current) setValue(authoritative);
  }, [snapshot.generation, snapshot.revision]);
  const queue = useMemo(() => new LatestWriteQueue<EqValue, DeviceSnapshot>(
    (next) => api.setEq({ generation: snapshot.generation, ...next }),
    (next) => { writingRef.current = false; onSnapshot(next); },
    (error) => { writingRef.current = false; setValue(authoritativeRef.current); onError(error); },
    300,
  ), [snapshot.generation, onSnapshot, onError]);
  useEffect(() => {
    queue.start();
    return () => queue.dispose();
  }, [queue]);

  return <section className="card" aria-labelledby="eq-title"><p className="eyebrow">Sound</p><h2 id="eq-title">Equalizer</h2><div className="eq-grid">
    {snapshot.capabilities.equalizer.map((band) => <label className="eq-band" key={band.id}>
      <output>{value[band.id as keyof EqValue] ?? 0}</output><span>{band.label}</span>
      <input aria-label={`${band.label} equalizer`} type="range" min={band.minimum} max={band.maximum} step={band.step}
        value={value[band.id as keyof EqValue] ?? 0} onChange={(event) => {
          const next = { ...value, [band.id]: Number(event.currentTarget.value) } as EqValue;
          writingRef.current = true; setValue(next); queue.enqueue(next);
        }} onPointerUp={() => void queue.flush()} onKeyUp={() => void queue.flush()} />
    </label>)}
  </div><button className="secondary" type="button" onClick={() => { const flat = { bass: 0, mid: 0, treble: 0 }; writingRef.current = true; setValue(flat); queue.enqueue(flat); void queue.flush(); }}>Reset</button></section>;
}
