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

/** A colour map: value in [0, 1] → RGB (FR-SPEC-04, palette selection). */
export type Palette = (value: number) => Rgb;

/** "Hot": 0 → black, through red/yellow, 1 → white. */
export const hotPalette: Palette = (value) => {
  const v = clamp01(value);
  return [
    Math.round(clamp01(v * 3) * 255),
    Math.round(clamp01(v * 3 - 1) * 255),
    Math.round(clamp01(v * 3 - 2) * 255),
  ];
};

/** Grayscale: 0 → black, 1 → white. */
export const grayscalePalette: Palette = (value) => {
  const g = Math.round(clamp01(value) * 255);
  return [g, g, g];
};

/** "Ice": 0 → black, through blue/cyan, 1 → white. */
export const icePalette: Palette = (value) => {
  const v = clamp01(value);
  return [
    Math.round(clamp01(v * 3 - 2) * 255),
    Math.round(clamp01(v * 3 - 1) * 255),
    Math.round(clamp01(v * 3) * 255),
  ];
};

/** The selectable palettes, keyed by display name (FR-SPEC-04). */
export const PALETTES: Record<string, Palette> = {
  hot: hotPalette,
  grayscale: grayscalePalette,
  ice: icePalette,
};

export const PALETTE_NAMES: readonly string[] = Object.keys(PALETTES);

/** Build one row of opaque RGBA pixels (one pixel per bin) with `palette`. */
export function rowRgba(
  bins: number[],
  minDb: number,
  maxDb: number,
  palette: Palette,
): Uint8ClampedArray {
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
  readonly palette?: Palette;
}

/** Scrolling waterfall: each `push` shifts the image up one row and draws the
 *  new spectrum row at the bottom. */
export class WaterfallRenderer {
  private readonly ctx: CanvasRenderingContext2D;
  private readonly minDb: number;
  private readonly maxDb: number;
  private palette: Palette;

  constructor(ctx: CanvasRenderingContext2D, options: WaterfallOptions) {
    this.ctx = ctx;
    this.minDb = options.minDb;
    this.maxDb = options.maxDb;
    this.palette = options.palette ?? hotPalette;
  }

  /** Switch the colour palette for subsequent rows (FR-SPEC-04). */
  setPalette(palette: Palette): void {
    this.palette = palette;
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
    rowImage.data.set(rowRgba(bins, this.minDb, this.maxDb, this.palette));
    this.ctx.putImageData(rowImage, 0, height - 1);
  }
}
