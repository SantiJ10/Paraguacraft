import { onScopeDispose, ref, watch, type Ref } from "vue";

/**
 * Copia de `source` que se actualiza recién cuando el usuario deja de escribir.
 *
 * Los filtros de mods recorren cientos de elementos y reconstruyen el DOM; sin
 * esto una búsqueda de 10 letras dispara 10 pasadas completas.
 */
export function useDebounced<T>(source: Ref<T>, delay = 180): Ref<T> {
  const debounced = ref(source.value) as Ref<T>;
  let timer: ReturnType<typeof setTimeout> | null = null;

  const clear = () => {
    if (timer !== null) {
      clearTimeout(timer);
      timer = null;
    }
  };

  watch(source, (value) => {
    clear();
    // Vaciar el buscador debe responder al instante.
    if (typeof value === "string" && value === "") {
      debounced.value = value;
      return;
    }
    timer = setTimeout(() => {
      debounced.value = value;
      timer = null;
    }, delay);
  });

  onScopeDispose(clear);

  return debounced;
}
