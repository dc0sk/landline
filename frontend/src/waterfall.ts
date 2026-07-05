// Canvas 2D waterfall renderer (ARC-12, A29, FR-SPEC-03). Deliberately uses the
// 2D context and ImageData only — NO WebGL — so it works on iOS Safari
// (TC-SPEC-04). The colour mapping (normalise → palette → row pixels) is pure
// and unit-tested; the scrolling draw is a thin Canvas wrapper.

export type Rgb = [number, number, number];

function clamp01(value: number): number {
  return Math.min(1, Math.max(0, value));
}

/** Map a dB value into [0, 1] against a display range, clamped. */
export function normalizeDb(db: number, minDb: number, maxDb: number): number {
  if (maxDb <= minDb) {
    return 0;
  }
  return clamp01((db - minDb) / (maxDb - minDb));
}

/** A "hot" colour map: 0 → black, through red/yellow, 1 → white. */
export function palette(value: number): Rgb {
  const v = clamp01(value);
  return [
    Math.round(clamp01(v * 3) * 255),
    Math.round(clamp01(v * 3 - 1) * 255),
    Math.round(clamp01(v * 3 - 2) * 255),
  ];
}

/** Build one row of opaque RGBA pixels (one pixel per bin). */
export function rowRgba(bins: number[], minDb: number, maxDb: number): Uint8ClampedArray {
  const pixels = new Uint8ClampedArray(bins.length * 4);
  for (let i = 0; i < bins.length; i++) {
    const [r, g, b] = palette(normalizeDb(bins[i] ?? minDb, minDb, maxDb));
    const offset = i * 4;
    pixels[offset] = r;
    pixels[offset + 1] = g;
    pixels[offset + 2] = b;
    pixels[offset + 3] = 255;
  }
  return pixels;
}

export interface WaterfallOptions {
  readonly minDb: number;
  readonly maxDb: number;
}

/** Scrolling waterfall: each `push` shifts the image up one row and draws the
 *  new spectrum row at the bottom. */
export class WaterfallRenderer {
  private readonly ctx: CanvasRenderingContext2D;
  private readonly minDb: number;
  private readonly maxDb: number;

  constructor(ctx: CanvasRenderingContext2D, options: WaterfallOptions) {
    this.ctx = ctx;
    this.minDb = options.minDb;
    this.maxDb = options.maxDb;
  }

  push(bins: number[]): void {
    if (bins.length === 0) {
      return;
    }
    const canvas = this.ctx.canvas;
    // One canvas pixel per bin; CSS scales it to the display width.
    if (canvas.width !== bins.length) {
      canvas.width = bins.length;
    }
    const { width, height } = canvas;
    if (height > 1) {
      const previous = this.ctx.getImageData(0, 1, width, height - 1);
      this.ctx.putImageData(previous, 0, 0);
    }
    const rowImage = this.ctx.createImageData(bins.length, 1);
    rowImage.data.set(rowRgba(bins, this.minDb, this.maxDb));
    this.ctx.putImageData(rowImage, 0, height - 1);
  }
}
