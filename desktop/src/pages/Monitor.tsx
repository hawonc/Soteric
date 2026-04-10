import { useState } from "react";
import { RefreshCw, ShieldAlert, ShieldCheck, Activity } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import type { DetectedProcess, Profile } from "@/types";

const KNOWN_TOOLS = [
  "codex", "claude", "claude-code", "opencode", "openhands",
  "cursor", "copilot", "windsurf", "antigravity",
];

interface Props {
  processes: DetectedProcess[];
  activeProfile: Profile | null;
  lastScan: string;
  monitorRunning: boolean;
  onScan: () => void;
  onStartMonitor: () => Promise<void>;
  onStopMonitor: () => Promise<void>;
}

export default function Monitor({ processes, activeProfile, lastScan, monitorRunning, onScan, onStartMonitor, onStopMonitor }: Props) {
  const isDetected = processes.length > 0;
  const [monitorBusy, setMonitorBusy] = useState(false);
  const [monitorError, setMonitorError] = useState<string | null>(null);

  return (
    <div className="p-6 space-y-4 max-w-2xl">
      <div className="mb-6">
        <h1 className="text-2xl font-semibold tracking-tight">Live Monitor</h1>
        <p className="text-sm text-muted-foreground">Scan for running AI coding tools</p>
      </div>

      {/* Status banner */}
      <Card className={isDetected ? "border-destructive/50 bg-destructive/5" : "border-emerald-600/30 bg-emerald-50/50 dark:bg-emerald-950/20"}>
        <CardContent className="flex items-center gap-3 py-4">
          {isDetected ? (
            <>
              <ShieldAlert className="w-6 h-6 text-destructive shrink-0" />
              <div>
                <p className="font-semibold text-destructive">AI Tool Detected</p>
                <p className="text-sm text-muted-foreground">{processes.length} process(es) found</p>
              </div>
            </>
          ) : (
            <>
              <ShieldCheck className="w-6 h-6 text-emerald-600 shrink-0" />
              <div>
                <p className="font-semibold text-emerald-700 dark:text-emerald-400">No AI tools running</p>
                <p className="text-sm text-muted-foreground">Your files are safe</p>
              </div>
            </>
          )}
        </CardContent>
      </Card>

      {/* Detected processes */}
      <Card>
        <CardHeader className="pb-3">
          <CardTitle className="text-sm font-medium">AI Coding Tools</CardTitle>
        </CardHeader>
        <CardContent>
          {processes.length > 0 ? (
            <div className="space-y-2">
              <div className="grid grid-cols-[1fr_80px_1fr_80px] text-xs font-medium text-muted-foreground uppercase tracking-wide pb-2 border-b">
                <span>Process</span>
                <span>PID</span>
                <span>Command</span>
                <span>Status</span>
              </div>
              {processes.map((p) => (
                <div key={p.pid} className="grid grid-cols-[1fr_80px_1fr_80px] text-sm py-1.5 items-center">
                  <span className="font-medium">{p.name}</span>
                  <span className="font-mono text-muted-foreground">{p.pid}</span>
                  <span className="font-mono text-xs text-muted-foreground truncate" title={p.command}>
                    {p.command}
                  </span>
                  <Badge variant="destructive" className="w-fit text-xs">running</Badge>
                </div>
              ))}
            </div>
          ) : (
            <p className="text-sm text-muted-foreground">No processes detected in last scan</p>
          )}
        </CardContent>
      </Card>

      {/* Protection status */}
      {isDetected && activeProfile && activeProfile.encrypted && (
        <Card className="border-amber-500/30 bg-amber-50/50 dark:bg-amber-950/20">
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-amber-700 dark:text-amber-400">
              Protection Triggered
            </CardTitle>
          </CardHeader>
          <CardContent className="text-sm space-y-1">
            <p>Profile: <span className="font-medium">{activeProfile.name}</span></p>
            <p className="text-muted-foreground">Files encrypted for this profile</p>
          </CardContent>
        </Card>
      )}

      <Separator />

      {/* Scan controls */}
      <div className="space-y-3">
        <p className="text-sm font-medium">Scan Controls</p>
        <div className="flex items-center gap-3">
          <Button onClick={onScan} className="flex items-center gap-2">
            <RefreshCw className="w-4 h-4" />
            Scan Now
          </Button>
          {monitorRunning ? (
            <Button
              variant="destructive"
              size="sm"
              disabled={monitorBusy}
              onClick={async () => {
                setMonitorBusy(true);
                setMonitorError(null);
                try { await onStopMonitor(); } catch (e) { setMonitorError(String(e)); }
                setMonitorBusy(false);
              }}
              className="flex items-center gap-1.5"
            >
              <Activity className="w-3.5 h-3.5" />
              {monitorBusy ? "Stopping..." : "Stop Monitor"}
            </Button>
          ) : (
            <Button
              variant="outline"
              disabled={monitorBusy}
              onClick={async () => {
                setMonitorBusy(true);
                setMonitorError(null);
                try { await onStartMonitor(); } catch (e) { setMonitorError(String(e)); }
                setMonitorBusy(false);
              }}
            >
              {monitorBusy ? "Starting..." : "Enable Background Monitor"}
            </Button>
          )}
        </div>
        {monitorError && <p className="text-xs text-destructive">{monitorError}</p>}
        <p className="text-xs text-muted-foreground">
          Last scan: {lastScan}
          {monitorRunning && " · Background monitor scans every 5s"}
        </p>
      </div>

      <Separator />

      {/* Known tools list */}
      <div>
        <p className="text-sm font-medium mb-3">Monitored Tools</p>
        <div className="flex flex-wrap gap-2">
          {KNOWN_TOOLS.map((tool) => (
            <Badge key={tool} variant="secondary" className="font-mono text-xs">
              {tool}
            </Badge>
          ))}
        </div>
      </div>
    </div>
  );
}
