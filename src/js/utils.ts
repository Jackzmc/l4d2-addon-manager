export function formatSize(bytes: number, si = false, dp = 1) {
  const thresh = si ? 1000 : 1024;

  if (Math.abs(bytes) < thresh) {
    return bytes + ' B';
  }

  const units = si
    ? ['kB', 'MB', 'GB', 'TB', 'PB', 'EB', 'ZB', 'YB']
    : ['KiB', 'MiB', 'GiB', 'TiB', 'PiB', 'EiB', 'ZiB', 'YiB'];
  let u = -1;
  const r = 10**dp;

  do {
    bytes /= thresh;
    ++u;
  } while (Math.round(Math.abs(bytes) * r) / r >= thresh && u < units.length - 1);

  return (dp > 0 ? bytes.toFixed(dp) : Math.round(bytes)) + ' ' + units[u];
}

export function getRelDate(timestamp: number) : string {
  const date = timestamp ? new Date(timestamp * 1000) : new Date();
  const secondsDiff = ((Date.now() - date.getTime()) / 1000);
  const dayDiff = Math.floor(secondsDiff / 86400);
  const monthsDiff = Math.round(dayDiff / 30)
  const yearsDiff = Math.round(monthsDiff / 12)
  if(yearsDiff > 0) return `${yearsDiff} year${yearsDiff == 1 ? "" : "s"} ago`
  if(monthsDiff > 0) return `${monthsDiff} month${monthsDiff == 1 ? "" : "s"} ago`
  if (isNaN(dayDiff) || dayDiff < 0 || dayDiff >= 31) return "<invalid date>"

  return dayDiff == 0 && (
    secondsDiff < 0 && "just now" || secondsDiff < 60 && `${Math.ceil(secondsDiff)} seconds ago` || secondsDiff < 120 && "1 minute ago" || secondsDiff < 3600 && Math.floor(secondsDiff / 60) + " minutes ago" || secondsDiff < 7200 && "1 hour ago" || secondsDiff < 86400 && Math.floor(secondsDiff / 3600) + " hours ago") || dayDiff == 1 && "yesterday" || dayDiff < 7 && dayDiff + " days ago" || dayDiff < 31 && Math.ceil(dayDiff / 7) + " weeks ago"
    || "Unknown"
}
