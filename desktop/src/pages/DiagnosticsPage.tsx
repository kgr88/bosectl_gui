import { useEffect, useState } from "react";
import { api } from "../api";
import type { BackendInfo, DeviceSnapshot } from "../contracts";
import { button, buttonRow, card, eyebrow, secondaryButton } from "../styles/ui";

export function diagnosticAddress(address: string | null | undefined): string {
  if (!address) return "Not reported";
  const parts = address.split(":");
  return parts.length === 6 ? `••:••:••:••:${parts[4]}:${parts[5]}` : "Redacted";
}

export function DiagnosticsPage({ snapshot, busy, onRefresh, onDisconnect }: { snapshot: DeviceSnapshot; busy: boolean; onRefresh: () => void; onDisconnect: () => void }) {
  const [backend, setBackend] = useState<BackendInfo | null>(null);
  useEffect(() => { let active = true; void api.backendInfo().then((info) => active && setBackend(info)).catch(() => undefined); return () => { active = false; }; }, []);
  const item = "bg-surface-raised p-[11px]";
  const term = "text-xs text-muted";
  const value = "mt-1 [overflow-wrap:anywhere]";
  return <section className={card} aria-labelledby="diagnostics-title"><p className={eyebrow}>Support</p><h2 className="text-[19px] font-bold" id="diagnostics-title">Diagnostics</h2><dl className="my-[18px] grid grid-cols-2 gap-px overflow-hidden rounded-lg border border-border bg-border max-[600px]:grid-cols-1">
    <div className={item}><dt className={term}>Model</dt><dd className={value}>{snapshot.identity?.model ?? "Unknown"}</dd></div>
    <div className={item}><dt className={term}>Firmware</dt><dd className={value}>{snapshot.firmware?.version || "Unknown"}</dd></div>
    <div className={item}><dt className={term}>Connection generation</dt><dd className={value}>{snapshot.generation}</dd></div>
    <div className={item}><dt className={term}>Snapshot revision</dt><dd className={value}>{snapshot.revision}</dd></div>
    <div className={item}><dt className={term}>Address</dt><dd className={value}>{diagnosticAddress(snapshot.identity?.address)}</dd></div>
    <div className={item}><dt className={term}>RFCOMM channel</dt><dd className={value}>{snapshot.connection.rfcomm_channel ?? "Not reported"}</dd></div>
    <div className={item}><dt className={term}>Backend</dt><dd className={value}>{backend ? `${backend.backend_version} · Python ${backend.python_version}` : "Loading…"}</dd></div>
    <div className={`${item} max-[600px]:hidden`} aria-hidden="true" />
  </dl><div className={buttonRow}><button className={button} type="button" disabled={busy} onClick={onRefresh}>Refresh now</button><button className={secondaryButton} type="button" disabled={busy} onClick={onDisconnect}>Disconnect</button></div></section>;
}
