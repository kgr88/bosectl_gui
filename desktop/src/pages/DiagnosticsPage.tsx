import type { DeviceSnapshot } from "../contracts";
import {
  dangerTextButton,
  groupList,
  row,
  section,
  sectionHeader,
  textButton,
} from "../styles/ui";

export function diagnosticAddress(address: string | null | undefined): string {
  if (!address) return "Unknown";
  const parts = address.split(":");
  return parts.length === 6 ? `••:••:••:••:${parts[4]}:${parts[5]}` : "Hidden";
}

export function DiagnosticsPage({
  snapshot,
  busy,
  onRefresh,
  onDisconnect,
}: {
  snapshot: DeviceSnapshot;
  busy: boolean;
  onRefresh: () => void;
  onDisconnect: () => void;
}) {
  const details = [
    ["Model", snapshot.identity?.model ?? "Unknown"],
    ["Software version", snapshot.firmware?.version || "Unknown"],
    ["Bluetooth address", diagnosticAddress(snapshot.identity?.address)],
  ];
  return (
    <>
      <section className={section} aria-labelledby="about-title">
        <h2 className={sectionHeader} id="about-title">
          Headphones
        </h2>
        <dl className={`${groupList} m-0`}>
          {details.map(([term, value]) => (
            <div className={row} key={term}>
              <dt>{term}</dt>
              <dd className="m-0 text-right text-muted [overflow-wrap:anywhere]">
                {value}
              </dd>
            </div>
          ))}
        </dl>
      </section>
      <section className={section} aria-label="Connection">
        <ul className={groupList}>
          <li>
            <button
              className={`${textButton} ${row} w-full`}
              type="button"
              disabled={busy}
              onClick={onRefresh}
            >
              Refresh
            </button>
          </li>
          <li>
            <button
              className={`${dangerTextButton} ${row} w-full`}
              type="button"
              disabled={busy}
              onClick={onDisconnect}
            >
              Disconnect
            </button>
          </li>
        </ul>
      </section>
    </>
  );
}
