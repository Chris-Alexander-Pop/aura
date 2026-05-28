import { useId, useMemo, useState } from "react"
import { motion } from "framer-motion"
import { useQuery } from "@tanstack/react-query"
import api from "@/lib/api"
import { getNavItem } from "../navigation"
import {
  filterHyprKeybindSections,
  HYPR_KEYBIND_REFERENCE_DISCLAIMER,
  HYPR_KEYBIND_SECTIONS,
} from "./hyprlandKeybindReference"

export function KeybindsPane() {
  const { icon, label } = getNavItem("keybinds")
  const searchId = useId()
  const [query, setQuery] = useState("")
  const [showLive, setShowLive] = useState(false)

  const sections = useMemo(() => filterHyprKeybindSections(query, HYPR_KEYBIND_SECTIONS), [query])

  const liveQuery = useQuery({
    queryKey: ["keybinds-live"],
    queryFn: () =>
      api.listKeybinds() as Promise<
        Array<{ combo: string; action: string; file: string; category: string }>
      >,
    enabled: showLive,
  })

  return (
    <motion.div
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -10 }}
      transition={{ duration: 0.18 }}
      className="flex flex-col gap-5 p-6 h-full overflow-y-auto"
    >
      <div>
        <div className="flex items-center gap-3 mb-2">
          <span className="icon text-mauve text-2xl">{icon}</span>
          <h2 className="text-xl font-semibold text-text">{label}</h2>
        </div>
        <p className="text-xs font-medium uppercase tracking-wide text-mauve/90">Hyprland reference · not a config editor</p>
        <p className="text-sm text-subtext0 leading-relaxed max-w-prose mt-2">{HYPR_KEYBIND_REFERENCE_DISCLAIMER}</p>
      </div>

      <div className="glass-card p-4 flex flex-col gap-3">
        <div className="flex items-center justify-between gap-3">
          <h3 className="text-sm font-semibold text-text">Live Hyprland binds</h3>
          <button
            type="button"
            className="text-xs px-2 py-1 rounded-lg border border-surface1/60 text-subtext1 hover:text-text"
            onClick={() => setShowLive((v) => !v)}
          >
            {showLive ? "Hide" : "Load from system"}
          </button>
        </div>
        {showLive && liveQuery.isLoading && <p className="text-xs text-subtext0">Reading config…</p>}
        {showLive && liveQuery.isError && (
          <p className="text-xs text-peach">Could not load binds (Hyprland config missing or sidecar down).</p>
        )}
        {showLive && Array.isArray(liveQuery.data) && liveQuery.data.length > 0 && (
          <ul className="max-h-48 overflow-y-auto flex flex-col gap-1.5 text-xs">
            {liveQuery.data.slice(0, 80).map((row, i) => (
              <li key={`${row.combo}-${i}`} className="flex gap-2 border-b border-surface0/30 pb-1">
                <kbd className="font-mono text-mauve shrink-0">{row.combo}</kbd>
                <span className="text-subtext1 truncate">{row.action}</span>
              </li>
            ))}
          </ul>
        )}
      </div>

      <div className="flex flex-col gap-1.5">
        <label htmlFor={searchId} className="text-[11px] uppercase tracking-wide text-subtext1">
          Search reference
        </label>
        <input
          id={searchId}
          type="search"
          autoComplete="off"
          spellCheck={false}
          placeholder="Filter by key chord or action…"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          className="w-full rounded-xl border border-surface0/80 bg-base/80 px-3 py-2.5 text-sm text-text placeholder:text-subtext0 focus:outline-none focus:ring-2 focus:ring-mauve/50"
        />
      </div>

      {sections.length === 0 ? (
        <p className="text-sm text-subtext0 py-4">No rows match that search.</p>
      ) : (
        <div className="flex flex-col gap-4 pb-4">
          {sections.map((sec) => (
            <section key={sec.id} className="glass-card p-4 flex flex-col gap-2.5">
              <h3 className="text-sm font-semibold text-text">{sec.title}</h3>
              <ul className="flex flex-col gap-2">
                {sec.rows.map((row) => (
                  <li
                    key={`${sec.id}-${row.combo}-${row.action}`}
                    className="flex flex-col sm:flex-row sm:items-baseline gap-1 sm:gap-4 text-sm border-b border-surface0/40 pb-2 last:border-0 last:pb-0"
                  >
                    <kbd className="shrink-0 font-mono text-xs text-mauve bg-surface0/60 px-2 py-1 rounded-lg border border-surface0/80">
                      {row.combo}
                    </kbd>
                    <span className="text-subtext1 leading-snug">{row.action}</span>
                  </li>
                ))}
              </ul>
            </section>
          ))}
        </div>
      )}
    </motion.div>
  )
}
