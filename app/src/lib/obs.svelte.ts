// Estado de la conexión con OBS, compartido por la vista de Transmisión y la barra superior.
// Es independiente de la proyección: nada de aquí cambia lo que se ve en el templo.
import { platform, type ObsDetection, type ObsStatus } from "$lib/platform";

const POLL_MS = 1000;
const LAUNCH_WAIT_MS = 25_000;

export const SCENES = {
  camera: "Selah · Cámara",
  cameraText: "Selah · Cámara + texto",
  wait: "Selah · Espera",
} as const;

export const obs = $state({
  detection: null as ObsDetection | null,
  connected: false,
  /** Abriendo OBS o conectando: la interfaz muestra que espera. */
  busy: false,
  status: null as ObsStatus | null,
});

let timer: ReturnType<typeof setInterval> | null = null;

async function poll() {
  if (!obs.connected) return;
  try {
    obs.status = await platform.obsStatus();
  } catch {
    // OBS se cerró o dejó de responder.
    obs.connected = false;
    obs.status = null;
    obs.detection = await platform.obsDetect().catch(() => obs.detection);
  }
}

function startPolling() {
  timer ??= setInterval(poll, POLL_MS);
}

export async function detect() {
  obs.detection = await platform.obsDetect();
  return obs.detection;
}

export async function connect() {
  await platform.obsConnect();
  obs.connected = true;
  startPolling();
  await poll();
}

/** Al abrir Selah: si OBS ya está abierto y preparado, se conecta sin pedir nada. */
export async function init() {
  const found = await detect().catch((e) => {
    platform.logError(`OBS: no se pudo detectar: ${JSON.stringify(e)}`).catch(() => {});
    return null;
  });
  if (!found?.running || !found.websocketEnabled) return;
  // Unos reintentos: al arrancar, OBS o el propio backend pueden tardar un momento en aceptar la conexión.
  for (let attempt = 1; attempt <= 4; attempt++) {
    try {
      await connect();
      return;
    } catch (e) {
      if (attempt === 4) platform.logError(`OBS: conexión automática fallida: ${JSON.stringify(e)}`).catch(() => {});
      else await new Promise((r) => setTimeout(r, 2000));
    }
  }
}

/** Abre OBS y espera a que acepte la conexión. */
export async function launchAndConnect() {
  obs.busy = true;
  try {
    await platform.obsLaunch();
    const deadline = Date.now() + LAUNCH_WAIT_MS;
    for (;;) {
      try {
        await connect();
        return;
      } catch (e) {
        if (Date.now() > deadline) throw e;
        await new Promise((r) => setTimeout(r, 1500));
      }
    }
  } finally {
    obs.busy = false;
    await detect().catch(() => {});
  }
}

export async function disconnect() {
  await platform.obsDisconnect();
  obs.connected = false;
  obs.status = null;
}

export function formatDuration(ms: number): string {
  const total = Math.floor(ms / 1000);
  const h = Math.floor(total / 3600);
  const m = Math.floor((total % 3600) / 60);
  const s = total % 60;
  return `${h}:${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
}
