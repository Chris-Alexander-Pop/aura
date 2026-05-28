import { motion } from "framer-motion"
import { useQuery } from "@tanstack/react-query"
import api from "@/lib/api"
import { cn } from "@/lib/utils"
import { getNavItem } from "../navigation"

type DisplayRule = {
  key: string
  title: string
  subtitle?: string
  enabled?: boolean
}

function summarizeJson(value: unknown): string | undefined {
  try {
    const s = JSON.stringify(value)
    if (!s || s === "null") return undefined
    return s.length > 120 ? `${s.slice(0, 117)}…` : s
  } catch {
    return undefined
  }
}

function normalizeRule(raw: unknown, index: number): DisplayRule {
  if (typeof raw === "string") {
    return { key: `str-${index}`, title: raw }
  }

  if (raw && typeof raw === "object" && !Array.isArray(raw)) {
    const o = raw as Record<string, unknown>
    const title =
      (typeof o.name === "string" && o.name.trim()) ||
      (typeof o.title === "string" && o.title.trim()) ||
      (typeof o.label === "string" && o.label.trim()) ||
      (typeof o.id === "string" && o.id.trim()) ||
      `Rule ${index + 1}`

    let subtitle: string | undefined =
      typeof o.description === "string" && o.description.trim()
        ? o.description.trim()
        : undefined
    if (!subtitle) subtitle = summarizeJson(o.summary ?? o.condition ?? o.trigger ?? o.triggers)

    const enabled = typeof o.enabled === "boolean" ? o.enabled : undefined
    const key =
      (typeof o.id === "string" && o.id) ||
      (typeof o.slug === "string" && o.slug) ||
      `rule-${index}`

    return { key, title, subtitle, enabled }
  }

  return {
    key: `rule-${index}`,
    title: `Rule ${index + 1}`,
    subtitle: summarizeJson(raw),
  }
}

export function AutomationsPane() {
  const { icon, label } = getNavItem("automations")
  const { data, isLoading, isError, error, refetch, isFetching } = useQuery({
    queryKey: ["automation-rules"],
    queryFn: api.getAutomationRules,
    staleTime: 30_000,
  })

  const rules: DisplayRule[] = Array.isArray(data)
    ? data.map((raw, i) => normalizeRule(raw, i))
    : []

  const showEmpty = !isLoading && !isError && rules.length === 0

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
            Automation rules from the sidecar. Create and edit flows in roadmap tooling when wired up.
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

      {isLoading ? (
        <div className="flex flex-col gap-3">
          {[...Array(4)].map((_, i) => (
            <div key={i} className="skeleton h-20 rounded-xl" style={{ opacity: 1 - i * 0.18 }} />
          ))}
        </div>
      ) : isError ? (
        <div className="glass-card p-5 flex flex-col gap-3 border-red/25">
          <div className="flex items-center gap-2 text-red">
            <span className="icon text-xl">error</span>
            <p className="text-sm font-medium">Could not load rules</p>
          </div>
          <p className="text-xs text-subtext0 leading-relaxed">
            {error instanceof Error ? error.message : "Sidecar request failed."}
          </p>
          <button type="button" className="btn-surface text-sm self-start" onClick={() => refetch()}>
            Try again
          </button>
        </div>
      ) : showEmpty ? (
        <div className="flex flex-1 flex-col items-center justify-center gap-4 py-12 px-4 text-center">
          <span className="icon text-5xl text-subtext0/70">{icon}</span>
          <div className="max-w-sm">
            <p className="text-sm font-medium text-text">No automation rules yet</p>
            <p className="text-xs text-subtext1 mt-2 leading-relaxed">
              When workflows or rules are registered in Aura, they will appear here as cards you can scan at a glance.
            </p>
          </div>
        </div>
      ) : (
        <ul className="flex flex-col gap-3 list-none m-0 p-0">
          {rules.map((rule) => (
            <li key={rule.key}>
              <div className="glass-card p-4 flex flex-col gap-2 hover:bg-surface1/40 transition-colors">
                <div className="flex items-start justify-between gap-3">
                  <p className="text-sm font-medium text-text leading-snug">{rule.title}</p>
                  {rule.enabled !== undefined && (
                    <span
                      className={cn(
                        "text-[10px] uppercase tracking-wide px-2 py-0.5 rounded-full shrink-0",
                        rule.enabled
                          ? "bg-green/15 text-green border border-green/30"
                          : "bg-surface0 text-subtext1 border border-surface0"
                      )}
                    >
                      {rule.enabled ? "On" : "Off"}
                    </span>
                  )}
                </div>
                {rule.subtitle && (
                  <p className="text-xs text-subtext0 font-mono leading-relaxed break-all">{rule.subtitle}</p>
                )}
              </div>
            </li>
          ))}
        </ul>
      )}
    </motion.div>
  )
}
