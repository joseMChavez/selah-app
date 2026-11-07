// Planes del culto (D-019): lo que se va a presentar, en orden, preparado de antemano.
// Se guardan en la base de datos local; aquí se guarda una copia para la interfaz.
// Como la biblioteca de letras, cada cambio se aplica primero en la copia y luego se guarda el plan entero.
import { SvelteSet } from "svelte/reactivity";
import { setSetting, settings, storage } from "$lib/core/settings.svelte";
import { platform, type Plan, type PlanItem } from "$lib/platform";

export type { Plan, PlanItem };

export const plans = $state({
  all: [] as Plan[],
  /** Lo que ocupan las copias de los medios de los planes, en bytes; null hasta saberlo. */
  filesBytes: null as number | null,
});

/** Archivos del plan abierto que no están en este equipo (movidos, borrados, plan hecho en otra computadora). */
export const missingFiles = new SvelteSet<string>();

/** El plan abierto en la pestaña Plan (y al que «+ Plan» añade letras). */
export const openPlan = {
  get id(): string | null {
    const id = settings.plan;
    return id && plans.all.some((p) => p.id === id) ? id : null;
  },
  get plan(): Plan | null {
    return plans.all.find((p) => p.id === settings.plan) ?? null;
  },
};

function failed(action: string, e: unknown) {
  storage.error = (e as { message?: string })?.message ?? String(e);
  platform.logError(`planes: ${action}: ${JSON.stringify(e)}`).catch(() => {});
}

/** El mismo orden que la base: con fecha primero, del más próximo al más lejano; después los que no tienen fecha. */
function sortPlans() {
  plans.all.sort((a, b) => {
    if (a.date !== b.date) {
      if (a.date === null) return 1;
      if (b.date === null) return -1;
      return a.date < b.date ? -1 : 1;
    }
    return a.name.localeCompare(b.name);
  });
}

/** Hoy en el reloj de este equipo, «AAAA-MM-DD». */
export function today(): string {
  const d = new Date();
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
}

async function checkFiles(id: string | null) {
  missingFiles.clear();
  if (!id) return;
  try {
    for (const path of await platform.planAllowFiles(id)) missingFiles.add(path);
  } catch (e) {
    failed("comprobar archivos", e);
  }
}

/** Borra las copias de medios que ya no usa ningún plan (al abrir Selah y al borrar planes). */
export async function cleanFiles() {
  try {
    plans.filesBytes = await platform.plansCleanFiles();
  } catch (e) {
    failed("limpiar copias", e);
  }
}

export async function loadPlans() {
  try {
    plans.all = await platform.plansList();
    await checkFiles(openPlan.id);
  } catch (e) {
    failed("leer", e);
  }
  cleanFiles();
}

export function choosePlan(id: string | null) {
  setSetting("plan", id);
  checkFiles(id);
}

export function savePlan(plan: Plan) {
  const snapshot = $state.snapshot(plan) as Plan;
  const i = plans.all.findIndex((p) => p.id === plan.id);
  if (i >= 0) plans.all[i] = snapshot;
  else plans.all.push(snapshot);
  sortPlans();
  platform.planSave(snapshot).catch((e) => failed("guardar", e));
}

/** Crea un plan vacío y lo abre. */
export function createPlan(name: string, date: string | null): Plan {
  const plan: Plan = { id: crypto.randomUUID(), name, date, notes: "", items: [] };
  savePlan(plan);
  choosePlan(plan.id);
  return plan;
}

/** Copia de un plan (por ejemplo, el del domingo pasado como base del próximo), sin fecha, y la abre. */
export function duplicatePlan(plan: Plan, name: string) {
  const copy: Plan = {
    ...($state.snapshot(plan) as Plan),
    id: crypto.randomUUID(),
    name,
    date: null,
    items: plan.items.map((item) => ({ ...item, id: crypto.randomUUID() })),
  };
  savePlan(copy);
  choosePlan(copy.id);
}

export function deletePlan(id: string) {
  plans.all = plans.all.filter((p) => p.id !== id);
  if (settings.plan === id) choosePlan(null);
  platform.planDelete(id).then(cleanFiles, (e) => failed("borrar", e));
}

/** Añade al final del plan abierto (desde otros paneles, como Textos). */
export function addToOpenPlan(item: PlanItem) {
  const plan = openPlan.plan;
  if (plan) savePlan({ ...plan, items: [...plan.items, item] });
}

/** Borra todos los planes. */
export async function resetPlans() {
  await platform.plansReset();
  plans.all = [];
  choosePlan(null);
  await cleanFiles();
}
