import { useEffect, useReducer, useState } from "react";
import { api } from "./api";
import { isApiError, type ApiError, type DiscoveredDevice } from "./contracts";
import { StatusPill } from "./components/StatusPill";
import { ConnectionPage } from "./pages/ConnectionPage";
import { initialDeviceState, reduceDeviceEvent } from "./state/deviceState";

function asApiError(error: unknown): ApiError {
  if (isApiError(error)) return error;
  return { code: "INTERNAL", message: String(error), retryable: false };
}

export default function App() {
  const [state, dispatch] = useReducer(reduceDeviceEvent, initialDeviceState);
  const [devices, setDevices] = useState<DiscoveredDevice[]>([]);
  const [busy, setBusy] = useState(false);
  const [localError, setLocalError] = useState<ApiError | null>(null);

  useEffect(() => {
    let active = true;
    let unlisten: (() => void) | undefined;
    void api.subscribe(dispatch).then((stop) => {
      if (active) unlisten = stop;
      else stop();
    }).catch((error: unknown) => {
      if (active) setLocalError(asApiError(error));
    });
    return () => {
      active = false;
      unlisten?.();
    };
  }, []);

  async function discover() {
    setBusy(true);
    setLocalError(null);
    try { setDevices(await api.discover()); }
    catch (error) { setLocalError(asApiError(error)); }
    finally { setBusy(false); }
  }

  async function connect(device: DiscoveredDevice) {
    setBusy(true);
    setLocalError(null);
    try {
      const snapshot = await api.connect({ kind: "discovered", device_id: device.device_id });
      dispatch({ event: "snapshot_changed", data: snapshot });
    } catch (error) { setLocalError(asApiError(error)); }
    finally { setBusy(false); }
  }

  const error = localError ?? state.error;
  const connection = state.snapshot?.connection.state ?? "disconnected";

  return (
    <main className="app-shell">
      <header className="app-header">
        <div><p className="eyebrow">BMAP device controls</p><h1>bosectl</h1></div>
        <StatusPill state={connection} />
      </header>
      {error && <div className="error" role="alert"><strong>{error.code}</strong><span>{error.message}</span></div>}
      <ConnectionPage devices={devices} busy={busy} onDiscover={() => void discover()} onConnect={(d) => void connect(d)} />
      <footer>Protocol v1 · controls are rendered from reported capabilities</footer>
    </main>
  );
}
