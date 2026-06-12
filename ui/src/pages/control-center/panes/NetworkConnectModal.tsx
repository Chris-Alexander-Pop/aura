import { useEffect, useId, useRef, useState } from "react"
import { AnimatePresence, motion } from "framer-motion"
import { cn } from "@/lib/utils"

export type NetworkConnectModalProps = {
  open: boolean
  ssid: string
  securityLabel: string
  onClose: () => void
  onConnect: (password: string) => Promise<void>
}

export function NetworkConnectModal({
  open,
  ssid,
  securityLabel,
  onClose,
  onConnect,
}: NetworkConnectModalProps) {
  const titleId = useId()
  const passwordRef = useRef<HTMLInputElement>(null)
  const [password, setPassword] = useState("")
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const [displaySsid, setDisplaySsid] = useState(ssid)
  const [displaySecurity, setDisplaySecurity] = useState(securityLabel)

  useEffect(() => {
    if (open && ssid) {
      setDisplaySsid(ssid)
      setDisplaySecurity(securityLabel)
    }
    if (!open) return
    setPassword("")
    setError(null)
  }, [open, ssid, securityLabel])

  useEffect(() => {
    if (!open) return
    const t = window.setTimeout(() => passwordRef.current?.focus(), 50)
    return () => window.clearTimeout(t)
  }, [open, displaySsid])

  useEffect(() => {
    if (!open) return
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape" && !busy) onClose()
    }
    window.addEventListener("keydown", onKey)
    return () => window.removeEventListener("keydown", onKey)
  }, [open, busy, onClose])

  const submit = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!password.trim()) {
      setError("Enter the network password.")
      return
    }
    setBusy(true)
    setError(null)
    try {
      await onConnect(password)
      onClose()
    } catch (err) {
      setError(err instanceof Error ? err.message : "Connection failed")
    } finally {
      setBusy(false)
    }
  }

  return (
    <AnimatePresence>
      {open ? (
        <>
          <motion.button
            type="button"
            aria-label="Close dialog"
            className="fixed inset-0 z-[80] bg-crust/55 backdrop-blur-md"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            onClick={() => !busy && onClose()}
          />
          <motion.div
            role="dialog"
            aria-modal="true"
            aria-labelledby={titleId}
            className={cn(
              "fixed left-1/2 top-1/2 z-[90] w-[min(100%-2rem,22rem)] -translate-x-1/2 -translate-y-1/2 rounded-2xl border border-surface0/80",
              "bg-mantle/95 backdrop-blur-xl shadow-2xl p-5 text-text"
            )}
            initial={{ opacity: 0, scale: 0.96, y: 8 }}
            animate={{ opacity: 1, scale: 1, y: 0 }}
            exit={{ opacity: 0, scale: 0.98, y: 6 }}
            transition={{ type: "spring", stiffness: 420, damping: 32 }}
          >
            <div className="flex items-start gap-3 mb-4">
              <span className="icon text-2xl text-mauve shrink-0">lock</span>
              <div className="min-w-0">
                <h3 id={titleId} className="text-lg font-semibold leading-tight truncate" title={displaySsid}>
                  {displaySsid}
                </h3>
                <p className="text-xs text-subtext0 mt-0.5">{displaySecurity}</p>
              </div>
            </div>

            <form onSubmit={submit} className="flex flex-col gap-3">
              <label className="flex flex-col gap-1.5">
                <span className="text-[11px] uppercase tracking-wide text-subtext1">Password</span>
                <input
                  ref={passwordRef}
                  type="password"
                  autoComplete="current-password"
                  className="w-full rounded-xl border border-surface0/80 bg-base/80 px-3 py-2.5 text-sm text-text placeholder:text-subtext0 focus:outline-none focus:ring-2 focus:ring-mauve/50"
                  placeholder="Network password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  disabled={busy}
                />
              </label>

              {error ? (
                <p className="text-xs text-red text-pretty" role="alert">
                  {error}
                </p>
              ) : null}

              <div className="flex gap-2 pt-1">
                <button
                  type="button"
                  className="flex-1 rounded-xl border border-surface0/70 py-2.5 text-sm font-medium text-subtext1 hover:bg-surface0/50 disabled:opacity-50"
                  onClick={() => !busy && onClose()}
                  disabled={busy}
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  className="flex-1 rounded-xl bg-mauve py-2.5 text-sm font-semibold text-base hover:brightness-110 disabled:opacity-50"
                  disabled={busy || !password.trim()}
                >
                  {busy ? "Connecting…" : "Connect"}
                </button>
              </div>
            </form>
          </motion.div>
        </>
      ) : null}
    </AnimatePresence>
  )
}
