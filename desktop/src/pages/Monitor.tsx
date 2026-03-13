import { RefreshCw, ShieldAlert, ShieldCheck } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import type { DetectedProcess, Profile } from "@/types";

const KNOWN_TOOLS = [
  "codex",
  "claude",
  "claude-code",
  "opencode",
  "openhands",
  "cursor",
  "copilot",
  "windsurf",
  "antigravity",
];

interface Props {
  processes: DetectedProcess[];
  activeProfile: Profile | null;
  lastScan: string;
  onScan: () => void;
}

export default function Monitor({ processes, activeProfile, lastScan, onScan }: Props) {
  const isDetected = processes.length > 0;

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
              <div className="grid grid-cols-3 text-xs font-medium text-muted-foreground uppercase tracking-wide pb-2 border-b">
                <span>Process</span>
                <span>PID</span>
                <span>Status</span>
              </div>
              {processes.map((p) => (
                <div key={p.pid} className="grid grid-cols-3 text-sm py-1.5">
                  <span className="font-medium">{p.name}</span>
                  <span className="font-mono text-muted-foreground">{p.pid}</span>
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
      {isDetected && activeProfile && (
        <Card className="border-amber-500/30 bg-amber-50/50 dark:bg-amber-950/20">
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-amber-700 dark:text-amber-400">
              Protection Triggered
            </CardTitle>
          </CardHeader>
          <CardContent className="text-sm space-y-1">
            <p>Profile: <span className="font-medium">{activeProfile.name}</span></p>
            <p className="text-muted-foreground">Files would be encrypted on detection</p>
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
          <Button variant="outline" disabled>
            Enable Background Monitor
          </Button>
        </div>
        <p className="text-xs text-muted-foreground">Last scan: {lastScan}</p>
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
