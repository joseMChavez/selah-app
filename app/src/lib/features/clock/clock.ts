// Reloj y temporizador. La salida los dibuja con su propio reloj: el backend solo guarda cuándo termina la cuenta.
import type { OutputContent } from "$lib/platform";

export type Timer = Extract<OutputContent, { kind: "timer" }>;

/** Una cuenta preparada (parada), con su título y su fondo si los tiene. */
export function preparedTimer(durationMs: number, label = "", background: string | null = null): Timer {
  return { kind: "timer", durationMs, endsAt: null, remainingMs: durationMs, label, background };
}

/** La misma cuenta desde el principio y en marcha; conserva el título y el fondo. */
export function startTimer(timer: Timer, now = Date.now()): Timer {
  return { ...timer, endsAt: now + timer.durationMs, remainingMs: timer.durationMs };
}

/** Lo que queda; negativo cuando ya se pasó del tiempo. */
export function timerRemaining(timer: Timer, now: number): number {
  return timer.endsAt === null ? timer.remainingMs : timer.endsAt - now;
}

export function pauseTimer(timer: Timer, now: number): Timer {
  return { ...timer, endsAt: null, remainingMs: timerRemaining(timer, now) };
}

export function resumeTimer(timer: Timer, now: number): Timer {
  return { ...timer, endsAt: now + timer.remainingMs };
}

/** 4:05, 1:02:09; con «−» delante cuando se pasó del tiempo. Redondea hacia arriba: 0:00 solo al terminar. */
export function formatCountdown(ms: number): string {
  const over = ms <= -1000;
  const total = over ? Math.floor(-ms / 1000) : Math.max(0, Math.ceil(ms / 1000));
  const h = Math.floor(total / 3600);
  const m = Math.floor((total % 3600) / 60);
  const s = String(total % 60).padStart(2, "0");
  const body = h > 0 ? `${h}:${String(m).padStart(2, "0")}:${s}` : `${m}:${s}`;
  return over ? `−${body}` : body;
}

export function formatClock(date: Date, locale: string, hour12: boolean, seconds: boolean): string {
  return date.toLocaleTimeString(locale, { hour: "numeric", minute: "2-digit", second: seconds ? "2-digit" : undefined, hour12 });
}
