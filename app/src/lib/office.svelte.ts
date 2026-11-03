// LibreOffice para las presentaciones, compartido por Medios y Ajustes. Si el equipo no tiene con qué
// convertir (o falla), Selah ofrece descargar su propia copia; solo se descarga cuando el usuario lo pide.
import { toMessage, type Message } from "$lib/messages";
import { platform, type OfficeProgress, type OfficeStatus } from "$lib/platform";

export const office = $state({
  status: null as OfficeStatus | null,
  /** Avance de la descarga en curso; null si no se está descargando. */
  progress: null as OfficeProgress | null,
  error: null as Message | null,
});

let listening = false;

export async function loadOffice() {
  office.status = await platform.officeStatus();
  if (!listening) {
    listening = true;
    await platform.onOfficeProgress((p) => {
      if (office.progress) office.progress = p;
    });
  }
}

/** Se puede ofrecer la descarga: no hay LibreOffice y hay copia para este sistema. */
export function canOfferOffice(): boolean {
  const s = office.status;
  return !!s && s.downloadable && !s.system && !s.privateCopy;
}

/** true si la copia quedó lista. */
export async function installOffice(): Promise<boolean> {
  if (office.progress) return false;
  office.error = null;
  office.progress = { downloaded: 0, total: office.status?.size ?? 0, extracting: false };
  try {
    await platform.officeInstall();
    return true;
  } catch (e) {
    if ((e as { code?: string })?.code !== "officeCanceled") office.error = toMessage(e);
    return false;
  } finally {
    office.progress = null;
    await loadOffice();
  }
}

export function cancelOffice() {
  platform.officeCancel();
}

export async function removeOffice() {
  office.error = null;
  try {
    await platform.officeRemove();
  } catch (e) {
    office.error = toMessage(e);
  } finally {
    await loadOffice();
  }
}

/** Bytes en MB, redondeados. */
export const megabytes = (bytes: number) => Math.round(bytes / 1_000_000);
