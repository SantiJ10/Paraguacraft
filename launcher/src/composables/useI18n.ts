import { computed } from "vue";
import { storeToRefs } from "pinia";
import { useSettingsStore } from "@/stores/settings";
import { getLocale, setLocale, t, type Locale, type MessageKey } from "@/i18n";

/** Aplica idioma desde ajustes y expone `t` reactivo al cambiar settings.language. */
export function useI18n() {
  const settings = useSettingsStore();
  const { settings: s } = storeToRefs(settings);

  const locale = computed(() => (s.value?.language as Locale) || "es");

  function sync() {
    const lang = (s.value?.language as Locale) || "es";
    if (getLocale() !== lang) setLocale(lang);
  }
  sync();

  function translate(key: MessageKey) {
    // Dependencia reactiva
    void locale.value;
    sync();
    return t(key);
  }

  function setLanguage(lang: Locale) {
    setLocale(lang);
    settings.update("language", lang);
  }

  return { t: translate, locale, setLanguage };
}
