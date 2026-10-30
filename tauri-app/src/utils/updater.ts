export interface UpdateInfo {
  hasUpdate: boolean;
  currentVersion: string;
  latestVersion: string;
  releaseUrl: string;
  setupDownloadUrl?: string;
  releaseNotes?: string;
}

export const CURRENT_VERSION = "0.2.2";

export async function checkForAppUpdate(): Promise<UpdateInfo> {
  try {
    const res = await fetch("https://api.github.com/repos/raghavatgit/StreamShield/releases/latest", {
      headers: {
        Accept: "application/vnd.github.v3+json",
      },
    });

    if (!res.ok) {
      return { hasUpdate: false, currentVersion: CURRENT_VERSION, latestVersion: CURRENT_VERSION, releaseUrl: "" };
    }

    const data = await res.json();
    const tag = (data.tag_name || "").replace(/^v/, "").trim();
    const releaseUrl = data.html_url || "https://github.com/raghavatgit/StreamShield/releases/latest";
    const releaseNotes = data.body || "";

    const setupAsset = (data.assets || []).find((a: any) =>
      a.name && (a.name.endsWith("-setup.exe") || a.name.endsWith(".exe"))
    );
    const setupDownloadUrl = setupAsset ? setupAsset.browser_download_url : undefined;

    const hasUpdate = isNewerVersion(tag, CURRENT_VERSION);

    return {
      hasUpdate,
      currentVersion: CURRENT_VERSION,
      latestVersion: tag || CURRENT_VERSION,
      releaseUrl,
      setupDownloadUrl,
      releaseNotes,
    };
  } catch {
    return { hasUpdate: false, currentVersion: CURRENT_VERSION, latestVersion: CURRENT_VERSION, releaseUrl: "" };
  }
}

function isNewerVersion(remote: string, current: string): boolean {
  if (!remote || !current) return false;
  const parse = (v: string) => v.split(".").map((n) => parseInt(n, 10) || 0);
  const r = parse(remote);
  const c = parse(current);
  for (let i = 0; i < Math.max(r.length, c.length); i++) {
    const rPart = r[i] || 0;
    const cPart = c[i] || 0;
    if (rPart > cPart) return true;
    if (rPart < cPart) return false;
  }
  return false;
}

