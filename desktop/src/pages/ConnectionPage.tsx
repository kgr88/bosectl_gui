import type { DiscoveredDevice } from "../contracts";

interface Props {
  devices: DiscoveredDevice[];
  busy: boolean;
  onDiscover: () => void;
  onConnect: (device: DiscoveredDevice) => void;
}

export function ConnectionPage({ devices, busy, onDiscover, onConnect }: Props) {
  return (
    <section className="card" aria-labelledby="connection-title">
      <div className="card__heading">
        <div>
          <p className="eyebrow">Windows Bluetooth</p>
          <h2 id="connection-title">Choose a paired headset</h2>
        </div>
        <button type="button" onClick={onDiscover} disabled={busy}>
          {busy ? "Searching…" : "Find devices"}
        </button>
      </div>
      {devices.length === 0 ? (
        <p className="muted">Discovery and device operations connect to the long-lived backend in Stage 2.</p>
      ) : (
        <ul className="device-list">
          {devices.map((device) => (
            <li key={device.device_id}>
              <span><strong>{device.name}</strong><small>{device.supported ? "Supported" : "Unsupported"}</small></span>
              <button type="button" disabled={busy || !device.supported} onClick={() => onConnect(device)}>
                Connect
              </button>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
