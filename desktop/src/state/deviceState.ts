import type { ApiError, ConnectionState, DeviceEvent, DeviceSnapshot } from "../contracts";

export interface DeviceState {
  snapshot: DeviceSnapshot | null;
  error: ApiError | null;
  connection: ConnectionState;
}

export const initialDeviceState: DeviceState = { snapshot: null, error: null, connection: "disconnected" };

export function isNewer(current: DeviceSnapshot | null, next: DeviceSnapshot): boolean {
  return current === null
    || next.generation > current.generation
    || (next.generation === current.generation && next.revision >= current.revision);
}

export function reduceDeviceEvent(state: DeviceState, event: DeviceEvent): DeviceState {
  if (event.event === "backend_error") return {
    ...state,
    error: event.data,
    connection: ["BACKEND_CRASHED", "DEVICE_LOST", "DEVICE_TIMEOUT"].includes(event.data.code)
      ? "error" : state.connection,
  };
  if (event.event === "connection_changed") {
    if (state.snapshot && (event.data.generation < state.snapshot.generation
      || (event.data.generation === state.snapshot.generation && event.data.revision < state.snapshot.revision))) return state;
    return { ...state, connection: event.data.state };
  }
  if (!isNewer(state.snapshot, event.data)) return state;
  return { snapshot: event.data, connection: event.data.connection.state, error: null };
}
