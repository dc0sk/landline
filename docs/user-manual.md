---
title: Operator User Manual
status: Draft
version: "1.1"
updated: 2026-07-21
authors:
  - Simon Keimer (DC0SK)
---

# landline — Operator User Manual

License notice: This project is licensed under AGPL-3.0-only. See the top-level LICENSE file.

This manual is for **operators**: how to sign in and use landline from a web browser to control a
rig, watch the spectrum, listen to and transmit audio, and drive GPIO. It does not cover
installation or administration — see [`deployment.md`](deployment.md) and
[`../deploy/RUNBOOK.md`](../deploy/RUNBOOK.md) for those.

## 1. Before you start

You need three things from whoever set up your station:

- The **address** of the landline server (e.g. `https://radio.example` or a private
  tunnel address such as `http://radio-pi:8443`).
- A **username and password**. landline never stores plaintext passwords; the operator who
  created your account can issue a new one but cannot recover the old.
- Your **role** — `Observer`, `Operator`, or `Admin` (see §3).

**Supported browsers.** Any current Firefox, Chrome/Chromium, Edge, or Safari (desktop or mobile).
Audio and microphone features need a **secure context** — i.e. the page served over `https://`
(or `http://localhost`). Over plain `http://` to a remote host, the browser will block microphone
access.

## 2. Signing in

1. Open the server address in your browser. You'll see the **landline** sign-in screen.
2. Enter your **name** and **password** and select **Sign in**.
3. On success the control interface appears and the header shows *Signed in as `<role>`*.

If the credentials are wrong you'll see an error and stay on the sign-in screen. Nothing else is
reachable until you sign in — every action is authenticated.

Your session uses a short-lived token that landline **refreshes automatically** in the background,
so you can stay signed in during a session without re-entering your password. Select **Sign out**
(top right) to end the session immediately; a server restart also signs everyone out (you simply
sign in again).

## 3. Roles and what they can do

Your role is fixed by your account and decides which controls work. If you try something your role
doesn't allow, the action is refused (and recorded in the audit log).

| Capability | Observer | Operator | Admin |
|---|:---:|:---:|:---:|
| Spectrum / waterfall | ✅ | ✅ | ✅ |
| Receive audio | ✅ | ✅ | ✅ |
| S-meter | ✅ | ✅ | ✅ |
| Read/set frequency, mode, passband | — | ✅ | ✅ |
| PTT / transmit (mic) | — | ✅ | ✅ |
| GPIO read/toggle | — | ✅ | ✅ |
| Audit log | — | — | ✅ |

Observers get situational awareness (spectrum, audio, signal strength) without any control.
Operators have full rig and station control. Admins additionally have the audit log.

## 4. The interface

After signing in the page shows, top to bottom:

- **Rig** — frequency, mode, passband, PTT, and the S-meter.
- **Spectrum** — the live waterfall and a palette selector.
- **GPIO** — one row per configured pin (Operator).
- **Audio devices** — microphone and speaker selection.

Only the sections your role and station support will do anything; a station with GPIO disabled, for
example, shows an empty GPIO panel.

## 5. Rig control

**Frequency.** The current frequency is shown in Hz. To change it, type the target frequency in
Hz in **Set frequency** and select **Set**. Only set frequencies within **your license privileges
and the rig's supported bands** — landline rejects clearly invalid values but does not know your
regulations. Example: 14,074,000 = 14.074 MHz.

**Mode and passband.** Pick the mode (USB, LSB, CW, FM, AM, …) from the **Mode** menu; the change
is sent immediately. Set **Passband (Hz)** for the receive filter width where the rig supports it
(leave `0` for the rig's default), then change the mode or passband field to apply.

**S-meter.** The received signal strength is displayed and updates as you operate.

**PTT — transmit.** The **PTT (transmit)** button keys the transmitter. While transmitting, the
button shows **ON AIR — release**; select it again to return to receive.

> ⚠️ **Transmitting is real.** Pressing PTT keys your radio on the air. Make sure your antenna or
> dummy load, band, and power are correct first. As a safety net, landline **auto-unkeys** after a
> configured timeout if PTT is left active, but do not rely on it — always release PTT yourself.

Only one operator controls the rig at a time; landline validates every command before it reaches
the radio, and records each state change.

## 6. Spectrum and waterfall

The **Spectrum** panel shows a scrolling waterfall of the receiver passband — newest line at the
bottom, scrolling upward. Use the **Palette** menu to switch the colour map (**hot**, **grayscale**,
or **ice**) to taste or for visibility. The display is live while you're signed in; if the
connection drops it reconnects automatically and resumes.

## 7. Audio

**Listening.** Receive audio plays automatically once you're signed in — no button to press. Adjust
your computer's or the rig's volume as usual.

> **Note — Opus builds.** Browser playback currently supports uncompressed audio only. If your
> station's server was built with Opus compression enabled, landline will tell you that audio is
> unavailable rather than play noise; the spectrum and all other controls are unaffected. Ask your
> administrator to run the server without the `opus` feature until browser-side Opus decoding
> ships.

**Choosing devices.** Under **Audio devices**, pick your **Input (microphone)** and
**Output (speaker)**. If the list shows only generic names, your browser hasn't granted microphone
permission yet — allow it when prompted so device names appear.

**Transmitting audio.** Microphone audio is sent **only while you are transmitting**: press
**PTT**, speak, release. Mic capture starts on PTT-on and stops on PTT-off, so your microphone is
never streamed when you're not on the air.

If you hear no audio, see §9.

## 8. GPIO (Operator)

If your station exposes GPIO pins, the **GPIO** panel lists each allowlisted pin with its number,
direction, and current level:

- **Output** pins show a level badge and a **Set HIGH / Set LOW** button — select it to drive the
  pin. The badge updates to confirm.
- **Input** pins show a read-only level badge that refreshes automatically.

Only pins your administrator has allowlisted appear or respond; every other pin is inaccessible.
Each change is Operator-gated and recorded in the audit log. Pins are driven to a safe state when
the service starts.

## 9. Troubleshooting

| Symptom | Likely cause | What to do |
|---|---|---|
| Can't sign in | Wrong name/password, or account not set up | Re-check credentials; ask your admin to (re)issue them |
| A control does nothing / "not permitted" | Your role lacks that capability | Check your role in the header; ask for an Operator account |
| Signed out unexpectedly | Server restarted, or session ended | Sign in again |
| No frequency/mode shown | You're an Observer (control is Operator-only) | Use an Operator account for rig control |
| No spectrum | Connection dropped | It reconnects automatically; refresh if it doesn't |
| No receive audio | Muted, wrong output device, or non-secure page | Check volume/output device; ensure the page is `https://` |
| "audio codec not supported" message | Server built with Opus; the browser decodes uncompressed audio only | Ask your admin to run the server without the `opus` feature (see §7) |
| Microphone unavailable on PTT | Browser mic permission denied, or non-secure page | Allow microphone access; use `https://` |
| Rig control returns an error | Rig temporarily unreachable | Wait a moment and retry; tell your admin if it persists |

For anything the manual doesn't cover, contact whoever administers your landline station.
