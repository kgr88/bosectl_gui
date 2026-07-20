import type { ApiError, DeviceEvent, DeviceSnapshot } from "../contracts";

export interface DeviceState {
  snapshot: DeviceSnapshot | null;
  error: ApiError | null;
}

export const initialDeviceState: DeviceState = { snapshot: null, error: null };

export function isNewer(current: DeviceSnapshot | null, next: DeviceSnapshot): boolean {
  return current === null
    || next.generation > current.generation
    || (next.generation === current.generation && next.revision >= current.revision);
}

export function reduceDeviceEvent(state: DeviceState, event: DeviceEvent): DeviceState {
  if (event.event === "backend_error") return { ...state, error: event.data };
  if (event.event !== "snapshot_changed" || !isNewer(state.snapshot, event.data)) return state;
  return { snapshot: event.data, error: null };
}
