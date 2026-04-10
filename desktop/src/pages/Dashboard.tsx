import { useState } from "react";
import { Shield, Bot, Zap, Clock, KeyRound } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import { Input } from "@/components/ui/input";
import type { Profile, DetectedProcess, ActivityEntry } from "@/types";

interface Props {
  activeProfile: Profile | null;
  processes: DetectedProcess[];
  activity: ActivityEntry[];
  lastScan: string;
  secret: string;
  onSecretChange: (secret: string) => void;
  onScan: () => void;
  onEncrypt: () => Promise<void>;
  onDecrypt: () => Promise<void>;
  onNavigate: (page: "dashboard" | "profiles" | "monitor" | "activity") => void;
}

export default function Dashboard({
  activeProfile,
  processes,
  activity,
  lastScan,
  secret,
  onSecretChange,
  onScan,
  onEncrypt,
  onDecrypt,
  onNavigate,
}: Props) {
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const isDetected = processes.length > 0;

  return (
    <div className="p-6 space-y-4 max-w-2xl">
      {/* Status header */}
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-2xl font-semibold tracking-tight">Dashboard</h1>
          <p className="text-sm text-muted-foreground">AI Tool Protection for Sensitive Files</p>
        </div>
        <Badge
          variant={isDetected ? "destructive" : "default"}
          className={`text-sm px-3 py-1 ${!isDetected ? "bg-emerald-600 hover:bg-emerald-600 text-white" : ""}`}
        >
          <span className={`mr-2 inline-block w-2 h-2 rounded-full ${isDetected ? "bg-white" : "bg-emerald-300"}`} />
          {isDetected ? "AI Detected" : "Protected"}
        </Badge>
      </div>

      {/* Active Profile */}
      <Card>
        <CardHeader className="pb-3">
          <CardTitle className="text-sm font-medium flex items-center gap-2">
            <Shield className="w-4 h-4" />
            Active Profile
          </CardTitle>
        </CardHeader>
        <CardContent>
          {activeProfile ? (
            <div className="space-y-1">
              <p className="font-semibold text-lg">{activeProfile.name}</p>
              <p className="text-sm text-muted-foreground">{activeProfile.root}</p>
              <div className="flex items-center gap-2 text-sm">
                <span className="text-muted-foreground">{activeProfile.files.length} protected files</span>
                <Badge
                  variant={activeProfile.encrypted ? "default" : "secondary"}
                  className={`text-xs ${activeProfile.encrypted ? "bg-amber-600 hover:bg-amber-600 text-white" : ""}`}
                >
                  {activeProfile.encrypted ? "Encrypted" : "Decrypted"}
                </Badge>
              </div>
              <Button
                variant="outline"
                size="sm"
                className="mt-3"
                onClick={() => onNavigate("profiles")}
              >
                Manage Profile
              </Button>
            </div>
          ) : (
            <div className="space-y-2">
              <p className="text-sm text-muted-foreground">No active profile</p>
              <Button variant="outline" size="sm" onClick={() => onNavigate("profiles")}>
                Set Up Profile
              </Button>
            </div>
          )}
        </CardContent>
      </Card>

      {/* AI Tool Detection */}
      <Card>
        <CardHeader className="pb-3">
          <CardTitle className="text-sm font-medium flex items-center gap-2">
            <Bot className="w-4 h-4" />
            AI Tool Detection
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          {isDetected ? (
            <div className="space-y-2">
              {processes.map((p) => (
                <div key={p.pid} className="flex items-center justify-between text-sm">
                  <span className="font-medium">{p.name}</span>
                  <Badge variant="destructive">PID {p.pid}</Badge>
                </div>
              ))}
            </div>
          ) : (
            <p className="text-sm text-emerald-600 dark:text-emerald-400 font-medium">
              No AI tools detected
            </p>
          )}
          <div className="flex items-center gap-2 text-xs text-muted-foreground">
            <Clock className="w-3 h-3" />
            Last scan: {lastScan}
          </div>
          <Button variant="outline" size="sm" onClick={onScan}>
            Scan Now
          </Button>
        </CardContent>
      </Card>

      {/* Secret Key */}
      <Card>
        <CardHeader className="pb-3">
          <CardTitle className="text-sm font-medium flex items-center gap-2">
            <KeyRound className="w-4 h-4" />
            Secret Key
          </CardTitle>
        </CardHeader>
        <CardContent>
          <Input
            type="password"
            placeholder="Enter encryption password..."
            value={secret}
            onChange={(e) => onSecretChange(e.target.value)}
            className="max-w-xs"
          />
          <p className="text-xs text-muted-foreground mt-2">
            {secret ? "Key set for this session" : "Required for encrypt/decrypt operations"}
          </p>
        </CardContent>
      </Card>

      {/* Quick Actions */}
      <Card>
        <CardHeader className="pb-3">
          <CardTitle className="text-sm font-medium flex items-center gap-2">
            <Zap className="w-4 h-4" />
            Quick Actions
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex gap-2">
            <Button
              variant="outline"
              size="sm"
              disabled={!activeProfile || busy || (activeProfile?.encrypted ?? false)}
              onClick={async () => {
                setBusy(true);
                setError(null);
                try { await onEncrypt(); } catch (e) { setError(String(e)); }
                setBusy(false);
              }}
            >
              {busy ? "Working..." : "Encrypt Files"}
            </Button>
            <Button
              variant="outline"
              size="sm"
              disabled={!activeProfile || busy || !(activeProfile?.encrypted ?? false)}
              onClick={async () => {
                setBusy(true);
                setError(null);
                try { await onDecrypt(); } catch (e) { setError(String(e)); }
                setBusy(false);
              }}
            >
              {busy ? "Working..." : "Decrypt Files"}
            </Button>
          </div>
          {error && (
            <p className="text-xs text-destructive mt-2">{error}</p>
          )}
          {!activeProfile && (
            <p className="text-xs text-muted-foreground mt-2">Activate a profile to enable encryption</p>
          )}
        </CardContent>
      </Card>

      <Separator />

      {/* Recent Activity */}
      <div>
        <p className="text-sm font-medium mb-3">Recent Activity</p>
        <div className="space-y-2">
          {activity.slice(0, 4).map((entry, i) => (
            <div key={i} className="flex items-start gap-3 text-sm">
              <span className="text-muted-foreground font-mono text-xs w-10 shrink-0 pt-0.5">
                {entry.time}
              </span>
              <span className="text-foreground">{entry.event}</span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
