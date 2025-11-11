import { notify } from "@kyvg/vue3-notification"
import { AddonFlags } from "../types/Addon.ts"
import { ItemResult } from "../types/App.ts"

const FLAG_TAG_NAMES: Record<number, string | undefined> = {
    [AddonFlags.Campaign]: 'Map',
    [AddonFlags.Survivor]: 'Survivor',
    [AddonFlags.Script]: 'Script',
    [AddonFlags.Skin]: 'Skin',
    [AddonFlags.Weapon]: 'Weapon'
}

export function getAddonContents(flags: number): string[] {
    const tags: string[] = []
    for(const [flag, name] of Object.entries(FLAG_TAG_NAMES)) {
        if(flags & Number(flag)) {
            tags.push(name!)
        }
    }
    return tags
}

/** returns # of errors and prints results */
export function handleItemResults(results: ItemResult[]) {
    console.debug(results)
    const errorCount = results.filter((entry) => entry.result === "error").length
    if(errorCount > 0) {
        for(const entry of results) {
            if(entry.result === "error")
                console.error(entry.filename, entry.error)
        }
        return errorCount
    } else {
        return 0
    }
}