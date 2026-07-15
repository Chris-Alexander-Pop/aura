import { type ClassValue, clsx } from "clsx"
import { twMerge } from "tailwind-merge"

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

/** `System.GetStats` cpu/ram/gpu/storage are 0–1 fractions (also accept legacy 0–100). */
export function usageToPercent(value: number): number {
  if (!Number.isFinite(value)) return 0
  const pct = value <= 1 ? value * 100 : value
  return Math.min(Math.max(pct, 0), 100)
}

/** Map °C to a 0–100 progress fill (full bar ≈ `scaleMax`°C). */
export function tempToBarPercent(tempC: number, scaleMax = 100): number {
  if (!Number.isFinite(tempC) || tempC <= 0) return 0
  return Math.min((tempC / scaleMax) * 100, 100)
}
