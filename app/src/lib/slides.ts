/** Separa el texto en diapositivas usando *** como divisor. Se descartan las diapositivas vacías. */
export function splitSlides(text: string): string[] {
  return text
    .split(/\*{3,}/)
    .map((slide) => slide.trim())
    .filter((slide) => slide.length > 0);
}
