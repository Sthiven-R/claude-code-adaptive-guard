// Locale-agnostic string utilities. Lives separate from `i18n.ts` so
// that pure utilities like `time.ts` can use the placeholder-substitution
// helper without inverting the dependency direction (a util importing
// from a presentation/locale module).

/**
 * Replace `{name}` placeholders in a template string. Used for strings
 * with dynamic numbers like "Load more ({n} more)". Unknown keys are
 * left as `{name}` rather than `undefined`, matching common i18n
 * library behavior.
 */
export function fmt(
  template: string,
  vars: Record<string, string | number>
): string {
  return template.replace(/\{(\w+)\}/g, (_, k) =>
    k in vars ? String(vars[k]) : `{${k}}`
  );
}
