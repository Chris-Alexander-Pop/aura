import AudioFlyout from "@/components/bar/flyouts/AudioFlyout"
import BrightnessFlyout from "@/components/bar/flyouts/BrightnessFlyout"
import { postPanelHover } from "@/lib/panel-hover"
export default function MediaPopup() {
  return (
    <div
      className="flex h-full min-h-0 flex-col gap-1 overflow-y-auto rounded-2xl border border-surface0/60 bg-mantle p-2 text-text shadow-2xl"
      onMouseEnter={() => postPanelHover("mediaPopupHover", true)}
      onMouseLeave={() => postPanelHover("mediaPopupHover", false)}
    >
      <AudioFlyout />
      <div className="mx-2 border-t border-surface0/50" />
      <BrightnessFlyout />
    </div>
  )
}
