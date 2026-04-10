import { useState } from "react";
import { KeyRound, Fingerprint, ArrowRightLeft, Trash2, Plus } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog";
import type { Profile, Mapping } from "@/types";

interface Props {
  secret: string;
  onSecretChange: (s: string) => void;
  biometricEnabled: boolean;
  profiles: Profile[];
  mappings: Mapping[];
  onChangeSecret: (current: string, next: string) => Promise<void>;
  onSetupBiometric: () => Promise<void>;
  onRemoveBiometric: () => Promise<void>;
  onAddMapping: (process: string, profile: string) => Promise<void>;
  onRemoveMapping: (process: string) => Promise<void>;
}

const KNOWN_TOOLS = [
  "codex", "claude", "claude-code", "opencode", "openhands",
  "cursor", "copilot", "windsurf", "antigravity",
];

export default function Settings({
  secret,
  onSecretChange,
  biometricEnabled,
  profiles,
  mappings,
  onChangeSecret,
  onSetupBiometric,
  onRemoveBiometric,
  onAddMapping,
  onRemoveMapping,
}: Props) {
  // Change secret
  const [newSecret, setNewSecret] = useState("");
  const [secretBusy, setSecretBusy] = useState(false);
  const [secretError, setSecretError] = useState<string | null>(null);
  const [secretSuccess, setSecretSuccess] = useState(false);

  // Biometric
  const [bioBusy, setBioBusy] = useState(false);
  const [bioError, setBioError] = useState<string | null>(null);

  // Mapping dialog
  const [mapOpen, setMapOpen] = useState(false);
  const [mapProcess, setMapProcess] = useState("");
  const [mapProfile, setMapProfile] = useState("");
  const [mapBusy, setMapBusy] = useState(false);
  const [mapError, setMapError] = useState<string | null>(null);

  async function handleChangeSecret() {
    setSecretBusy(true);
    setSecretError(null);
    setSecretSuccess(false);
    try {
      await onChangeSecret(secret, newSecret);
      setNewSecret("");
      setSecretSuccess(true);
    } catch (e) {
      setSecretError(String(e));
    }
    setSecretBusy(false);
  }

  async function handleBiometric(setup: boolean) {
    setBioBusy(true);
    setBioError(null);
    try {
      if (setup) await onSetupBiometric();
      else await onRemoveBiometric();
    } catch (e) {
      setBioError(String(e));
    }
    setBioBusy(false);
  }

  async function handleAddMapping() {
    if (!mapProcess.trim() || !mapProfile.trim()) return;
    setMapBusy(true);
    setMapError(null);
    try {
      await onAddMapping(mapProcess.trim(), mapProfile.trim());
      setMapOpen(false);
      setMapProcess("");
      setMapProfile("");
    } catch (e) {
      setMapError(String(e));
    }
    setMapBusy(false);
  }

  return (
    <div className="p-6 space-y-4 max-w-2xl">
      <div className="mb-6">
        <h1 className="text-2xl font-semibold tracking-tight">Settings</h1>
        <p className="text-sm text-muted-foreground">Manage encryption keys, biometrics, and process mappings</p>
      </div>

      {/* Secret Key */}
      <Card>
        <CardHeader className="pb-3">
          <CardTitle className="text-sm font-medium flex items-center gap-2">
            <KeyRound className="w-4 h-4" />
            Encryption Key
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="space-y-2">
            <Label>Current session key</Label>
            <Input
              type="password"
              placeholder="Enter encryption password..."
              value={secret}
              onChange={(e) => onSecretChange(e.target.value)}
              className="max-w-xs"
            />
            <p className="text-xs text-muted-foreground">
              {secret ? "Key set for this session" : "Required for encrypt/decrypt operations"}
            </p>
          </div>

          <Separator />

          <div className="space-y-2">
            <Label>Change secret key</Label>
            <p className="text-xs text-muted-foreground">Re-encrypts active profile files with the new key</p>
            <div className="flex gap-2 items-end">
              <Input
                type="password"
                placeholder="New secret key..."
                value={newSecret}
                onChange={(e) => setNewSecret(e.target.value)}
                className="max-w-xs"
              />
              <Button
                size="sm"
                disabled={!newSecret || !secret || secretBusy}
                onClick={handleChangeSecret}
              >
                {secretBusy ? "Changing..." : "Change"}
              </Button>
            </div>
            {secretError && <p className="text-xs text-destructive">{secretError}</p>}
            {secretSuccess && <p className="text-xs text-emerald-600">Secret changed successfully</p>}
          </div>
        </CardContent>
      </Card>

      {/* Biometric */}
      <Card>
        <CardHeader className="pb-3">
          <CardTitle className="text-sm font-medium flex items-center gap-2">
            <Fingerprint className="w-4 h-4" />
            Biometric Authentication
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="flex items-center gap-3">
            <Badge
              variant={biometricEnabled ? "default" : "secondary"}
              className={biometricEnabled ? "bg-emerald-600 hover:bg-emerald-600 text-white" : ""}
            >
              {biometricEnabled ? "Enabled" : "Disabled"}
            </Badge>
            <span className="text-sm text-muted-foreground">Touch ID (macOS only)</span>
          </div>
          <p className="text-xs text-muted-foreground">
            Store your encryption key in the macOS Keychain, protected by Touch ID.
          </p>
          {biometricEnabled ? (
            <Button
              variant="destructive"
              size="sm"
              disabled={bioBusy}
              onClick={() => handleBiometric(false)}
            >
              {bioBusy ? "Removing..." : "Remove Biometric"}
            </Button>
          ) : (
            <Button
              size="sm"
              disabled={bioBusy || !secret}
              onClick={() => handleBiometric(true)}
            >
              {bioBusy ? "Setting up..." : "Enable Touch ID"}
            </Button>
          )}
          {!secret && !biometricEnabled && (
            <p className="text-xs text-muted-foreground">Set a session key first to enable biometric</p>
          )}
          {bioError && <p className="text-xs text-destructive">{bioError}</p>}
        </CardContent>
      </Card>

      {/* Process Mappings */}
      <Card>
        <CardHeader className="pb-3">
          <CardTitle className="text-sm font-medium flex items-center gap-2">
            <ArrowRightLeft className="w-4 h-4" />
            Process → Profile Mappings
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <p className="text-xs text-muted-foreground">
            Auto-activate a profile when a specific AI tool is detected.
          </p>

          {mappings.length > 0 ? (
            <div className="space-y-2">
              <div className="grid grid-cols-[1fr_1fr_auto] text-xs font-medium text-muted-foreground uppercase tracking-wide pb-2 border-b">
                <span>Process</span>
                <span>Profile</span>
                <span />
              </div>
              {mappings.map((m) => (
                <div key={m.process} className="grid grid-cols-[1fr_1fr_auto] items-center text-sm py-1.5">
                  <span className="font-mono">{m.process}</span>
                  <span>{m.profile}</span>
                  <Button
                    variant="ghost"
                    size="sm"
                    className="h-7 w-7 p-0"
                    onClick={() => onRemoveMapping(m.process)}
                  >
                    <Trash2 className="w-3.5 h-3.5 text-muted-foreground" />
                  </Button>
                </div>
              ))}
            </div>
          ) : (
            <p className="text-sm text-muted-foreground">No mappings configured</p>
          )}

          <Button
            variant="outline"
            size="sm"
            onClick={() => { setMapProcess(""); setMapProfile(profiles[0]?.name ?? ""); setMapError(null); setMapOpen(true); }}
          >
            <Plus className="w-3.5 h-3.5 mr-1" />
            Add Mapping
          </Button>
        </CardContent>
      </Card>

      {/* Add Mapping Dialog */}
      <Dialog open={mapOpen} onOpenChange={setMapOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Add Process Mapping</DialogTitle>
          </DialogHeader>
          <div className="space-y-4 py-2">
            <div className="space-y-2">
              <Label>Process name</Label>
              <select
                className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                value={mapProcess}
                onChange={(e) => setMapProcess(e.target.value)}
              >
                <option value="">Select a process...</option>
                {KNOWN_TOOLS.map((t) => (
                  <option key={t} value={t}>{t}</option>
                ))}
              </select>
              <p className="text-xs text-muted-foreground">Or type a custom process name:</p>
              <Input
                placeholder="Custom process name..."
                value={mapProcess}
                onChange={(e) => setMapProcess(e.target.value)}
              />
            </div>
            <div className="space-y-2">
              <Label>Target profile</Label>
              <select
                className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                value={mapProfile}
                onChange={(e) => setMapProfile(e.target.value)}
              >
                <option value="">Select a profile...</option>
                {profiles.map((p) => (
                  <option key={p.name} value={p.name}>{p.name}</option>
                ))}
              </select>
            </div>
            {mapError && <p className="text-sm text-destructive">{mapError}</p>}
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setMapOpen(false)}>Cancel</Button>
            <Button onClick={handleAddMapping} disabled={!mapProcess || !mapProfile || mapBusy}>
              {mapBusy ? "Adding..." : "Add"}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
