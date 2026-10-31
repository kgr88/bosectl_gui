import type { ConnectionState } from "../contracts";

export function StatusPill({ state }: { state: ConnectionState }) {
  const stateColor =
    state === "connected"
      ? "border-[#65a77f] text-[#197044]"
      : state === "error"
        ? "border-current text-danger"
        : "border-border";
  return (
    <span
      className={`${stateColor} whitespace-nowrap rounded-full border bg-surface px-2.5 py-[5px] text-xs capitalize forced-colors:border-[CanvasText]`}
    >
      {state.replace("_", " ")}
    </span>
  );
}
