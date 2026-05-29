import { motion } from "framer-motion"
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { useState } from "react"
import api, { type WorkflowView } from "@/lib/api"
import { cn } from "@/lib/utils"
import { getNavItem } from "../navigation"

export function AutomationsPane() {
  const { icon, label } = getNavItem("automations")
  const qc = useQueryClient()
  const [name, setName] = useState("")
  const [actionsJson, setActionsJson] = useState(
    '[{"type":"send_notification","title":"Aura","body":"Workflow ran"}]'
  )

  const { data, isLoading, isError, error, refetch, isFetching } = useQuery({
    queryKey: ["automation-rules"],
    queryFn: api.getAutomationRules,
    staleTime: 30_000,
  })

  const createMut = useMutation({
    mutationFn: () => {
      let actions: unknown
      try {
        actions = JSON.parse(actionsJson)
      } catch {
        throw new Error("Actions must be valid JSON array")
      }
      return api.createAutomationWorkflow({ name: name.trim() || "New workflow", actions })
    },
    onSuccess: () => {
      setName("")
      void qc.invalidateQueries({ queryKey: ["automation-rules"] })
    },
  })

  const triggerMut = useMutation({
    mutationFn: (workflowId: string) => api.triggerAutomation(workflowId),
    onSuccess: () => void qc.invalidateQueries({ queryKey: ["automation-rules"] }),
  })

  const toggleMut = useMutation({
    mutationFn: ({ id, enabled }: { id: string; enabled: boolean }) =>
      enabled ? api.disableAutomationWorkflow(id) : api.enableAutomationWorkflow(id),
    onSuccess: () => void qc.invalidateQueries({ queryKey: ["automation-rules"] }),
  })

  const deleteMut = useMutation({
    mutationFn: (id: string) => api.deleteAutomationWorkflow(id),
    onSuccess: () => void qc.invalidateQueries({ queryKey: ["automation-rules"] }),
  })

  const rules: WorkflowView[] = Array.isArray(data) ? data : []

  return (
    <motion.div
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -10 }}
      transition={{ duration: 0.18 }}
      className="flex flex-col gap-4 p-6 h-full overflow-y-auto"
    >
      <div className="flex items-start justify-between gap-3">
        <div>
          <div className="flex items-center gap-3 mb-1">
            <span className="icon text-mauve text-2xl">{icon}</span>
            <h2 className="text-xl font-semibold text-text">{label}</h2>
          </div>
          <p className="text-xs text-subtext1 max-w-prose">
            Workflows stored in the sidecar. Run now uses a safe action allowlist (notify-send and
            scripts under ~/.config/ags/automation/scripts).
          </p>
        </div>
        <button
          type="button"
          className="btn-surface text-xs shrink-0"
          onClick={() => refetch()}
          disabled={isFetching}
        >
          <span className={cn("icon text-base", isFetching && "animate-spin")}>refresh</span>
          Refresh
        </button>
      </div>

      <div className="glass-card p-4 flex flex-col gap-3">
        <p className="text-xs font-medium text-text">New workflow</p>
        <input
          className="w-full rounded-xl border border-surface0/80 bg-base/80 px-3 py-2 text-sm"
          placeholder="Workflow name"
          value={name}
          onChange={(e) => setName(e.target.value)}
        />
        <textarea
          className="w-full rounded-xl border border-surface0/80 bg-base/80 px-3 py-2 text-xs font-mono min-h-[72px]"
          value={actionsJson}
          onChange={(e) => setActionsJson(e.target.value)}
        />
        <button
          type="button"
          className="btn-surface text-sm self-start"
          disabled={createMut.isPending}
          onClick={() => createMut.mutate()}
        >
          Create workflow
        </button>
        {createMut.isError && (
          <p className="text-xs text-red">
            {createMut.error instanceof Error ? createMut.error.message : "Create failed"}
          </p>
        )}
      </div>

      {isLoading ? (
        <div className="flex flex-col gap-3">
          {[...Array(3)].map((_, i) => (
            <div key={i} className="skeleton h-20 rounded-xl" />
          ))}
        </div>
      ) : isError ? (
        <div className="glass-card p-5 border-red/25">
          <p className="text-sm text-red">{error instanceof Error ? error.message : "Load failed"}</p>
        </div>
      ) : rules.length === 0 ? (
        <div className="glass-card p-8 text-center text-subtext1 text-sm">No workflows yet.</div>
      ) : (
        <ul className="flex flex-col gap-3 list-none m-0 p-0">
          {rules.map((rule) => (
            <li key={rule.id}>
              <div className="glass-card p-4 flex flex-col gap-3">
                <div className="flex items-start justify-between gap-2">
                  <p className="text-sm font-medium text-text">{rule.name}</p>
                  <span
                    className={cn(
                      "text-[10px] uppercase px-2 py-0.5 rounded-full",
                      rule.enabled
                        ? "bg-green/15 text-green"
                        : "bg-surface0 text-subtext1"
                    )}
                  >
                    {rule.enabled ? "On" : "Off"}
                  </span>
                </div>
                <div className="flex flex-wrap gap-2">
                  <button
                    type="button"
                    className="btn-surface text-xs"
                    disabled={triggerMut.isPending || !rule.enabled}
                    onClick={() => triggerMut.mutate(rule.id)}
                  >
                    Run now
                  </button>
                  <button
                    type="button"
                    className="btn-surface text-xs"
                    onClick={() => toggleMut.mutate({ id: rule.id, enabled: rule.enabled })}
                  >
                    {rule.enabled ? "Disable" : "Enable"}
                  </button>
                  <button
                    type="button"
                    className="btn-surface text-xs text-red"
                    onClick={() => deleteMut.mutate(rule.id)}
                  >
                    Delete
                  </button>
                </div>
              </div>
            </li>
          ))}
        </ul>
      )}
    </motion.div>
  )
}
