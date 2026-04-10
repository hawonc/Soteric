import { useState } from "react";
import { LayoutDashboard, Users, Monitor, ClipboardList, Shield, Settings as SettingsIcon } from "lucide-react";
import { Separator } from "@/components/ui/separator";
import { useAppState } from "./store";
import Dashboard from "./pages/Dashboard";
import Profiles from "./pages/Profiles";
import MonitorPage from "./pages/Monitor";
import ActivityLog from "./pages/ActivityLog";
import Settings from "./pages/Settings";

type Page = "dashboard" | "profiles" | "monitor" | "activity" | "settings";

const NAV = [
  { id: "dashboard" as Page, label: "Dashboard", icon: LayoutDashboard },
  { id: "profiles" as Page, label: "Profiles", icon: Users },
  { id: "monitor" as Page, label: "Live Monitor", icon: Monitor },
  { id: "activity" as Page, label: "Activity Log", icon: ClipboardList },
  { id: "settings" as Page, label: "Settings", icon: SettingsIcon },
];

export default function App() {
  const [page, setPage] = useState<Page>("dashboard");
  const state = useAppState();

  return (
    <div className="flex h-screen bg-background text-foreground overflow-hidden">
      {/* Sidebar */}
      <aside className="w-52 shrink-0 flex flex-col border-r bg-sidebar">
        {/* Logo */}
        <div className="flex items-center gap-2.5 px-4 py-5">
          <div className="w-7 h-7 rounded-md bg-primary flex items-center justify-center">
            <Shield className="w-4 h-4 text-primary-foreground" />
          </div>
          <span className="font-semibold tracking-tight">Soteric</span>
        </div>

        <Separator />

        {/* Nav */}
        <nav className="flex-1 px-2 py-3 space-y-0.5">
          {NAV.map(({ id, label, icon: Icon }) => (
            <button
              key={id}
              onClick={() => setPage(id)}
              className={`w-full flex items-center gap-2.5 px-3 py-2 rounded-md text-sm transition-colors ${
                page === id
                  ? "bg-sidebar-primary text-sidebar-primary-foreground font-medium"
                  : "text-sidebar-foreground hover:bg-sidebar-accent hover:text-sidebar-accent-foreground"
              }`}
            >
              <Icon className="w-4 h-4 shrink-0" />
              {label}
            </button>
          ))}
        </nav>

        <Separator />

        {/* Status footer */}
        <div className="px-4 py-3">
          <div className="flex items-center gap-2 text-xs text-muted-foreground">
            <span
              className={`w-2 h-2 rounded-full shrink-0 ${
                state.processes.length > 0 ? "bg-destructive" : "bg-emerald-500"
              }`}
            />
            {state.processes.length > 0 ? "AI Detected" : "Protected"}
          </div>
          {state.activeProfile && (
            <p className="text-xs text-muted-foreground mt-1 truncate">
              {state.activeProfile.name}
            </p>
          )}
          {state.monitorRunning && (
            <p className="text-xs text-emerald-600 mt-1">Monitor active</p>
          )}
        </div>
      </aside>

      {/* Main content */}
      <main className="flex-1 overflow-y-auto">
        {page === "dashboard" && (
          <Dashboard
            activeProfile={state.activeProfile}
            processes={state.processes}
            activity={state.activity}
            lastScan={state.lastScan}
            secret={state.secret}
            onSecretChange={state.setSecret}
            onScan={state.runScan}
            onEncrypt={state.encryptNow}
            onDecrypt={state.decryptNow}
            onNavigate={setPage}
          />
        )}
        {page === "profiles" && (
          <Profiles
            profiles={state.profiles}
            onActivate={state.activateProfile}
            onDeactivate={state.deactivateProfile}
            onDelete={state.deleteProfile}
            onNavigate={setPage}
            onCreate={state.createProfile}
            onAppend={state.appendProfile}
          />
        )}
        {page === "monitor" && (
          <MonitorPage
            processes={state.processes}
            activeProfile={state.activeProfile}
            lastScan={state.lastScan}
            monitorRunning={state.monitorRunning}
            onScan={state.runScan}
            onStartMonitor={state.startMonitor}
            onStopMonitor={state.stopMonitor}
          />
        )}
        {page === "activity" && <ActivityLog activity={state.activity} />}
        {page === "settings" && (
          <Settings
            secret={state.secret}
            onSecretChange={state.setSecret}
            biometricEnabled={state.biometricEnabled}
            profiles={state.profiles}
            mappings={state.mappings}
            onChangeSecret={state.changeSecret}
            onSetupBiometric={state.setupBiometric}
            onRemoveBiometric={state.removeBiometric}
            onAddMapping={state.addMapping}
            onRemoveMapping={state.removeMapping}
          />
        )}
      </main>
    </div>
  );
}
