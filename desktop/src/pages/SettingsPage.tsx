import { useEffect, useState } from "react";
import { api } from "../api";
import type { DeviceSnapshot, SettingCapability, SettingValue } from "../contracts";
import { card, eyebrow, input, muted, row, select, smallMuted } from "../styles/ui";

interface Props { snapshot: DeviceSnapshot; onSnapshot: (value: DeviceSnapshot) => void; onError: (error: unknown) => void; }

function SettingInput({ capability, value, busy, onChange }: {
  capability: SettingCapability; value: SettingValue | undefined; busy: boolean; onChange: (value: SettingValue) => void;
}) {
  if (capability.kind === "toggle") return <input type="checkbox" checked={Boolean(value)} disabled={busy || !capability.writable} onChange={(e) => onChange(e.currentTarget.checked)} />;
  if (capability.kind === "choice") return <select className={select} value={String(value ?? "")} disabled={busy || !capability.writable} onChange={(e) => onChange(e.currentTarget.value)}>{capability.options.map((option) => <option value={option.value} key={option.value}>{option.label}</option>)}</select>;
  if (capability.kind === "range") return <input type="range" value={Number(value ?? capability.minimum ?? 0)} min={capability.minimum ?? undefined} max={capability.maximum ?? undefined} step={capability.step ?? undefined} disabled={busy || !capability.writable} onChange={(e) => onChange(Number(e.currentTarget.value))} />;
  return <input className={`${input} w-[min(260px,48%)]`} key={String(value ?? "")} type="text" defaultValue={String(value ?? "")} readOnly={!capability.writable} disabled={busy} onBlur={(e) => capability.writable && e.currentTarget.value !== String(value ?? "") && onChange(e.currentTarget.value)} />;
}

export function SettingsPage({ snapshot, onSnapshot, onError }: Props) {
  const [draft, setDraft] = useState(snapshot.values.settings);
  const [busy, setBusy] = useState<string | null>(null);
  useEffect(() => setDraft(snapshot.values.settings), [snapshot.generation, snapshot.revision]);
  async function update(capability: SettingCapability, value: SettingValue) {
    const previous = draft[capability.key];
    setDraft((current) => ({ ...current, [capability.key]: value }));
    setBusy(capability.key);
    try { onSnapshot(await api.setSetting({ generation: snapshot.generation, key: capability.key, value })); }
    catch (error) { setDraft((current) => ({ ...current, [capability.key]: previous })); onError(error); }
    finally { setBusy(null); }
  }
  return <section className={card} aria-labelledby="settings-title"><p className={eyebrow}>Device</p><h2 className="text-[19px] font-bold" id="settings-title">Settings</h2><div className="mt-3.5">
    {snapshot.capabilities.settings.map((capability) => <label className={`${row} min-h-[50px] border-t border-border`} key={capability.key}>
      <span className="flex-1"><strong>{capability.label}</strong>{!capability.writable && <small className={smallMuted}>Read only</small>}</span>
      <SettingInput capability={capability} value={draft[capability.key]} busy={busy === capability.key} onChange={(value) => void update(capability, value)} />
    </label>)}
  </div>{snapshot.capabilities.settings.length === 0 && <p className={muted}>No configurable settings were reported.</p>}</section>;
}
