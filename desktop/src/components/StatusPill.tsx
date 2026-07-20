import type { ConnectionState } from "../contracts";

export function StatusPill({ state }: { state: ConnectionState }) {
  return <span className={`status status--${state}`}>{state.replace("_", " ")}</span>;
}
