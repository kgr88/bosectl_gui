import { useState } from "react";
import type { ConnectionTarget, DiscoveredDevice } from "../contracts";
import { button, card, eyebrow, fieldLabel, heading, input, muted, pageGrid, row, select, smallMuted } from "../styles/ui";

interface Props {
  devices: DiscoveredDevice[];
  busy: boolean;
  state: string;
  onDiscover: () => void;
  onConnect: (device: DiscoveredDevice) => void;
  onManualConnect: (target: ConnectionTarget) => void;
}

export function ConnectionPage({ devices, busy, state, onDiscover, onConnect, onManualConnect }: Props) {
  const [address, setAddress] = useState("");
  const [deviceType, setDeviceType] = useState<"qc_ultra2" | "qc35">("qc_ultra2");
  return <div className={pageGrid}><section className={card} aria-labelledby="connection-title">
    <div className={heading}><div><p className={eyebrow}>Windows Bluetooth</p><h2 className="text-[19px] font-bold" id="connection-title">Choose a paired headset</h2></div><button className={button} type="button" onClick={onDiscover} disabled={busy}>{busy ? (state === "connecting" ? "Connecting…" : "Searching…") : "Find devices"}</button></div>
    {devices.length === 0 ? <p className={muted}>Pair your headset in Windows Settings, then search again.</p> : <ul className="mt-[18px] list-none p-0">{devices.map((device) => <li className={`${row} border-t border-border py-[11px]`} key={device.device_id}><span><strong>{device.name}</strong><small className={smallMuted}>{device.supported ? "Supported" : "Choose its model manually"}</small></span><button className={button} type="button" disabled={busy || !device.supported} onClick={() => onConnect(device)}>Connect</button></li>)}</ul>}
  </section><section className={card} aria-labelledby="manual-title"><p className={eyebrow}>Fallback</p><h2 className="text-[19px] font-bold" id="manual-title">Connect manually</h2><p className={muted}>Use this when Windows reports the headset without enough model information.</p><form className="mt-[18px] grid gap-[13px]" onSubmit={(event) => { event.preventDefault(); onManualConnect({ kind: "manual", address: address.trim(), device_type: deviceType }); }}>
    <label className={fieldLabel}>Bluetooth address<input className={input} required pattern="[0-9A-Fa-f]{2}(:[0-9A-Fa-f]{2}){5}" placeholder="AA:BB:CC:DD:EE:FF" value={address} onChange={(event) => setAddress(event.currentTarget.value)} /></label>
    <label className={fieldLabel}>Headset model<select className={select} value={deviceType} onChange={(event) => setDeviceType(event.currentTarget.value as typeof deviceType)}><option value="qc_ultra2">QC Ultra 2</option><option value="qc35">QC35 / QC35 II</option></select></label>
    <button className={button} type="submit" disabled={busy || !address.trim()}>Connect</button>
  </form></section></div>;
}
