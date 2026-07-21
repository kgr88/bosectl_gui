import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import type { DeviceSnapshot } from "./contracts";
import { EqualizerPage } from "./pages/EqualizerPage";
import { HomePage } from "./pages/HomePage";

function snapshot(overrides: Partial<DeviceSnapshot> = {}): DeviceSnapshot {
  return {
    identity: null, battery: null, firmware: null,
    connection: { state: "connected", reason: null },
    capabilities: { noise_control: null, equalizer: [], settings: [], profiles: null },
    values: { noise_control: null, eq: {}, settings: {}, profile_id: null },
    generation: 1, revision: 1, ...overrides,
  };
}

describe("capability rendering", () => {
  it("renders QC35 ANR choices without an Ultra slider", () => {
    const value = snapshot({
      capabilities: { noise_control: { kind: "anr", range: null, options: [{ value: "high", label: "High" }, { value: "low", label: "Low" }] }, equalizer: [], settings: [], profiles: null },
      values: { noise_control: { kind: "anr", value: "high" }, eq: {}, settings: {}, profile_id: null },
    });
    const html = renderToStaticMarkup(<HomePage snapshot={value} onSnapshot={() => undefined} onError={() => undefined} />);
    expect(html).toContain('role="group"');
    expect(html).toContain("High");
    expect(html).not.toContain('type="range"');
  });

  it("renders only reported equalizer bands with accessible labels", () => {
    const value = snapshot({
      capabilities: { noise_control: null, equalizer: [{ id: "bass", label: "Bass", minimum: -10, maximum: 10, step: 1 }], settings: [], profiles: null },
      values: { noise_control: null, eq: { bass: 3 }, settings: {}, profile_id: null },
    });
    const html = renderToStaticMarkup(<EqualizerPage snapshot={value} onSnapshot={() => undefined} onError={() => undefined} />);
    expect(html).toContain('aria-label="Bass equalizer"');
    expect(html).not.toContain("Treble");
  });
});
