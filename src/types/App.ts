export interface SetRoute {
    name?: string
}

export interface InitAppData {
    initial_route: SetRoute
}

export type ScanState = "started" | "failed" | "complete"
export type ScanResultType = "updated" | "renamed" | "added" | "no_action"
export interface ScanResult {
    result: ScanResultType,
    filename: string
}

export const ScanResultMessage: Record<ScanResultType, { title: string } | undefined> = {
    // Don't show these:
    updated: undefined,
    no_action: undefined,

    ["added"]: {
        title: "New Addon Found"
    },
    ["renamed"]: {
        title: "Found Renamed Addon"
    }
}