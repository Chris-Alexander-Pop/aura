/** Per-output liveness for shell windows. Dead tags must not map, paint, or OSD. */

const liveTags = new Set<string>()
const deadTags = new Set<string>()

export function markShellTagLive(tag: string): void {
    deadTags.delete(tag)
    liveTags.add(tag)
}

export function markShellTagDead(tag: string): void {
    liveTags.delete(tag)
    deadTags.add(tag)
}

export function isShellTagLive(tag: string): boolean {
    if (deadTags.has(tag)) return false
    return liveTags.has(tag)
}

export function isShellTagDead(tag: string): boolean {
    return deadTags.has(tag)
}
