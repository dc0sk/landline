// Audio device enumeration (ARC-11, A30, FR-AUD-03/04). Uses the standard
// browser MediaDevices API (NFR-COMPAT-07). The partitioning logic is pure and
// unit-tested; the permission prompt (getUserMedia, needed before labels are
// visible) stays in the DOM glue.

/** The subset of `MediaDeviceInfo` we depend on. */
export interface MediaDeviceLike {
  readonly deviceId: string;
  readonly kind: string; // "audioinput" | "audiooutput" | "videoinput"
  readonly label: string;
}

export interface AudioDevice {
  readonly deviceId: string;
  readonly label: string;
}

export interface AudioDevices {
  readonly inputs: AudioDevice[];
  readonly outputs: AudioDevice[];
}

/** Split a device list into audio inputs and outputs, giving unlabelled devices
 *  a stable fallback name (labels are blank until mic permission is granted). */
export function partitionAudioDevices(devices: MediaDeviceLike[]): AudioDevices {
  const inputs: AudioDevice[] = [];
  const outputs: AudioDevice[] = [];
  for (const device of devices) {
    if (device.kind === "audioinput") {
      inputs.push({
        deviceId: device.deviceId,
        label: device.label || `Microphone ${inputs.length + 1}`,
      });
    } else if (device.kind === "audiooutput") {
      outputs.push({
        deviceId: device.deviceId,
        label: device.label || `Speaker ${outputs.length + 1}`,
      });
    }
  }
  return { inputs, outputs };
}

/** Minimal enumerator surface (satisfied by `navigator.mediaDevices`). */
export interface DeviceEnumerator {
  enumerateDevices(): Promise<MediaDeviceLike[]>;
}

/** Enumerate and partition the available audio devices. */
export async function loadAudioDevices(enumerator: DeviceEnumerator): Promise<AudioDevices> {
  return partitionAudioDevices(await enumerator.enumerateDevices());
}
