export interface Addon {
    filename: string,
    updated_at: number // TODO: not sure type, verify,
    created_at: number // TODO: not sure type, verify,
    file_size: number,
    flags: number // AddonFlags enum
}

export interface AddonEntry {
    addon: Addon,
    workshop_info: WorkshopItem | null,
    tags: string[]
}

export interface WorkshopItem {
    publishedfileid: string,
    title: string
}

export const enum AddonFlags {
    None = 0,
    Workshop = 1,
    Campaign = 2
}