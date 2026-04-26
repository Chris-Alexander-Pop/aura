import { useState } from "react"
import api from "@/lib/api"
import { cn } from "@/lib/utils"

type ConfirmAction = "reboot" | "poweroff"

const actions = [
  { id: "lock", icon: "lock", label: "Lock", run: () => api.sessionLock() },
  { id: "logout", icon: "logout", label: "Log out", run: () => api.sessionLogout() },
  { id: "suspend", icon: "bedtime", label: "Suspend", run: () => api.sessionSuspend() },
] as const

export default function PowerFlyout() {
  const [confirm, setConfirm] = useState<ConfirmAction | null>(null)
  const [busy, setBusy] = useState<string | null>(null)

  const run = async (id: string, fn: () => Promise<unknown>) => {
    setBusy(id)
    try {
      await fn()
    } finally {
      setBusy(null)
    }
  }

  const runConfirmed = async () => {
    if (!confirm) return
    await run(confirm, confirm === "reboot" ? api.sessionReboot : api.sessionPowerOff)
    setConfirm(null)
  }

  return (
    <div className="flex flex-col gap-3 px-3 pb-3 pt-3 text-text">
      <h2 className="text-sm font-semibold leading-tight text-subtext1">Session</h2>

      <div className="grid grid-cols-3 gap-2">
        {actions.map((action) => (
          <button
            key={action.id}
            type="button"
            className="flex min-h-20 flex-col items-center justify-center gap-2 rounded-2xl bg-surface0/90 px-2 py-3 text-subtext1 transition-colors hover:bg-surface1 hover:text-text disabled:opacity-50"
            disabled={busy != null}
            onClick={() => run(action.id, action.run)}
          >
            <span className="icon text-[26px]">{busy === action.id ? "progress_activity" : action.icon}</span>
            <span className="text-[11px] font-medium">{action.label}</span>
          </button>
        ))}
      </div>

      <div className="grid grid-cols-2 gap-2">
        <button
          type="button"
          className="flex min-h-16 flex-col items-center justify-center gap-1 rounded-2xl bg-red/10 px-2 py-3 text-red transition-colors hover:bg-red/20 disabled:opacity-50"
          disabled={busy != null}
          onClick={() => setConfirm("reboot")}
        >
          <span className="icon text-[24px]">restart_alt</span>
          <span className="text-[11px] font-semibold">Reboot</span>
        </button>
        <button
          type="button"
          className="flex min-h-16 flex-col items-center justify-center gap-1 rounded-2xl bg-red/10 px-2 py-3 text-red transition-colors hover:bg-red/20 disabled:opacity-50"
          disabled={busy != null}
          onClick={() => setConfirm("poweroff")}
        >
          <span className="icon text-[24px]">power_settings_new</span>
          <span className="text-[11px] font-semibold">Power off</span>
        </button>
      </div>

      {confirm ? (
        <div className="rounded-2xl border border-red/30 bg-red/10 p-3">
          <p className="text-[12px] font-medium text-red">
            Confirm {confirm === "reboot" ? "reboot" : "power off"}?
          </p>
          <div className="mt-3 flex gap-2">
            <button
              type="button"
              className={cn(
                "flex-1 rounded-full bg-red px-3 py-2 text-[12px] font-semibold text-crust",
                busy === confirm && "opacity-70"
              )}
              disabled={busy != null}
              onClick={runConfirmed}
            >
              {busy === confirm ? "Working..." : "Confirm"}
            </button>
            <button
              type="button"
              className="flex-1 rounded-full bg-surface1 px-3 py-2 text-[12px] font-semibold text-subtext1 hover:text-text"
              disabled={busy != null}
              onClick={() => setConfirm(null)}
            >
              Cancel
            </button>
          </div>
        </div>
      ) : null}
    </div>
  )
}
