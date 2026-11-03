/**
 * Separa el texto en diapositivas. Divide una línea que solo tenga dos o más asteriscos (`**`, `***`, `* * *`)
 * o tres o más seguidos en cualquier sitio (`***`, como siempre). Se descartan las diapositivas vacías.
 */
export function splitSlides(text: string): string[] {
  return text
    .split(/^[ \t]*(?:\*[ \t]*){2,}$|\*{3,}/m)
    .map((slide) => slide.trim())
    .filter((slide) => slide.length > 0);
}
