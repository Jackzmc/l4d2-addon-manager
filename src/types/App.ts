export interface SetRoute {
    name?: string
}

export interface InitAppData {
    initial_route: SetRoute
}

export type ScanStateEvent = ScanStateEvent_Started | ScanStateEvent_Aborted | ScanStateEvent_Complete
export interface ScanStateEvent_Started {
    state: "started"
}
export interface ScanStateEvent_Aborted {
    state: "aborted",
    reason?: string
}
export interface ScanStateEvent_Complete {
    state: "complete",
    total: number,
    added: number,
    failed: number
}

export type ScanResultType = "updated" | "renamed" | "added" | "no_action"
export interface ScanResultEvent {
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