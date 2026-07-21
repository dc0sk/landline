// GPIO control panel (ARC-10 → ARC-08). Lists the allowlisted pins and, for
// outputs, offers a toggle. The API calls + toggle logic live in control.ts
// (unit-tested); this module is the browser DOM glue, refreshed on a timer so
// input-pin readings stay current.

import type { ApiClient } from "./api.ts";
import { listGpio, setGpio, toggledLevel, type GpioPin, type GpioStatus } from "./control.ts";

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
    let status: GpioStatus;
    try {
      status = await listGpio(this.api, token);
    } catch {
      return; // GPIO disabled or unreachable — leave the panel as-is
    }
    this.render(status);
  }

  private render(status: GpioStatus): void {
    this.container.replaceChildren();
    if (status.degraded) {
      // The station is up but its GPIO hardware is not. Say so plainly: the
      // pins below cannot be driven, and showing them without this note would
      // look like hardware that simply never changes state.
      const warning = document.createElement("p");
      warning.className = "gpio-degraded";
      warning.textContent =
        "GPIO hardware is unavailable — pins cannot be read or set. Ask your administrator to check the GPIO chip path and permissions.";
      this.container.append(warning);
    }
    if (status.pins.length === 0) {
      const note = document.createElement("p");
      note.textContent = "No GPIO pins configured.";
      this.container.append(note);
      return;
    }
    for (const pin of status.pins) {
      this.container.append(this.row(pin, status.degraded));
    }
  }

  private row(pin: GpioPin, degraded: boolean): HTMLElement {
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
      button.disabled = degraded || pin.level === null;
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
