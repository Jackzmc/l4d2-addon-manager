export function formatBytes(bytes) {
    if(bytes > 1000000000) {
        return `${Math.round(bytes / 1000000000.0)} GB`
    }else if (bytes > 1000000) {
        return `${Math.round(bytes / 1000000.0)} MB`
    } else if (bytes > 1000) {
        return `${Math.round(bytes / 1000.0)} KB`
    } else {
        return `${Math.round(bytes)} B`
    }
}
export function formatDate(date) {
    const d = new Date(date * 1000)
    return `${d.toLocaleDateString()}`
}