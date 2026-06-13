import {
  DndContext,
  closestCenter,
  KeyboardSensor,
  PointerSensor,
  useSensor,
  useSensors,
  type DragEndEvent,
} from "@dnd-kit/core"
import {
  SortableContext,
  sortableKeyboardCoordinates,
  useSortable,
  verticalListSortingStrategy,
  arrayMove,
} from "@dnd-kit/sortable"
import { CSS } from "@dnd-kit/utilities"
import { cn } from "@/lib/utils"

function SortableRow({
  id,
  label,
  disabled,
}: {
  id: string
  label: string
  disabled?: boolean
}) {
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({
    id,
    disabled,
  })
  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
  }

  return (
    <div
      ref={setNodeRef}
      style={style}
      className={cn(
        "flex items-center gap-2 rounded-lg border border-surface0/60 bg-surface0/25 px-2 py-1.5 text-xs",
        isDragging && "opacity-70 shadow-md ring-1 ring-mauve/40",
        disabled && "opacity-50"
      )}
    >
      <button
        type="button"
        className="icon cursor-grab text-subtext0 active:cursor-grabbing touch-none"
        aria-label={`Drag to reorder ${label}`}
        disabled={disabled}
        {...attributes}
        {...listeners}
      >
        drag_indicator
      </button>
      <span className="flex-1 font-medium text-text">{label}</span>
    </div>
  )
}

export function SortableIdList({
  ids,
  onReorder,
  labelForId,
  disabled,
}: {
  ids: string[]
  onReorder: (next: string[]) => void
  labelForId?: (id: string) => string
  disabled?: boolean
}) {
  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 6 } }),
    useSensor(KeyboardSensor, { coordinateGetter: sortableKeyboardCoordinates })
  )

  const onDragEnd = (event: DragEndEvent) => {
    const { active, over } = event
    if (!over || active.id === over.id) return
    const oldIndex = ids.indexOf(String(active.id))
    const newIndex = ids.indexOf(String(over.id))
    if (oldIndex < 0 || newIndex < 0) return
    onReorder(arrayMove(ids, oldIndex, newIndex))
  }

  const label = labelForId ?? ((id: string) => id)

  return (
    <DndContext sensors={sensors} collisionDetection={closestCenter} onDragEnd={onDragEnd}>
      <SortableContext items={ids} strategy={verticalListSortingStrategy}>
        <div className="flex flex-col gap-1">
          {ids.map((id) => (
            <SortableRow key={id} id={id} label={label(id)} disabled={disabled} />
          ))}
        </div>
      </SortableContext>
    </DndContext>
  )
}

/** Toggle chips for all options; enabled ids stay in `order`, toggling on appends to end. */
export function toggleIdInOrder(
  order: string[],
  id: string,
  enabled: boolean,
  allIds: readonly string[]
): string[] {
  if (enabled) {
    return order.filter((x) => x !== id)
  }
  const next = [...order.filter((x) => x !== id), id]
  const missing = allIds.filter((x) => !next.includes(x))
  return [...next, ...missing.filter((x) => order.includes(x) && !next.includes(x))]
}

export function toggleChipInOrder(order: string[], id: string, allIds: readonly string[]): string[] {
  const enabled = order.includes(id)
  if (enabled) {
    return order.filter((x) => x !== id)
  }
  return [...order, id]
}
