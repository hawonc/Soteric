import { useState } from "react";
import type { Profile, DetectedProcess, ActivityEntry } from "./types";

// Mock data — replace with Tauri invoke calls later
export const MOCK_PROFILES: Profile[] = [
  {
    name: "secrets",
    root: "/project",
    files: ["/project/secret.txt", "/project/temp/codex.txt", "/project/.env"],
    active: true,
  },
  {
    name: "hidden-files",
    root: "/project",
    files: ["/project/.gitconfig", "/project/.zshrc"],
    active: false,
  },
  {
    name: "temp-files",
    root: "/project/temp",
    files: ["/project/temp/draft.txt"],
    active: false,
  },
];

export const MOCK_PROCESSES: DetectedProcess[] = [];

export const MOCK_ACTIVITY: ActivityEntry[] = [
  { time: "14:31", event: "Scan completed — no AI tools detected" },
  { time: "14:20", event: "Profile activated: secrets" },
  { time: "13:55", event: "Profile created: temp-files" },
  { time: "13:40", event: "Scan completed — no AI tools detected" },
];

export function useAppState() {
  const [profiles, setProfiles] = useState<Profile[]>(MOCK_PROFILES);
  const [processes] = useState<DetectedProcess[]>(MOCK_PROCESSES);
  const [activity] = useState<ActivityEntry[]>(MOCK_ACTIVITY);
  const [lastScan, setLastScan] = useState<string>("14:31");

  const activeProfile = profiles.find((p) => p.active) ?? null;

  function activateProfile(name: string) {
    setProfiles((prev) =>
      prev.map((p) => ({ ...p, active: p.name === name }))
    );
  }

  function deactivateProfile(name: string) {
    setProfiles((prev) =>
      prev.map((p) => (p.name === name ? { ...p, active: false } : p))
    );
  }

  function deleteProfile(name: string) {
    setProfiles((prev) => prev.filter((p) => p.name !== name));
  }

  function runScan() {
    const now = new Date();
    setLastScan(`${now.getHours()}:${String(now.getMinutes()).padStart(2, "0")}`);
  }

  return {
    profiles,
    processes,
    activity,
    activeProfile,
    lastScan,
    activateProfile,
    deactivateProfile,
    deleteProfile,
    runScan,
  };
}
