import { useId, useMemo, useState } from "react"
import { motion } from "framer-motion"
import { useMutation, useQuery } from "@tanstack/react-query"
import api from "@/lib/api"
import { getNavItem } from "../navigation"
import {
  filterHyprKeybindSections,
  HYPR_KEYBIND_REFERENCE_DISCLAIMER,
  HYPR_KEYBIND_SECTIONS,
} from "./hyprlandKeybindReference"

function filterLiveBinds(
  rows: Array<{ combo: string; action: string; file: string; category: string }>,
  query: string
) {
  const q = query.trim().toLowerCase()
  if (!q) return rows
  return rows.filter(
    (row) =>
      row.combo.toLowerCase().includes(q) ||
      row.action.toLowerCase().includes(q) ||
      row.category.toLowerCase().includes(q) ||
      row.file.toLowerCase().includes(q)
  )
}

export function KeybindsPane() {
  const { icon, label } = getNavItem("keybinds")
  const searchId = useId()
  const [query, setQuery] = useState("")
  const [showLive, setShowLive] = useState(true)

  const sections = useMemo(() => filterHyprKeybindSections(query, HYPR_KEYBIND_SECTIONS), [query])

  const liveQuery = useQuery({
    queryKey: ["keybinds-live"],
    queryFn: () => api.listKeybinds(),
    enabled: showLive,
  })

  const filteredLive = useMemo(
    () => (Array.isArray(liveQuery.data) ? filterLiveBinds(liveQuery.data, query) : []),
    [liveQuery.data, query]
  )

  const validateMut = useMutation({
    mutationFn: () => api.validateKeybinds(),
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
        <p className="text-xs font-medium uppercase tracking-wide text-mauve/90">
          Hyprland binds · live list + reference
        </p>
        <p className="text-sm text-subtext0 leading-relaxed max-w-prose mt-2">
          {HYPR_KEYBIND_REFERENCE_DISCLAIMER}
        </p>
      </div>

      <div className="flex flex-col gap-1.5">
        <label htmlFor={searchId} className="text-[11px] uppercase tracking-wide text-subtext1">
          Filter binds
        </label>
        <input
          id={searchId}
          type="search"
          autoComplete="off"
          spellCheck={false}
          placeholder="Filter by key chord, action, or category…"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          className="w-full rounded-xl border border-surface0/80 bg-base/80 px-3 py-2.5 text-sm text-text placeholder:text-subtext0 focus:outline-none focus:ring-2 focus:ring-mauve/50"
        />
      </div>

      <div className="glass-card p-4 flex flex-col gap-3">
        <div className="flex items-center justify-between gap-3 flex-wrap">
          <h3 className="text-sm font-semibold text-text">Live Hyprland binds</h3>
          <div className="flex gap-2">
            <button
              type="button"
              className="text-xs px-2 py-1 rounded-lg border border-surface1/60 text-subtext1 hover:text-text"
              disabled={validateMut.isPending}
              onClick={() => validateMut.mutate()}
            >
              {validateMut.isPending ? "Validating…" : "Validate"}
            </button>
            <button
              type="button"
              className="text-xs px-2 py-1 rounded-lg border border-surface1/60 text-subtext1 hover:text-text"
              onClick={() => setShowLive((v) => !v)}
            >
              {showLive ? "Hide live" : "Load live"}
            </button>
          </div>
        </div>
        {validateMut.data &&
          (validateMut.data.duplicates.length > 0 ||
            validateMut.data.unknown_dispatches.length > 0) && (
            <div className="text-xs text-peach flex flex-col gap-1">
              {validateMut.data.duplicates.length > 0 && (
                <p>Duplicates: {validateMut.data.duplicates.join(", ")}</p>
              )}
              {validateMut.data.unknown_dispatches.length > 0 && (
                <p>Unknown dispatches: {validateMut.data.unknown_dispatches.join(", ")}</p>
              )}
            </div>
          )}
        {validateMut.isError && (
          <p className="text-xs text-red">
            {validateMut.error instanceof Error ? validateMut.error.message : "Validation failed"}
          </p>
        )}
        {showLive && liveQuery.isLoading && <p className="text-xs text-subtext0">Reading config…</p>}
        {showLive && liveQuery.isError && (
          <p className="text-xs text-peach">
            Could not load binds (Hyprland config missing or sidecar down).
          </p>
        )}
        {showLive && filteredLive.length > 0 && (
          <ul className="max-h-64 overflow-y-auto flex flex-col gap-1.5 text-xs">
            {filteredLive.slice(0, 120).map((row, i) => (
              <li key={`${row.combo}-${row.file}-${i}`} className="flex gap-2 border-b border-surface0/30 pb-1">
                <kbd className="font-mono text-mauve shrink-0">{row.combo}</kbd>
                <span className="text-subtext1 truncate">{row.action}</span>
                <span className="text-[10px] text-subtext0 shrink-0 ml-auto">{row.category}</span>
              </li>
            ))}
          </ul>
        )}
        {showLive && !liveQuery.isLoading && !liveQuery.isError && filteredLive.length === 0 && (
          <p className="text-xs text-subtext0">No live binds match this filter.</p>
        )}
      </div>

      {sections.length === 0 ? (
        <p className="text-sm text-subtext0 py-4">No reference rows match that search.</p>
      ) : (
        <div className="flex flex-col gap-4 pb-4">
          <p className="text-[11px] uppercase tracking-wide text-subtext1">Static reference</p>
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
