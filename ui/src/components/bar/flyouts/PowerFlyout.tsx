import { useState } from "react"
import api from "@/lib/api"
import {
  FlyoutActionRow,
  FlyoutShell,
  FlyoutTitle,
} from "@/components/bar/flyouts/FlyoutPrimitives"
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
    <FlyoutShell>
      <FlyoutTitle>Session</FlyoutTitle>

      <div className="flex flex-col gap-0.5">
        {actions.map((action) => (
          <FlyoutActionRow
            key={action.id}
            icon={action.icon}
            label={action.label}
            disabled={busy != null}
            loading={busy === action.id}
            onClick={() => run(action.id, action.run)}
          />
        ))}
      </div>

      <div className="mt-1 flex flex-col gap-0.5 border-t border-surface0/40 pt-1.5">
        <FlyoutActionRow
          icon="restart_alt"
          label="Reboot"
          destructive
          disabled={busy != null}
          onClick={() => setConfirm("reboot")}
        />
        <FlyoutActionRow
          icon="power_settings_new"
          label="Power off"
          destructive
          disabled={busy != null}
          onClick={() => setConfirm("poweroff")}
        />
      </div>

      {confirm ? (
        <div className="rounded-lg border border-red/25 bg-red/10 p-2">
          <p className="text-[10px] font-medium text-red">
            Confirm {confirm === "reboot" ? "reboot" : "power off"}?
          </p>
          <div className="mt-2 flex gap-1.5">
            <button
              type="button"
              className={cn(
                "flex flex-1 items-center justify-center gap-1 rounded-full bg-red px-2 py-1.5 text-[10px] font-semibold text-crust",
                busy === confirm && "opacity-70",
              )}
              disabled={busy != null}
              onClick={runConfirmed}
            >
              {busy === confirm ? (
                <>
                  <span className="icon animate-spin text-sm">progress_activity</span>
                  Working…
                </>
              ) : (
                "Confirm"
              )}
            </button>
            <button
              type="button"
              className="flex-1 rounded-full bg-surface1 px-2 py-1.5 text-[10px] font-semibold text-subtext1 hover:text-text"
              disabled={busy != null}
              onClick={() => setConfirm(null)}
            >
              Cancel
            </button>
          </div>
        </div>
      ) : null}
    </FlyoutShell>
  )
}
