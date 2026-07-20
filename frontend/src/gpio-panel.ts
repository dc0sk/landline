// GPIO control panel (ARC-10 → ARC-08). Lists the allowlisted pins and, for
// outputs, offers a toggle. The API calls + toggle logic live in control.ts
// (unit-tested); this module is the browser DOM glue, refreshed on a timer so
// input-pin readings stay current.

import type { ApiClient } from "./api.ts";
import { listGpio, setGpio, toggledLevel, type GpioPin } from "./control.ts";

export interface GpioPanelOptions {
  readonly container: HTMLElement;
  readonly api: ApiClient;
  /** Current access token, or `null` when signed out. */
  readonly token: () => string | null;
  readonly onError?: (message: string) => void;
}

const REFRESH_MS = 2000;

export class GpioPanel {
  private readonly container: HTMLElement;
  private readonly api: ApiClient;
  private readonly token: () => string | null;
  private readonly onError: ((message: string) => void) | undefined;
  private timer: number | null = null;

  constructor(options: GpioPanelOptions) {
    this.container = options.container;
    this.api = options.api;
    this.token = options.token;
    this.onError = options.onError;
  }

  start(): void {
    void this.refresh();
    this.timer = window.setInterval(() => void this.refresh(), REFRESH_MS);
  }

  stop(): void {
    if (this.timer !== null) {
      window.clearInterval(this.timer);
      this.timer = null;
    }
    this.container.replaceChildren();
  }

  async refresh(): Promise<void> {
    const token = this.token();
    if (token === null) {
      return;
    }
    let pins: GpioPin[];
    try {
      pins = await listGpio(this.api, token);
    } catch {
      return; // GPIO disabled or unreachable — leave the panel as-is
    }
    this.render(pins);
  }

  private render(pins: GpioPin[]): void {
    this.container.replaceChildren();
    if (pins.length === 0) {
      const note = document.createElement("p");
      note.textContent = "No GPIO pins configured.";
      this.container.append(note);
      return;
    }
    for (const pin of pins) {
      this.container.append(this.row(pin));
    }
  }

  private row(pin: GpioPin): HTMLElement {
    const row = document.createElement("div");
    row.className = "gpio-row";

    const label = document.createElement("span");
    label.textContent = `GPIO${pin.pin} (${pin.direction})`;

    const level = document.createElement("span");
    // A pin the backend could not read has no level: show it as unknown rather
    // than defaulting to a level the pin may not actually be at.
    level.className = `gpio-level gpio-${pin.level ?? "unknown"}`;
    level.textContent = pin.level === null ? "UNKNOWN" : pin.level.toUpperCase();

    row.append(label, level);

    if (pin.direction === "out") {
      const button = document.createElement("button");
      button.type = "button";
      // With the level unknown there is no meaningful toggle target, so the
      // control is disabled rather than guessing a direction to drive.
      button.disabled = pin.level === null;
      button.textContent = pin.level === "high" ? "Set LOW" : "Set HIGH";
      button.addEventListener("click", () => void this.toggle(pin));
      row.append(button);
    }
    return row;
  }

  private async toggle(pin: GpioPin): Promise<void> {
    const token = this.token();
    if (token === null) {
      return;
    }
    try {
      if (pin.level === null) return;
      await setGpio(this.api, token, pin.pin, toggledLevel(pin.level));
      await this.refresh();
    } catch {
      this.onError?.("Could not set GPIO pin (Operator role required).");
    }
  }
}
