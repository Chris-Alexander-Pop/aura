import HubModuleTiles from "@/components/module-hub/HubModuleTiles"

/** Corner dropdown tile grid — same cards as module hub, no section chrome. */
export default function DropdownModuleTiles({ moduleIds }: { moduleIds: string[] }) {
  return <HubModuleTiles moduleIds={moduleIds} showSectionHeader={false} />
}
