import type { Locale } from "../types";

const localeTags: Record<Locale, string> = {
  en: "en-US",
  "pt-BR": "pt-BR",
};

const dateTimeOptions: Intl.DateTimeFormatOptions = {
  year: "numeric",
  month: "2-digit",
  day: "2-digit",
  hour: "2-digit",
  minute: "2-digit",
};

const dateTimeFormatters: Record<Locale, Intl.DateTimeFormat> = {
  en: new Intl.DateTimeFormat(localeTags.en, dateTimeOptions),
  "pt-BR": new Intl.DateTimeFormat(localeTags["pt-BR"], dateTimeOptions),
};

const timeFormatters: Record<Locale, Intl.DateTimeFormat> = {
  en: new Intl.DateTimeFormat(localeTags.en, { hour: "2-digit", minute: "2-digit" }),
  "pt-BR": new Intl.DateTimeFormat(localeTags["pt-BR"], { hour: "2-digit", minute: "2-digit" }),
};

export function formatDateTime(value: string | Date, locale: Locale): string {
  return dateTimeFormatters[locale].format(typeof value === "string" ? new Date(value) : value);
}

export function formatTime(value: string | Date, locale: Locale): string {
  return timeFormatters[locale].format(typeof value === "string" ? new Date(value) : value);
}
