import { test } from "node:test";
import assert from "node:assert/strict";
import {
  loadAudioDevices,
  partitionAudioDevices,
  type MediaDeviceLike,
} from "./audio-devices.ts";

test("partitions inputs and outputs and ignores video", () => {
  const devices: MediaDeviceLike[] = [
    { deviceId: "i1", kind: "audioinput", label: "Headset Mic" },
    { deviceId: "o1", kind: "audiooutput", label: "" },
    { deviceId: "v1", kind: "videoinput", label: "Webcam" },
    { deviceId: "i2", kind: "audioinput", label: "" },
  ];
  const result = partitionAudioDevices(devices);
  assert.deepEqual(result.inputs, [
    { deviceId: "i1", label: "Headset Mic" },
    { deviceId: "i2", label: "Microphone 2" }, // fallback label
  ]);
  assert.deepEqual(result.outputs, [{ deviceId: "o1", label: "Speaker 1" }]);
});

test("loadAudioDevices reads from the enumerator", async () => {
  const result = await loadAudioDevices({
    enumerateDevices: async () => [
      { deviceId: "i1", kind: "audioinput", label: "Mic" },
    ],
  });
  assert.equal(result.inputs.length, 1);
  assert.equal(result.outputs.length, 0);
});
