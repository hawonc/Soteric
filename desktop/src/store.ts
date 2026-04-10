import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Profile, DetectedProcess, ActivityEntry, Mapping } from "./types";

export function useAppState() {
  const [profiles, setProfiles] = useState<Profile[]>([]);
  const [processes, setProcesses] = useState<DetectedProcess[]>([]);
  const [activity, setActivity] = useState<ActivityEntry[]>([]);
  const [lastScan, setLastScan] = useState<string>("—");
  const [loading, setLoading] = useState(true);
  const [secret, setSecret] = useState<string>("");
  const [mappings, setMappings] = useState<Mapping[]>([]);
  const [biometricEnabled, setBiometricEnabled] = useState(false);
  const [monitorRunning, setMonitorRunning] = useState(false);

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

  const loadMappings = useCallback(async () => {
    try {
      const result = await invoke<Mapping[]>("list_mappings");
      setMappings(result);
    } catch (e) {
      console.error("Failed to load mappings:", e);
    }
  }, []);

  const checkBiometric = useCallback(async () => {
    try {
      const result = await invoke<boolean>("check_biometric");
      setBiometricEnabled(result);
    } catch {
      setBiometricEnabled(false);
    }
  }, []);

  const checkMonitor = useCallback(async () => {
    try {
      const result = await invoke<boolean>("is_monitor_running");
      setMonitorRunning(result);
    } catch {
      setMonitorRunning(false);
    }
  }, []);

  useEffect(() => {
    Promise.all([loadProfiles(), loadMappings(), checkBiometric(), checkMonitor()])
      .finally(() => setLoading(false));

    // Listen for background monitor events
    let prevPids: Set<number> = new Set();

    const unlistenScan = listen<{ processes: DetectedProcess[]; time: string }>(
      "monitor-scan",
      (event) => {
        const { processes: newProcs, time } = event.payload;
        const newPids = new Set(newProcs.map((p) => p.pid));

        // Log only when detection status changes
        const appeared = newProcs.filter((p) => !prevPids.has(p.pid));
        const disappeared = [...prevPids].filter((pid) => !newPids.has(pid));

        if (appeared.length > 0) {
          const names = appeared.map((p) => p.name).join(", ");
          setActivity((prev) =>
            [{ time, event: `Monitor detected: ${names}` }, ...prev].slice(0, 50)
          );
        }
        if (disappeared.length > 0 && prevPids.size > 0) {
          setActivity((prev) =>
            [{ time, event: `Monitor: ${disappeared.length} AI process(es) stopped` }, ...prev].slice(0, 50)
          );
        }

        prevPids = newPids;
        setProcesses(newProcs);
        setLastScan(time);
      }
    );

    const unlistenActivity = listen<{ event: string; time: string }>(
      "monitor-activity",
      (event) => {
        const { time, event: msg } = event.payload;
        setActivity((prev) => [{ time, event: msg }, ...prev].slice(0, 50));
        // Reload profiles since monitor may have changed active profile
        loadProfiles();
      }
    );

    return () => {
      unlistenScan.then((fn) => fn());
      unlistenActivity.then((fn) => fn());
    };
  }, [loadProfiles, loadMappings, checkBiometric, checkMonitor]);

  async function activateProfile(name: string) {
    try {
      await invoke("activate_profile", { name, secret: secret || null });
      await loadProfiles();
      addActivity(`Profile activated + files encrypted: ${name}`);
    } catch (e) {
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
      addActivity(`Error: failed to append to ${name} — ${e}`);
      throw e;
    }
  }

  async function encryptNow() {
    try {
      await invoke("encrypt_now", { secret: secret || null });
      await loadProfiles();
      addActivity("Files encrypted manually");
    } catch (e) {
      addActivity(`Error: encryption failed — ${e}`);
      throw e;
    }
  }

  async function decryptNow() {
    try {
      await invoke("decrypt_now", { secret: secret || null });
      await loadProfiles();
      addActivity("Files decrypted manually");
    } catch (e) {
      addActivity(`Error: decryption failed — ${e}`);
      throw e;
    }
  }

  async function changeSecret(currentSecret: string, newSecret: string) {
    try {
      await invoke("set_secret", { currentSecret: currentSecret || null, newSecret });
      setSecret(newSecret);
      addActivity("Secret key changed");
    } catch (e) {
      addActivity(`Error: failed to change secret — ${e}`);
      throw e;
    }
  }

  async function setupBiometric() {
    try {
      const key = secret;
      if (!key) throw new Error("Set a secret key first");
      await invoke("setup_biometric", { secret: key });
      setBiometricEnabled(true);
      addActivity("Biometric (Touch ID) enabled");
    } catch (e) {
      addActivity(`Error: biometric setup failed — ${e}`);
      throw e;
    }
  }

  async function removeBiometric() {
    try {
      await invoke("remove_biometric");
      setBiometricEnabled(false);
      addActivity("Biometric (Touch ID) removed");
    } catch (e) {
      addActivity(`Error: failed to remove biometric — ${e}`);
      throw e;
    }
  }

  async function addMapping(process: string, profile: string) {
    try {
      await invoke("set_mapping", { process, profile });
      await loadMappings();
      addActivity(`Mapping added: ${process} → ${profile}`);
    } catch (e) {
      addActivity(`Error: failed to add mapping — ${e}`);
      throw e;
    }
  }

  async function removeMapping(process: string) {
    try {
      await invoke("delete_mapping", { process });
      await loadMappings();
      addActivity(`Mapping removed: ${process}`);
    } catch (e) {
      addActivity(`Error: failed to remove mapping — ${e}`);
      throw e;
    }
  }

  async function startMonitor() {
    try {
      await invoke("start_monitor", { secret: secret || null });
      setMonitorRunning(true);
      addActivity("Background monitor started");
    } catch (e) {
      addActivity(`Error: failed to start monitor — ${e}`);
      throw e;
    }
  }

  async function stopMonitor() {
    try {
      await invoke("stop_monitor");
      setMonitorRunning(false);
      addActivity("Background monitor stopped");
    } catch (e) {
      addActivity(`Error: failed to stop monitor — ${e}`);
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
    mappings,
    biometricEnabled,
    monitorRunning,
    activateProfile,
    deactivateProfile,
    deleteProfile,
    createProfile,
    appendProfile,
    encryptNow,
    decryptNow,
    changeSecret,
    setupBiometric,
    removeBiometric,
    addMapping,
    removeMapping,
    startMonitor,
    stopMonitor,
    runScan,
  };
}
