import { useEffect, useState } from "react";
import { api } from "../api";
import type { BackendInfo, DeviceSnapshot } from "../contracts";

export function DiagnosticsPage({ snapshot, busy, onRefresh, onDisconnect }: { snapshot: DeviceSnapshot; busy: boolean; onRefresh: () => void; onDisconnect: () => void }) {
  const [backend, setBackend] = useState<BackendInfo | null>(null);
  useEffect(() => { let active = true; void api.backendInfo().then((info) => active && setBackend(info)).catch(() => undefined); return () => { active = false; }; }, []);
  return <section className="card" aria-labelledby="diagnostics-title"><p className="eyebrow">Support</p><h2 id="diagnostics-title">Diagnostics</h2><dl className="diagnostics">
    <div><dt>Model</dt><dd>{snapshot.identity?.model ?? "Unknown"}</dd></div>
    <div><dt>Firmware</dt><dd>{snapshot.firmware?.version || "Unknown"}</dd></div>
    <div><dt>Connection generation</dt><dd>{snapshot.generation}</dd></div>
    <div><dt>Snapshot revision</dt><dd>{snapshot.revision}</dd></div>
    <div><dt>Address</dt><dd>{snapshot.identity?.address ?? "Not reported"}</dd></div>
    <div><dt>RFCOMM channel</dt><dd>{snapshot.connection.rfcomm_channel ?? "Not reported"}</dd></div>
    <div><dt>Backend</dt><dd>{backend ? `${backend.backend_version} · Python ${backend.python_version}` : "Loading…"}</dd></div>
  </dl><div className="button-row"><button type="button" disabled={busy} onClick={onRefresh}>Refresh now</button><button className="secondary" type="button" disabled={busy} onClick={onDisconnect}>Disconnect</button></div></section>;
}
