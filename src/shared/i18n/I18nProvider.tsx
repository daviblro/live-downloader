import { createContext, createElement, useContext, useMemo, type ReactNode } from "react";
import type { Locale } from "../contracts";
import { english } from "./locales/en";
import { portugueseBrazil } from "./locales/pt-BR";
import type { Translation } from "./types";

export const translations: Record<Locale, Translation> = { en: english, "pt-BR": portugueseBrazil };

export function localizeRuntimeText(value: string, translation: Translation): string {
  return translation.runtime[value] ?? translation.states[value] ?? value;
}

interface I18nValue {
  locale: Locale;
  translation: Translation;
}

const I18nContext = createContext<I18nValue>({ locale: "en", translation: english });

export function I18nProvider({ locale, children }: { locale: Locale; children: ReactNode }) {
  const value = useMemo(() => ({ locale, translation: translations[locale] ?? english }), [locale]);
  return createElement(I18nContext.Provider, { value }, children);
}

export function useI18n(): I18nValue {
  return useContext(I18nContext);
}
