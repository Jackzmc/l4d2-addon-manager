export interface Addon {
    filename: string,
    /** ISO Date, parse as Date */
    updated_at: string
    /** ISO Date, parse as Date */
    created_at: string
    file_size: number
    flags: number // AddonFlags enum
    title: string,
    author: string | null,
    version: string | null,
    tagline: string | null,
    chapter_ids: string | null,
    workshop_id: string | null
}

export interface AddonEntry {
    addon: Addon,
    workshop_info: WorkshopItem | null,
    tags: string[],
    enabled: boolean
}

export interface WorkshopItem {
    publishedfileid: string,
    title: string
}

export const enum AddonFlags {
    None = 0,
    Workshop = 1,
    Campaign = 2,
    Survivor = 4,
    Script = 8,
    Skin = 16,
    Weapon = 32
}