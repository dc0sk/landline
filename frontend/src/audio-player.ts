// RX audio playback (ARC-11, A31). Parses the backend's binary audio frames,
// reorders them through a jitter buffer with loss concealment (FR-AUD-06), and
// plays them via Web Audio. The parse + jitter logic is pure and unit-tested;
// the AudioContext scheduling is a thin browser wrapper.

/** A decoded audio frame: sequence number + 16-bit PCM samples. */
export interface AudioFrame {
  readonly seq: number;
  readonly pcm: Int16Array;
}

/** Parse a binary WS audio frame: 8-byte little-endian sequence header followed
 *  by little-endian 16-bit PCM. */
export function parseAudioFrame(buffer: ArrayBuffer): AudioFrame {
  const view = new DataView(buffer);
  const seq = Number(view.getBigUint64(0, true));
  const pcm = new Int16Array(buffer.slice(8)); // browsers are little-endian
  return { seq, pcm };
}

/** What to play next: decoded samples, or a concealed loss. */
export type Playout = { readonly kind: "data"; readonly pcm: Int16Array } | { readonly kind: "lost" };

/** Reordering jitter buffer with graceful loss concealment (FR-AUD-06) — the
 *  browser-side mirror of the backend `JitterBuffer`. */
export class JitterBuffer {
  private readonly targetDepth: number;
  private readonly maxDepth: number;
  private readonly frames = new Map<number, Int16Array>();
  private nextSeq: number | null = null;
  private started = false;

  constructor(targetDepth: number, maxDepth: number) {
    this.targetDepth = Math.max(1, targetDepth);
    this.maxDepth = Math.max(this.targetDepth, maxDepth);
  }

  push(frame: AudioFrame): void {
    if (this.nextSeq !== null && frame.seq < this.nextSeq) {
      return; // late — already played past
    }
    this.frames.set(frame.seq, frame.pcm);
  }

  pop(): Playout | null {
    if (!this.started) {
      if (this.frames.size < this.targetDepth) {
        return null;
      }
      this.started = true;
      this.nextSeq = Math.min(...this.frames.keys());
    }
    if (this.nextSeq === null) {
      return null;
    }
    const next = this.nextSeq;
    const pcm = this.frames.get(next);
    if (pcm !== undefined) {
      this.frames.delete(next);
      this.nextSeq = next + 1;
      return { kind: "data", pcm };
    }
    if (this.frames.size > this.maxDepth) {
      this.nextSeq = next + 1;
      return { kind: "lost" };
    }
    return null;
  }
}

/** Plays RX audio frames through Web Audio, scheduling buffers back-to-back. */
export class AudioPlayer {
  private readonly context: AudioContext;
  private readonly jitter: JitterBuffer;
  private readonly sampleRate: number;
  private nextStartTime = 0;
  private timer: number | null = null;

  constructor(sampleRate: number) {
    this.sampleRate = sampleRate;
    this.context = new AudioContext({ sampleRate });
    this.jitter = new JitterBuffer(3, 10);
  }

  push(frame: AudioFrame): void {
    this.jitter.push(frame);
  }

  start(): void {
    this.timer = window.setInterval(() => this.drain(), 20);
  }

  stop(): void {
    if (this.timer !== null) {
      window.clearInterval(this.timer);
      this.timer = null;
    }
    void this.context.close();
  }

  private drain(): void {
    for (let playout = this.jitter.pop(); playout !== null; playout = this.jitter.pop()) {
      if (playout.kind === "data") {
        this.schedule(playout.pcm);
      }
      // "lost" -> skip (a small silent gap conceals the loss)
    }
  }

  private schedule(pcm: Int16Array): void {
    const buffer = this.context.createBuffer(1, pcm.length, this.sampleRate);
    const channel = buffer.getChannelData(0);
    for (let i = 0; i < pcm.length; i++) {
      channel[i] = (pcm[i] ?? 0) / 32768;
    }
    const source = this.context.createBufferSource();
    source.buffer = buffer;
    source.connect(this.context.destination);
    const now = this.context.currentTime;
    if (this.nextStartTime < now) {
      this.nextStartTime = now;
    }
    source.start(this.nextStartTime);
    this.nextStartTime += buffer.duration;
  }
}
