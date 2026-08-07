/**
 * Portify desktop UI.
 *
 * Deliberately framework-free: the whole surface is one list, a filter box and
 * a settings drawer, and every byte of runtime saved keeps the bundle small
 * enough that the window opens instantly from the tray.
 */

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from "@tauri-apps/plugin-notification";

/* ---------- types shared with the Rust side ---------- */

interface ProcessInfo {
  pid: number;
  name: string;
  exe: string | null;
  command: string | null;
  description: string | null;
  memory_bytes: number | null;
  started_at: number | null;
}

interface PortGroup {
  port: number;
  protocol: "tcp" | "udp";
  service: string | null;
  processes: ProcessInfo[];
  addresses: string[];
  sockets: number;
  connections: number;
  owner_hidden: boolean;
}

interface ScanResult {
  ports: PortGroup[];
  elapsed_ms: number;
  elevated: boolean;
}

interface KillOutcome {
  pid: number;
  process_name: string;
  port: number | null;
  status:
    | "killed"
    | "escalated"
    | "not_found"
    | "permission_denied"
    | "survived"
    | "refused";
  detail: string;
}

interface Settings {
  refresh_interval_secs: number;
  notifications: boolean;
  confirm_before_kill: boolean;
  include_all: boolean;
  hide_on_blur: boolean;
  shortcut: string;
}

/* ---------- element handles ---------- */

const el = {
  list: must<HTMLElement>("list"),
  search: must<HTMLInputElement>("search"),
  refresh: must<HTMLButtonElement>("refresh"),
  settingsToggle: must<HTMLButtonElement>("settings-toggle"),
  close: must<HTMLButtonElement>("close"),
  settings: must<HTMLElement>("settings"),
  status: must<HTMLElement>("status"),
  privileges: must<HTMLElement>("privileges"),
  dot: must<HTMLElement>("status-dot"),
  interval: must<HTMLSelectElement>("setting-interval"),
  notifications: must<HTMLInputElement>("setting-notifications"),
  confirm: must<HTMLInputElement>("setting-confirm"),
  all: must<HTMLInputElement>("setting-all"),
  hideOnBlur: must<HTMLInputElement>("setting-hide-on-blur"),
  shortcut: must<HTMLInputElement>("setting-shortcut"),
  shortcutNote: must<HTMLElement>("shortcut-note"),
  shortcutReset: must<HTMLButtonElement>("shortcut-reset"),
};

function must<T extends HTMLElement>(id: string): T {
  const node = document.getElementById(id);
  if (!node) throw new Error(`missing element #${id}`);
  return node as T;
}

/* ---------- state ---------- */

const DEFAULT_SETTINGS: Settings = {
  refresh_interval_secs: 5,
  notifications: true,
  confirm_before_kill: true,
  include_all: false,
  hide_on_blur: false,
  shortcut: "CmdOrCtrl+Alt+P",
};

let settings: Settings = { ...DEFAULT_SETTINGS };
let ports: PortGroup[] = [];
let query = "";
/** Duration of the last scan, kept so the footer can be redrawn on filtering. */
let lastElapsedMs = 0;
let timer: number | undefined;
let windowVisible = true;
/** Ports with a kill in flight, so the row can show progress and ignore clicks. */
const killing = new Set<number>();
/** Ports waiting for a second click to confirm. */
const armed = new Map<number, number>();
/**
 * How long a row stays armed before disarming itself.
 *
 * Long enough to read a collateral warning naming a dozen ports, short enough
 * that a forgotten confirmation does not sit there waiting for a stray click.
 */
const ARM_TIMEOUT_MS = 6000;

const appWindow = getCurrentWindow();

/* ---------- data ---------- */

async function refresh(manual = false): Promise<void> {
  if (manual) el.refresh.classList.add("spinning");
  try {
    const result = await invoke<ScanResult>("list_ports", {
      includeAll: settings.include_all,
    });
    ports = result.ports;
    lastElapsedMs = result.elapsed_ms;
    render();
    updateStatus();
    el.privileges.textContent = result.elevated
      ? "elevated"
      : "standard user — some owners hidden";
    setDot(ports.length > 0 ? "active" : "idle");
  } catch (error) {
    setDot("error");
    renderPlaceholder("Could not read the socket table", String(error), true);
    setStatus("scan failed");
  } finally {
    if (manual) el.refresh.classList.remove("spinning");
  }
}

async function kill(group: PortGroup, force: boolean): Promise<void> {
  killing.add(group.port);
  render();
  try {
    const outcomes = await invoke<KillOutcome[]>("kill_port", {
      port: group.port,
      protocol: group.protocol,
      force,
    });
    const failed = outcomes.filter(
      (outcome) => outcome.status !== "killed" && outcome.status !== "escalated",
    );
    const summary =
      failed.length === 0
        ? `Port ${group.port} freed`
        : (failed[0]?.detail ?? `Could not free port ${group.port}`);

    setStatus(summary, failed.length === 0 ? "good" : "bad");
    await notify(failed.length === 0 ? "Port freed" : "Kill failed", summary);
  } catch (error) {
    setStatus(String(error), "bad");
  } finally {
    killing.delete(group.port);
    armed.delete(group.port);
    await refresh();
  }
}

async function notify(title: string, body: string): Promise<void> {
  if (!settings.notifications) return;
  try {
    let granted = await isPermissionGranted();
    if (!granted) granted = (await requestPermission()) === "granted";
    if (granted) sendNotification({ title, body });
  } catch {
    // A missing notification is never worth interrupting the flow for.
  }
}

/* ---------- rendering ---------- */

function visiblePorts(): PortGroup[] {
  if (!query) return ports;
  const needle = query.toLowerCase();
  return ports.filter((group) => {
    const process = group.processes[0];
    return (
      String(group.port).includes(needle) ||
      group.protocol.includes(needle) ||
      (group.service ?? "").toLowerCase().includes(needle) ||
      (process?.name ?? "").toLowerCase().includes(needle) ||
      (process?.command ?? "").toLowerCase().includes(needle) ||
      String(process?.pid ?? "").includes(needle)
    );
  });
}

function render(): void {
  const rows = visiblePorts();

  if (rows.length === 0) {
    if (ports.length === 0) {
      renderPlaceholder("No ports in use", "Nothing is listening right now.");
    } else {
      renderPlaceholder("No matches", `Nothing matches “${query}”.`);
    }
    return;
  }

  const fragment = document.createDocumentFragment();
  for (const group of rows) fragment.append(renderRow(group));

  el.list.replaceChildren(fragment);
}

function renderRow(group: PortGroup): HTMLElement {
  const process = group.processes[0];
  const row = document.createElement("div");
  row.className = "row";
  row.setAttribute("role", "listitem");
  if (group.owner_hidden && !process) row.classList.add("hidden-owner");

  const port = document.createElement("div");
  port.className = "port";
  port.textContent = String(group.port);

  const meta = document.createElement("div");
  meta.className = "meta";

  const processLine = document.createElement("div");
  processLine.className = "process";

  if (group.protocol === "udp") {
    const tag = document.createElement("span");
    tag.className = "tag udp";
    tag.textContent = "udp";
    processLine.append(tag);
  }

  const name = document.createElement("span");
  name.className = "name";
  name.textContent = process?.name ?? (group.owner_hidden ? "Hidden owner" : "System");
  processLine.append(name);

  if (process) {
    const pid = document.createElement("span");
    pid.className = "pid";
    pid.textContent = `PID ${process.pid}`;
    processLine.append(pid);
  }

  const detail = document.createElement("div");
  detail.className = "detail";

  // While the row is armed, the second line stops describing the port and
  // starts warning about what else this kill takes down with it. One WSL relay
  // process can front every forwarded port on the machine.
  const alsoHeld = armed.has(group.port) ? otherPortsHeldBy(group) : [];
  if (alsoHeld.length > 0) {
    detail.classList.add("warning");
    detail.textContent = describeCollateral(alsoHeld);
  } else {
    detail.textContent = describe(group, process);
    detail.title = process?.command ?? "";
  }

  meta.append(processLine, detail);

  const button = document.createElement("button");
  button.className = "kill";
  button.type = "button";

  // "armed" and "killing" are separate states: the first has to accept the
  // click that confirms, the second must reject any further ones.
  if (killing.has(group.port)) {
    button.textContent = "Killing…";
    button.dataset.state = "killing";
  } else if (armed.has(group.port)) {
    button.textContent = "Confirm?";
    button.dataset.state = "armed";
  } else {
    button.textContent = "Kill";
  }

  if (!process && group.owner_hidden) {
    button.disabled = true;
    button.title = "The owning process belongs to another user — run Portify elevated";
  } else {
    button.title = settings.confirm_before_kill
      ? "Click, then confirm · Shift-click to force-kill immediately"
      : "Click to free this port · Shift-click to force";
    button.addEventListener("click", (event) => onKillClick(group, event.shiftKey));
  }

  row.append(port, meta, button);
  return row;
}

/** Other ports the same process is holding, which a kill would also free. */
function otherPortsHeldBy(group: PortGroup): number[] {
  const pid = group.processes[0]?.pid;
  if (pid === undefined) return [];
  return ports
    .filter((other) => other.port !== group.port && other.processes[0]?.pid === pid)
    .map((other) => other.port)
    .sort((a, b) => a - b);
}

function describeCollateral(alsoHeld: number[]): string {
  const shown = alsoHeld.slice(0, 6).join(", ");
  if (alsoHeld.length > 6) {
    return `⚠ also holds ${alsoHeld.length} other ports (${shown}, …) — all go down with it`;
  }
  if (alsoHeld.length === 1) {
    return `⚠ also holds port ${shown}, which goes down with it`;
  }
  return `⚠ also holds ports ${shown} — all go down with it`;
}

/** Second line of a row: service name, memory, connections, bind addresses. */
function describe(group: PortGroup, process: ProcessInfo | undefined): string {
  const parts: string[] = [];
  // What the process is, then what the port conventionally means. The first is
  // observed, the second is a guess from a number, so it goes second.
  if (process?.description) parts.push(process.description);
  if (group.service && group.service !== process?.description) parts.push(group.service);
  if (process?.memory_bytes) parts.push(formatBytes(process.memory_bytes));
  if (group.connections > 0) {
    parts.push(`${group.connections} connection${group.connections === 1 ? "" : "s"}`);
  }
  if (group.addresses.length > 0) {
    parts.push(group.addresses.map(withoutPort).join(", "));
  }
  return parts.join("  ·  ");
}

/**
 * Drop the port from a bind address.
 *
 * The port is already the headline of the row, so repeating it once per
 * interface ("10.0.0.5:137, 172.16.0.1:137, 192.168.1.10:137") spends the
 * whole line restating what the reader just read. Handles the bracketed IPv6
 * form too, where the last colon is still the separator.
 */
function withoutPort(address: string): string {
  const separator = address.lastIndexOf(":");
  return separator > 0 ? address.slice(0, separator) : address;
}

function renderPlaceholder(headline: string, body: string, isError = false): void {
  const wrapper = document.createElement("div");
  wrapper.className = isError ? "placeholder error" : "placeholder";

  const title = document.createElement("div");
  title.className = "headline";
  title.textContent = headline;

  const text = document.createElement("div");
  text.textContent = body;

  wrapper.append(title, text);
  el.list.replaceChildren(wrapper);
}

function formatBytes(bytes: number): string {
  const units = ["B", "KB", "MB", "GB"];
  let value = bytes;
  let unit = 0;
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024;
    unit += 1;
  }
  return unit === 0 ? `${bytes} B` : `${value.toFixed(1)} ${units[unit]}`;
}

/**
 * Footer count.
 *
 * Reports what is on screen, not what was scanned. Showing "103 ports" above a
 * list of nine is a plain contradiction, and the number the reader can check by
 * counting is the one that has to be right.
 */
function updateStatus(): void {
  const shown = visiblePorts().length;
  const total = ports.length;
  const noun = shown === 1 ? "port" : "ports";
  const count = shown === total ? `${total} ${noun}` : `${shown} of ${total} ${noun}`;
  setStatus(`${count} · ${lastElapsedMs} ms`);
}

function setStatus(text: string, tone?: "good" | "bad"): void {
  el.status.textContent = text;
  el.status.className = tone ?? "";
}

function setDot(state: "idle" | "active" | "busy" | "error"): void {
  el.dot.dataset.state = state;
}

/* ---------- interaction ---------- */

function onKillClick(group: PortGroup, force: boolean): void {
  if (killing.has(group.port)) return;

  // Already armed: this is the confirming click, whatever modifier it carries.
  const pending = armed.get(group.port);
  if (pending !== undefined) {
    window.clearTimeout(pending);
    armed.delete(group.port);
    void kill(group, force);
    return;
  }

  // Shift is the deliberate escape hatch, so it skips the confirmation step
  // entirely — arming on shift-click would contradict the button's own tooltip
  // and make the modifier look broken.
  if (force || !settings.confirm_before_kill) {
    void kill(group, force);
    return;
  }

  // Confirmation is a second click on the same button rather than a modal:
  // it keeps the "one gesture" promise while still being deliberate.
  const handle = window.setTimeout(() => {
    armed.delete(group.port);
    render();
  }, ARM_TIMEOUT_MS);
  armed.set(group.port, handle);
  render();
}

function restartTimer(): void {
  if (timer !== undefined) window.clearInterval(timer);
  timer = undefined;
  if (settings.refresh_interval_secs <= 0) return;

  timer = window.setInterval(() => {
    // Polling a window nobody is looking at is pure battery cost.
    if (!windowVisible) return;
    // Never rebuild the list in the middle of a destructive decision. A refresh
    // replaces every row, so the button the user is reaching for is destroyed
    // and recreated, and a click that lands in that gap goes nowhere.
    if (armed.size > 0 || killing.size > 0) return;
    void refresh();
  }, settings.refresh_interval_secs * 1000);
}

async function persist(): Promise<void> {
  try {
    await invoke("save_settings", { settings });
  } catch (error) {
    setStatus(`Could not save settings: ${error}`, "bad");
  }
}

const SHORTCUT_HELP =
  "Works from anywhere, even when Portify is hidden. Leave empty to disable.";

function resetShortcutNote(): void {
  el.shortcut.classList.remove("invalid");
  el.shortcutNote.classList.remove("error");
  el.shortcutNote.textContent = SHORTCUT_HELP;
}

function applySettingsToForm(): void {
  el.interval.value = String(settings.refresh_interval_secs);
  el.shortcut.value = settings.shortcut;
  el.notifications.checked = settings.notifications;
  el.confirm.checked = settings.confirm_before_kill;
  el.all.checked = settings.include_all;
  el.hideOnBlur.checked = settings.hide_on_blur;
}

function wireEvents(): void {
  el.search.addEventListener("input", () => {
    query = el.search.value.trim();
    render();
    updateStatus();
  });

  el.refresh.addEventListener("click", () => void refresh(true));

  el.settingsToggle.addEventListener("click", () => {
    el.settings.hidden = !el.settings.hidden;
  });

  el.close.addEventListener("click", () => void appWindow.hide());

  el.interval.addEventListener("change", () => {
    settings.refresh_interval_secs = Number(el.interval.value);
    restartTimer();
    void persist();
  });

  for (const [input, key] of [
    [el.notifications, "notifications"],
    [el.confirm, "confirm_before_kill"],
    [el.hideOnBlur, "hide_on_blur"],
  ] as const) {
    input.addEventListener("change", () => {
      settings[key] = input.checked;
      void persist();
    });
  }

  // Committed on blur or Enter rather than on every keystroke: registering a
  // half-typed accelerator would fail on every character.
  const commitShortcut = async (value?: string): Promise<void> => {
    const wanted = (value ?? el.shortcut.value).trim();
    el.shortcut.value = wanted;
    if (wanted === settings.shortcut) {
      resetShortcutNote();
      return;
    }

    const previous = settings.shortcut;
    settings.shortcut = wanted;
    try {
      await invoke("save_settings", { settings });
      el.shortcut.classList.remove("invalid");
      el.shortcutNote.classList.remove("error");
      el.shortcutNote.textContent = wanted
        ? `Saved. Press ${wanted} from anywhere to show or hide Portify.`
        : "Shortcut disabled.";
    } catch (error) {
      // The backend refuses to persist an accelerator it could not bind, so the
      // running shortcut is still the old one.
      settings.shortcut = previous;
      el.shortcut.classList.add("invalid");
      el.shortcutNote.classList.add("error");
      el.shortcutNote.textContent = String(error);
    }
  };

  el.shortcut.addEventListener("blur", () => void commitShortcut());
  el.shortcutReset.addEventListener("click", () => void commitShortcut(DEFAULT_SETTINGS.shortcut));
  el.shortcut.addEventListener("keydown", (event) => {
    if (event.key === "Enter") {
      event.preventDefault();
      el.shortcut.blur();
    }
    if (event.key === "Escape") {
      event.preventDefault();
      el.shortcut.value = settings.shortcut;
      resetShortcutNote();
      el.shortcut.blur();
    }
  });

  el.all.addEventListener("change", () => {
    settings.include_all = el.all.checked;
    void persist();
    void refresh(true);
  });

  document.addEventListener("keydown", (event) => {
    // The shortcut field handles its own Escape (revert), and the search field
    // needs Escape to clear before anything hides the window.
    if (event.target === el.shortcut) return;
    if (event.key === "Escape") {
      if (query) {
        el.search.value = "";
        query = "";
        render();
      } else {
        void appWindow.hide();
      }
    }
    if (event.key === "f" && (event.ctrlKey || event.metaKey)) {
      event.preventDefault();
      el.search.focus();
      el.search.select();
    }
    if (event.key === "r" && (event.ctrlKey || event.metaKey)) {
      event.preventDefault();
      void refresh(true);
    }
  });

  void appWindow.onFocusChanged(({ payload: focused }) => {
    if (!focused && settings.hide_on_blur) void appWindow.hide();
  });

  // The tray owns window visibility, so it tells the UI when to wake up.
  void listen("portify://shown", () => {
    windowVisible = true;
    void refresh();
  });
  void listen("portify://hidden", () => {
    windowVisible = false;
  });
  void listen("portify://refresh", () => void refresh(true));
}

/* ---------- boot ---------- */

async function boot(): Promise<void> {
  wireEvents();
  try {
    settings = await invoke<Settings>("get_settings");
  } catch {
    settings = { ...DEFAULT_SETTINGS };
  }
  applySettingsToForm();
  restartTimer();
  await refresh(true);
  el.search.focus();

  // The window is hidden until this point, so the first thing the user sees is
  // a populated list rather than an empty frame filling in.
  try {
    await invoke("ready");
  } catch {
    // The Rust side shows the window on a timer regardless.
  }
}

void boot();
