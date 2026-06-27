import type { ReactNode } from "react"
import { motion } from "framer-motion"
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import api, { type AuraSettingsView, type PowerProfile } from "@/lib/api"
import { cn } from "@/lib/utils"
import { DROPDOWN_TILE_IDS } from "@/lib/dropdown-tiles"
import {
  SortableIdList,
  toggleChipInOrder,
} from "@/components/settings/SortableIdList"
import { applyThemeToDocument } from "@/lib/applyTheme"
import { ALL_NAV_ITEMS, getNavItem } from "../navigation"
const THEME_OPTIONS = ["dark", "light", "catppuccin-mocha", "catppuccin-frappe"] as const

function Row({ label, value }: { label: string; value: ReactNode }) {
  return (
    <div className="flex items-start justify-between gap-4 text-sm">
      <span className="text-subtext0 shrink-0">{label}</span>
      <span className="text-text text-right font-medium break-all">{value}</span>
    </div>
  )
}

function SettingsCard({
  title,
  icon,
  children,
  footer,
}: {
  title: string
  icon: string
  children: ReactNode
  footer?: React.ReactNode
}) {
  return (
    <div className="glass-card p-4 flex flex-col gap-3">
      <div className="flex items-center gap-2 border-b border-surface0/50 pb-2">
        <span className="icon text-mauve text-lg">{icon}</span>
        <h3 className="text-sm font-semibold text-text">{title}</h3>
      </div>
      <div className="flex flex-col gap-2">{children}</div>
      {footer ? <div className="pt-1 border-t border-surface0/40">{footer}</div> : null}
    </div>
  )
}

function GtkPlaceholderCard({ title, icon, body }: { title: string; icon: string; body: string }) {
  return (
    <div className="glass-card p-4 flex flex-col gap-2 opacity-85 border-dashed border-surface1/80">
      <div className="flex items-center gap-2">
        <span className="icon text-subtext1 text-lg">{icon}</span>
        <h3 className="text-sm font-semibold text-subtext1">{title}</h3>
      </div>
      <p className="text-xs text-subtext0 leading-relaxed">{body}</p>
      <p className="text-[10px] uppercase tracking-wide text-subtext1/70">GTK / shell — not in this webview</p>
    </div>
  )
}

const POWER_LABELS: Record<PowerProfile, string> = {
  performance: "Performance",
  balanced: "Balanced",
  saver: "Power saver",
}

export function SettingsPane() {
  const { icon, label } = getNavItem("settings")
  const qc = useQueryClient()

  const powerProfile = useQuery({
    queryKey: ["settings", "power-profile"],
    queryFn: api.getPowerProfile,
    refetchInterval: 15_000,
  })

  const battery = useQuery({
    queryKey: ["settings", "battery"],
    queryFn: api.getBatteryState,
    refetchInterval: 30_000,
  })

  const network = useQuery({
    queryKey: ["settings", "network-status"],
    queryFn: api.getNetworkStatus,
    refetchInterval: 5000,
  })

  const vpn = useQuery({
    queryKey: ["settings", "vpn-status"],
    queryFn: api.getVpnStatus,
    refetchInterval: 10_000,
  })

  const bluetoothAdapters = useQuery({
    queryKey: ["settings", "bt-adapters"],
    queryFn: api.getBluetoothAdapters,
    refetchInterval: 15_000,
  })

  const bluetoothDevices = useQuery({
    queryKey: ["settings", "bt-devices"],
    queryFn: api.getBluetoothDevices,
    refetchInterval: 15_000,
  })

  const audio = useQuery({
    queryKey: ["settings", "audio-devices"],
    queryFn: api.getAudioDevices,
    refetchInterval: 10_000,
  })

  const security = useQuery({
    queryKey: ["settings", "security-status"],
    queryFn: api.getSecurityStatus,
    refetchInterval: 60_000,
  })

  const system = useQuery({
    queryKey: ["settings", "system-stats"],
    queryFn: api.getSystemStats,
    refetchInterval: 3000,
  })

  const auraSettings = useQuery({
    queryKey: ["aura-settings"],
    queryFn: api.getAuraSettings,
  })

  const saveAura = useMutation({
    mutationFn: (partial: Partial<AuraSettingsView>) => api.setAuraSettings(partial),
    onSuccess: () => void qc.invalidateQueries({ queryKey: ["aura-settings"] }),
  })

  const resetAura = useMutation({
    mutationFn: () => api.resetAuraSettings(),
    onSuccess: () => void qc.invalidateQueries({ queryKey: ["aura-settings"] }),
  })

  const powerMut = useMutation({
    mutationFn: (profile: PowerProfile) => api.setPowerProfile(profile),
    onSuccess: () => void qc.invalidateQueries({ queryKey: ["settings", "power-profile"] }),
  })

  async function applyPowerProfile(profile: PowerProfile) {
    await powerMut.mutateAsync(profile)
  }

  const defaultSink = audio.data?.sinks.find((s) => s.is_default)
  const defaultSource = audio.data?.sources.find((s) => s.is_default)
  const btPowered =
    bluetoothAdapters.data?.some((a) => a.powered) ?? false
  const btConnected = bluetoothDevices.data?.filter((d) => d.connected).length ?? 0

  const securityEntries =
    security.data != null
      ? Object.entries(security.data).filter(([k]) => k !== undefined).slice(0, 6)
      : []

  return (
    <motion.div
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -10 }}
      transition={{ duration: 0.18 }}
      className="flex flex-col gap-5 p-6 min-h-0 h-full overflow-y-auto overscroll-contain"
    >
      <div>
        <div className="flex items-center gap-3 mb-1">
          <span className="icon text-mauve text-2xl">{icon}</span>
          <h2 className="text-xl font-semibold text-text">{label}</h2>
        </div>
        <p className="text-xs text-subtext1 max-w-prose leading-relaxed">
          Sidecar snapshots in one place. Deeper edits for network, Bluetooth, and audio stay on their dedicated panes; the
          cards below are read-mostly summaries plus power profile.
        </p>
      </div>

      <SettingsCard
        title="Aura preferences"
        icon="tune"
        footer={<p className="text-[10px] text-subtext1">Settings.Get / Settings.Set — bar layout, theme, control center modules</p>}
      >
        {auraSettings.isLoading ? (
          <div className="skeleton h-24 rounded-lg" />
        ) : auraSettings.isError ? (
          <p className="text-xs text-red">
            {auraSettings.error instanceof Error
              ? auraSettings.error.message
              : "Could not load Aura settings from sidecar"}
          </p>
        ) : auraSettings.data ? (
          <div className="flex flex-col gap-3">
            <div>
              <p className="text-xs text-subtext0 mb-2">Theme</p>
              <div className="flex flex-wrap gap-2">
                {THEME_OPTIONS.map((t) => (
                  <button
                    key={t}
                    type="button"
                    disabled={saveAura.isPending}
                    onClick={() => {
                      applyThemeToDocument(t)
                      void saveAura.mutateAsync({ theme: t })
                    }}
                    className={cn(
                      "toggle-chip text-xs",
                      auraSettings.data.settings.theme === t && "active"
                    )}
                  >
                    {t}
                  </button>
                ))}
              </div>
            </div>
            <div>
              <p className="text-xs text-subtext0 mb-2">Bar sections (drag to reorder)</p>
              <SortableIdList
                ids={auraSettings.data.settings.bar_section_order}
                disabled={saveAura.isPending}
                onReorder={(order) => void saveAura.mutateAsync({ bar_section_order: order })}
              />
            </div>
            <div>
              <p className="text-xs text-subtext0 mb-2">Control center panes</p>
              <div className="flex flex-wrap gap-2 max-h-32 overflow-y-auto">
                {ALL_NAV_ITEMS.map((item) => {
                  const enabled = auraSettings.data.settings.cc_enabled_panes.includes(item.id)
                  return (
                    <button
                      key={item.id}
                      type="button"
                      disabled={saveAura.isPending}
                      onClick={() => {
                        const set = new Set(auraSettings.data!.settings.cc_enabled_panes)
                        if (enabled) set.delete(item.id)
                        else set.add(item.id)
                        void saveAura.mutateAsync({
                          cc_enabled_panes: Array.from(set),
                        })
                      }}
                      className={cn("toggle-chip text-[10px]", enabled && "active")}
                    >
                      {item.label}
                    </button>
                  )
                })}
              </div>
            </div>
            <div>
              <p className="text-xs text-subtext0 mb-2">Module hub trigger</p>
              <div className="flex flex-wrap gap-2">
                {(
                  [
                    { id: "left_edge", label: "Left edge" },
                    { id: "top_third", label: "Top third" },
                    { id: "none", label: "Off (keybind only)" },
                  ] as const
                ).map((opt) => (
                  <button
                    key={opt.id}
                    type="button"
                    disabled={saveAura.isPending}
                    onClick={() => void saveAura.mutateAsync({ module_hub_trigger: opt.id })}
                    className={cn(
                      "toggle-chip text-xs",
                      auraSettings.data.settings.module_hub_trigger === opt.id && "active"
                    )}
                  >
                    {opt.label}
                  </button>
                ))}
              </div>
            </div>
            <div>
              <p className="text-xs text-subtext0 mb-2">Fullscreen behavior</p>
              <button
                type="button"
                disabled={saveAura.isPending}
                onClick={() =>
                  void saveAura.mutateAsync({
                    hide_shell_on_fullscreen: !auraSettings.data.settings.hide_shell_on_fullscreen,
                  })
                }
                className={cn(
                  "toggle-chip text-xs",
                  auraSettings.data.settings.hide_shell_on_fullscreen && "active"
                )}
              >
                Hide bar and overlays on fullscreen
              </button>
            </div>
            <div>
              <p className="text-xs text-subtext0 mb-2">Dropdown modules</p>
              <div className="flex flex-wrap gap-2 mb-2">
                {DROPDOWN_TILE_IDS.map((mod) => {
                  const enabled = auraSettings.data.settings.dropdown_modules.includes(mod)
                  return (
                    <button
                      key={mod}
                      type="button"
                      disabled={saveAura.isPending}
                      onClick={() => {
                        const next = toggleChipInOrder(
                          auraSettings.data!.settings.dropdown_modules,
                          mod,
                          DROPDOWN_TILE_IDS
                        )
                        void saveAura.mutateAsync({ dropdown_modules: next })
                      }}
                      className={cn("toggle-chip text-[10px]", enabled && "active")}
                    >
                      {mod}
                    </button>
                  )
                })}
              </div>
              {auraSettings.data.settings.dropdown_modules.length > 0 ? (
                <>
                  <p className="text-[10px] text-subtext1 mb-1">Tile order (enabled only)</p>
                  <SortableIdList
                    ids={auraSettings.data.settings.dropdown_modules}
                    disabled={saveAura.isPending}
                    onReorder={(order) => void saveAura.mutateAsync({ dropdown_modules: order })}
                  />
                </>
              ) : null}
            </div>
            <button
              type="button"
              className="toggle-chip text-xs self-start"
              disabled={resetAura.isPending}
              onClick={() => void resetAura.mutateAsync()}
            >
              Reset Aura settings
            </button>
            {(saveAura.isError || resetAura.isError) && (
              <p className="text-xs text-red">
                {saveAura.error instanceof Error
                  ? saveAura.error.message
                  : resetAura.error instanceof Error
                    ? resetAura.error.message
                    : "Save failed"}
              </p>
            )}
          </div>
        ) : (
          <p className="text-xs text-subtext0">Could not load Aura settings from sidecar</p>
        )}
      </SettingsCard>

      <div className="grid gap-4 sm:grid-cols-2">
        <SettingsCard title="Power profile" icon="bolt" footer={<p className="text-[10px] text-subtext1">Writes via Power.SetProfile</p>}>
          {powerProfile.isLoading ? (
            <div className="skeleton h-10 rounded-lg" />
          ) : powerProfile.isError ? (
            <p className="text-xs text-red">
              {powerProfile.error instanceof Error ? powerProfile.error.message : "Could not load profile"}
            </p>
          ) : (
            <div className="flex flex-wrap gap-2">
              {(Object.keys(POWER_LABELS) as PowerProfile[]).map((p) => (
                <button
                  key={p}
                  type="button"
                  disabled={powerMut.isPending}
                  onClick={() => void applyPowerProfile(p)}
                  className={cn(
                    "toggle-chip text-xs capitalize",
                    powerProfile.data?.profile === p && "active"
                  )}
                >
                  {POWER_LABELS[p]}
                </button>
              ))}
            </div>
          )}
          {powerMut.isError && (
            <p className="text-xs text-red">
              {powerMut.error instanceof Error ? powerMut.error.message : "Could not set profile"}
            </p>
          )}
          {battery.isLoading ? (
            <div className="skeleton h-8 rounded-lg" />
          ) : battery.data ? (
            <div className="mt-1 space-y-1.5 pt-2 border-t border-surface0/40">
              <Row label="Battery" value={`${battery.data.percent}%${battery.data.charging ? " · charging" : ""}`} />
              <Row label="Time remaining" value={battery.data.time_remaining || "—"} />
            </div>
          ) : (
            <p className="text-xs text-subtext0">No battery data from sidecar</p>
          )}
        </SettingsCard>

        <SettingsCard title="Network snapshot" icon="wifi" footer={<p className="text-[10px] text-subtext1">From Network.GetStatus</p>}>
          {network.isLoading ? (
            <div className="skeleton h-20 rounded-lg" />
          ) : (
            <>
              <Row label="Connection" value={network.data?.active_connection ?? "—"} />
              <Row label="Wi-Fi" value={network.data?.wifi_enabled ? "On" : "Off"} />
              <Row label="Local IP" value={network.data?.local_ip ?? "—"} />
              <Row label="Public IP" value={network.data?.public_ip ?? "—"} />
            </>
          )}
        </SettingsCard>

        <SettingsCard title="VPN" icon="vpn_key" footer={<p className="text-[10px] text-subtext1">From Vpn.GetStatus</p>}>
          {vpn.isLoading ? (
            <div className="skeleton h-16 rounded-lg" />
          ) : (
            <>
              <Row label="State" value={vpn.data?.state ?? "—"} />
              <Row label="Detail" value={vpn.data?.message || "—"} />
              {vpn.data?.profile_id ? <Row label="Profile" value={vpn.data.profile_id} /> : null}
            </>
          )}
        </SettingsCard>

        <SettingsCard title="Bluetooth snapshot" icon="bluetooth" footer={<p className="text-[10px] text-subtext1">Bluetooth.GetAdapters / GetDevices</p>}>
          {bluetoothAdapters.isLoading || bluetoothDevices.isLoading ? (
            <div className="skeleton h-16 rounded-lg" />
          ) : (
            <>
              <Row label="Adapter" value={bluetoothAdapters.data?.length ? `${bluetoothAdapters.data.length} present` : "—"} />
              <Row label="Radio" value={btPowered ? "Powered" : "Off / none"} />
              <Row label="Connected devices" value={String(btConnected)} />
            </>
          )}
        </SettingsCard>

        <SettingsCard title="Audio defaults" icon="tune" footer={<p className="text-[10px] text-subtext1">Audio.GetDevices</p>}>
          {audio.isLoading ? (
            <div className="skeleton h-20 rounded-lg" />
          ) : (
            <>
              <Row label="Default output" value={defaultSink?.name ?? "—"} />
              <Row label="Output volume" value={defaultSink != null ? `${defaultSink.volume}%` : "—"} />
              <Row label="Default input" value={defaultSource?.name ?? "—"} />
            </>
          )}
        </SettingsCard>

        <SettingsCard title="Security status" icon="shield" footer={<p className="text-[10px] text-subtext1">Security.GetStatus</p>}>
          {security.isLoading ? (
            <div className="skeleton h-20 rounded-lg" />
          ) : securityEntries.length > 0 ? (
            securityEntries.map(([k, v]) => (
              <Row key={k} label={k} value={typeof v === "object" ? JSON.stringify(v) : String(v)} />
            ))
          ) : (
            <p className="text-xs text-subtext0">Empty or unavailable</p>
          )}
        </SettingsCard>

        <SettingsCard title="System snapshot" icon="monitor_heart" footer={<p className="text-[10px] text-subtext1">System.GetStats</p>}>
          {system.isLoading ? (
            <div className="skeleton h-24 rounded-lg" />
          ) : system.data ? (
            <>
              <Row label="CPU" value={`${system.data.cpu}%`} />
              <Row label="RAM" value={`${system.data.ram}%`} />
              <Row label="Temp" value={`${system.data.temp}°C`} />
              {system.data.gpu != null ? <Row label="GPU" value={`${system.data.gpu}%`} /> : null}
              {system.data.storage != null ? <Row label="Storage" value={`${system.data.storage}%`} /> : null}
            </>
          ) : (
            <p className="text-xs text-subtext0">No stats</p>
          )}
        </SettingsCard>
      </div>

      <div>
        <p className="text-[11px] uppercase tracking-wider text-subtext1 mb-2 px-0.5">GTK-only / compositor prefs</p>
        <p className="text-xs text-subtext0 mb-3 max-w-prose">
          These are intentionally not mirrored over the sidecar web API today. Use your desktop settings apps, dotfiles, or the GTK shell (`style/`, Hyprland) instead.
        </p>
        <div className="grid gap-3 sm:grid-cols-2">
          <GtkPlaceholderCard
            title="Appearance & GTK theme"
            icon="palette"
            body="Accent colors, dark/light preference, and GTK widget themes are handled by GNOME Settings, nwg-look, or your distro’s appearance tool — not this React route."
          />
          <GtkPlaceholderCard
            title="Display & scaling"
            icon="desktop_windows"
            body="Resolution, fractional scaling, and variable refresh live in your compositor or wlr-randr-style tools; Hyprland monitor rules are outside the webview."
          />
          <GtkPlaceholderCard
            title="Keyboard layouts & input"
            icon="keyboard"
            body="IBus, fcitx, and XKB layouts are configured through GTK/system settings — there is no RPC hook here yet."
          />
          <GtkPlaceholderCard
            title="Locale, time & timezone"
            icon="schedule"
            body="Region formats and clocks are system-level (`timedatectl`, Settings → Date & Time). This panel won’t duplicate them until there is an audited sidecar surface."
          />
        </div>
      </div>
    </motion.div>
  )
}
