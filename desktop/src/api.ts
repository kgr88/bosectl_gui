import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import {
  type ApiError,
  type BackendInfo,
  type ConnectionTarget,
  type DeviceEvent,
  type DeviceSnapshot,
  type DiscoveredDevice,
  type ProfileInput,
  type ProfileList,
  type PollField,
  type SetEqRequest,
  type SetNoiseControlRequest,
  type SetSettingRequest,
  isDeviceSnapshot,
  isBackendInfo,
  isDiscoveredDevice,
  isDeviceEvent,
  isProfileList,
} from "./contracts";
import { createMockApi } from "./mockApi";

export interface BoseApi {
  backendInfo(): Promise<BackendInfo>;
  discover(): Promise<DiscoveredDevice[]>;
  connect(target: ConnectionTarget): Promise<DeviceSnapshot>;
  disconnect(): Promise<void>;
  refresh(): Promise<DeviceSnapshot>;
  currentSnapshot(): Promise<DeviceSnapshot | null>;
  poll(generation: number, fields: PollField[]): Promise<DeviceSnapshot>;
  setNoiseControl(request: SetNoiseControlRequest): Promise<DeviceSnapshot>;
  setEq(request: SetEqRequest): Promise<DeviceSnapshot>;
  setSetting(request: SetSettingRequest): Promise<DeviceSnapshot>;
  getProfiles(generation: number): Promise<ProfileList>;
  setProfile(generation: number, profile: ProfileInput): Promise<ProfileList>;
  deleteProfile(generation: number, profileId: string): Promise<ProfileList>;
  subscribe(listener: (event: DeviceEvent) => void): Promise<UnlistenFn>;
}

function boundaryError(message: string): ApiError {
  return { code: "INVALID_REQUEST", message, retryable: false };
}

async function snapshotCommand(command: string, args?: Record<string, unknown>): Promise<DeviceSnapshot> {
  const result: unknown = await invoke(command, args);
  if (!isDeviceSnapshot(result)) throw boundaryError(`${command} returned an invalid snapshot`);
  return result;
}

const tauriApi: BoseApi = {
  async backendInfo() {
    const result: unknown = await invoke("get_backend_info");
    if (!isBackendInfo(result)) throw boundaryError("get_backend_info returned invalid data");
    return result;
  },

  async discover() {
    const result: unknown = await invoke("discover_devices");
    if (!result || typeof result !== "object" || !("devices" in result)
      || !Array.isArray(result.devices) || !result.devices.every(isDiscoveredDevice)) {
      throw boundaryError("discover_devices returned invalid data");
    }
    return result.devices;
  },

  connect: (target) => snapshotCommand("connect_device", { target }),

  async disconnect() {
    const result = await invoke<{ ok: boolean }>("disconnect_device");
    if (!result?.ok) throw boundaryError("disconnect_device did not acknowledge the request");
  },

  refresh: () => snapshotCommand("get_device_snapshot"),
  async currentSnapshot() {
    const result: unknown = await invoke("get_current_device_snapshot");
    if (result !== null && !isDeviceSnapshot(result)) {
      throw boundaryError("get_current_device_snapshot returned invalid data");
    }
    return result;
  },
  poll: (generation, fields) => snapshotCommand("poll_device_snapshot", {
    request: { generation, fields },
  }),
  setNoiseControl: (request) => snapshotCommand("set_noise_control", { request }),
  setEq: (request) => snapshotCommand("set_equalizer", { request }),
  setSetting: (request) => snapshotCommand("set_device_setting", { request }),

  async getProfiles(generation) {
    const result: unknown = await invoke("get_profiles", { request: { generation } });
    if (!isProfileList(result)) throw boundaryError("get_profiles returned invalid data");
    return result;
  },

  async setProfile(generation, profile) {
    const result: unknown = await invoke("set_profile", { request: { generation, profile } });
    if (!isProfileList(result)) throw boundaryError("set_profile returned invalid data");
    return result;
  },

  async deleteProfile(generation, profileId) {
    const result: unknown = await invoke("delete_profile", {
      request: { generation, profile_id: profileId },
    });
    if (!isProfileList(result)) throw boundaryError("delete_profile returned invalid data");
    return result;
  },

  subscribe: (listener) => listen<unknown>("bose://device-event", (event) => {
    if (isDeviceEvent(event.payload)) listener(event.payload);
  }),
};

export const api: BoseApi = import.meta.env.DEV
  && typeof window !== "undefined"
  && !("__TAURI_INTERNALS__" in window)
  ? createMockApi()
  : tauriApi;
