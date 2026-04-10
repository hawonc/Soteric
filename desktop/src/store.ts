import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Profile, DetectedProcess, ActivityEntry } from "./types";

export function useAppState() {
  const [profiles, setProfiles] = useState<Profile[]>([]);
  const [processes, setProcesses] = useState<DetectedProcess[]>([]);
  const [activity, setActivity] = useState<ActivityEntry[]>([]);
  const [lastScan, setLastScan] = useState<string>("—");
  const [loading, setLoading] = useState(true);
  const [secret, setSecret] = useState<string>("");

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
    try {
      await invoke("activate_profile", { name, secret: secret || null });
      await loadProfiles();
      addActivity(`Profile activated + files encrypted: ${name}`);
    } catch (e) {
      console.error("Failed to activate profile:", e);
      addActivity(`Error: failed to activate ${name} — ${e}`);
      throw e;
    }
  }

  async function deactivateProfile(name: string) {
    try {
      await invoke("deactivate_profile", { name, secret: secret || null });
      await loadProfiles();
      addActivity(`Profile deactivated + files decrypted: ${name}`);
    } catch (e) {
      console.error("Failed to deactivate profile:", e);
      addActivity(`Error: failed to deactivate ${name} — ${e}`);
      throw e;
    }
  }

  async function deleteProfile(name: string) {
    try {
      await invoke("delete_profile", { name });
      await loadProfiles();
      addActivity(`Profile deleted: ${name}`);
    } catch (e) {
      console.error("Failed to delete profile:", e);
      addActivity(`Error: failed to delete ${name} — ${e}`);
      throw e;
    }
  }

  async function createProfile(name: string, files: string[], globs: string[]) {
    try {
      await invoke("create_profile", { name, files, globs });
      await loadProfiles();
      addActivity(`Profile created: ${name}`);
    } catch (e) {
      console.error("Failed to create profile:", e);
      addActivity(`Error: failed to create profile ${name} — ${e}`);
      throw e;
    }
  }

  async function appendProfile(name: string, files: string[], globs: string[]) {
    try {
      await invoke("append_profile", { name, files, globs });
      await loadProfiles();
      addActivity(`Files added to profile: ${name}`);
    } catch (e) {
      console.error("Failed to append profile:", e);
      addActivity(`Error: failed to append to ${name} — ${e}`);
      throw e;
    }
  }

  async function encryptNow() {
    try {
      await invoke("encrypt_now", { secret: secret || null });
      addActivity("Files encrypted manually");
    } catch (e) {
      console.error("Failed to encrypt:", e);
      addActivity(`Error: encryption failed — ${e}`);
      throw e;
    }
  }

  async function decryptNow() {
    try {
      await invoke("decrypt_now", { secret: secret || null });
      addActivity("Files decrypted manually");
    } catch (e) {
      console.error("Failed to decrypt:", e);
      addActivity(`Error: decryption failed — ${e}`);
      throw e;
    }
  }

  async function runScan() {
    try {
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
    } catch (e) {
      console.error("Failed to scan:", e);
      addActivity("Error: scan failed");
    }
  }

  return {
    profiles,
    processes,
    activity,
    activeProfile,
    lastScan,
    loading,
    secret,
    setSecret,
    activateProfile,
    deactivateProfile,
    deleteProfile,
    createProfile,
    appendProfile,
    encryptNow,
    decryptNow,
    runScan,
  };
}
