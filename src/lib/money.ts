/** Convert a peso input string to integer centavos. Never uses floating point. */
export function pesosToCentavos(raw: string): number | null {
  const trimmed = raw.trim().replace(/,/g, "");
  if (!/^\d+(\.\d{0,2})?$/.test(trimmed)) {
    return null;
  }
  const [whole, fraction = ""] = trimmed.split(".");
  const frac = (fraction + "00").slice(0, 2);
  const pesos = Number.parseInt(whole, 10);
  const cents = Number.parseInt(frac, 10);
  if (!Number.isSafeInteger(pesos) || !Number.isSafeInteger(cents)) {
    return null;
  }
  return pesos * 100 + cents;
}

export function centavosToPesosInput(centavos: number): string {
  const abs = Math.abs(centavos);
  const whole = Math.floor(abs / 100);
  const frac = abs % 100;
  return `${whole}.${frac.toString().padStart(2, "0")}`;
}

export function formatPhp(centavos: number): string {
  const negative = centavos < 0;
  const abs = Math.abs(centavos);
  const whole = Math.floor(abs / 100);
  const frac = abs % 100;
  const grouped = whole.toLocaleString("en-PH");
  return `${negative ? "−" : ""}₱${grouped}.${frac.toString().padStart(2, "0")}`;
}
