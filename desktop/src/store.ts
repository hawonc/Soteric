import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Profile, DetectedProcess, ActivityEntry } from "./types";

export function useAppState() {
  const [profiles, setProfiles] = useState<Profile[]>([]);
  const [processes, setProcesses] = useState<DetectedProcess[]>([]);
  const [activity, setActivity] = useState<ActivityEntry[]>([]);
  const [lastScan, setLastScan] = useState<string>("—");
  const [loading, setLoading] = useState(true);

  const activeProfile = profiles.find((p) => p.active) ?? null;

  function addActivity(event: string) {
    const now = new Date();
    const time = `${now.getHours()}:${String(now.getMinutes()).padStart(2, "0")}`;
    setActivity((prev) => [{ time, event }, ...prev].slice(0, 50));
  }

  const loadProfiles = useCallback(async () => {
    try {
      const result = await invoke<Profile[]>("list_profiles");
      setProfiles(result);
    } catch (e) {
      console.error("Failed to load profiles:", e);
    }
  }, []);

  useEffect(() => {
    loadProfiles().finally(() => setLoading(false));
  }, [loadProfiles]);

  async function activateProfile(name: string) {
    await invoke("activate_profile", { name });
    await loadProfiles();
    addActivity(`Profile activated: ${name}`);
  }

  async function deactivateProfile(name: string) {
    await invoke("deactivate_profile", { name });
    await loadProfiles();
    addActivity(`Profile deactivated: ${name}`);
  }

  async function deleteProfile(name: string) {
    await invoke("delete_profile", { name });
    await loadProfiles();
    addActivity(`Profile deleted: ${name}`);
  }

  async function runScan() {
    const result = await invoke<DetectedProcess[]>("scan_processes");
    setProcesses(result);
    const now = new Date();
    const time = `${now.getHours()}:${String(now.getMinutes()).padStart(2, "0")}`;
    setLastScan(time);
    if (result.length > 0) {
      addActivity(
        `Scan completed — ${result.length} AI tool(s) detected: ${result.map((p) => p.name).join(", ")}`
      );
    } else {
      addActivity("Scan completed — no AI tools detected");
    }
  }

  return {
    profiles,
    processes,
    activity,
    activeProfile,
    lastScan,
    loading,
    activateProfile,
    deactivateProfile,
    deleteProfile,
    runScan,
  };
}
