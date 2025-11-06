import { AddonFlags } from "../types/Addon.ts"

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