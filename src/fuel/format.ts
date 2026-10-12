// Exact text → integer parsing for form input and `pl-PL` display formatting.
// Stored values are never rounded; rounding to 2 places happens only here, for display.

const DECIMAL2 = /^(\d+)(?:[.,](\d{1,2}))?$/;

/**
 * Parses a non-negative decimal with at most 2 fraction digits (`,` or `.` as separator, no sign,
 * no thousands separators) into hundredths, computed from the text parts without float maths:
 * `"46,35"` → 4635, `"7,7"` → 770, `"5"` → 500, `"1,234"` → null.
 */
export function parseDecimal2(text: string): number | null {
  const match = DECIMAL2.exec(text.trim());
  if (!match) return null;
  const whole = Number(match[1]);
  const fraction = Number((match[2] ?? "").padEnd(2, "0"));
  const value = whole * 100 + fraction;
  return Number.isSafeInteger(value) ? value : null;
}

/** Parses whole kilometres: digits only, any whitespace (e.g. `100 400`) ignored. */
export function parseKm(text: string): number | null {
  const digits = text.replace(/\s/g, "");
  if (!/^\d+$/.test(digits)) return null;
  const value = Number(digits);
  return Number.isSafeInteger(value) ? value : null;
}

/** True for a `YYYY-MM-DD` string as produced by `<input type="date">`; calendar validity is the backend's job. */
export function isIsoDate(text: string): boolean {
  return /^\d{4}-\d{2}-\d{2}$/.test(text);
}

/** Today's local date as `YYYY-MM-DD` (not UTC, which can be a day off). */
export function todayIso(): string {
  const now = new Date();
  const month = String(now.getMonth() + 1).padStart(2, "0");
  const day = String(now.getDate()).padStart(2, "0");
  return `${now.getFullYear()}-${month}-${day}`;
}

const TWO_PLACES = new Intl.NumberFormat("pl-PL", { minimumFractionDigits: 2, maximumFractionDigits: 2 });
const WHOLE = new Intl.NumberFormat("pl-PL", { maximumFractionDigits: 0 });

/** `9` → `"9,00 l/100 km"`. */
export function formatConsumption(litresPer100Km: number): string {
  return `${TWO_PLACES.format(litresPer100Km)} l/100 km`;
}

/** Grosze → `"137,00 zł"`. */
export function formatMoney(grosze: number): string {
  return `${TWO_PLACES.format(grosze / 100)} zł`;
}

/** Grosze per litre → `"3,05 zł/l"`. */
export function formatPricePerLitre(groszePerLitre: number): string {
  return `${TWO_PLACES.format(groszePerLitre / 100)} zł/l`;
}

/** `100400` → `"100 400 km"`. */
export function formatKm(km: number): string {
  return `${WHOLE.format(km)} km`;
}

/** `YYYY-MM-DD` → `DD.MM.YYYY`; anything else is returned unchanged. */
export function formatDate(iso: string): string {
  const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(iso);
  return match ? `${match[3]}.${match[2]}.${match[1]}` : iso;
}
