import { useState, useEffect } from "react";
import { Shield, FileText, Trash2, CheckCircle2, Circle, Plus } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog";
import type { Profile } from "@/types";

interface Props {
  profiles: Profile[];
  onActivate: (name: string) => Promise<void>;
  onDeactivate: (name: string) => Promise<void>;
  onDelete: (name: string) => Promise<void>;
  onCreate: (name: string, files: string[], globs: string[]) => Promise<void>;
  onAppend: (name: string, files: string[], globs: string[]) => Promise<void>;
  onNavigate: (page: "dashboard" | "profiles" | "monitor" | "activity" | "settings") => void;
}

export default function Profiles({ profiles, onActivate, onDeactivate, onDelete, onCreate, onAppend, onNavigate }: Props) {
  const [selected, setSelected] = useState<string>(profiles[0]?.name ?? "");
  const selectedProfile = profiles.find((p) => p.name === selected) ?? null;

  useEffect(() => {
    if (selected === "" || !profiles.some((p) => p.name === selected)) {
      setSelected(profiles[0]?.name ?? "");
    }
  }, [profiles, selected]);

  // Create profile dialog state
  const [createOpen, setCreateOpen] = useState(false);
  const [newName, setNewName] = useState("");
  const [newFiles, setNewFiles] = useState("");
  const [newGlobs, setNewGlobs] = useState("");
  const [createError, setCreateError] = useState<string | null>(null);
  const [createBusy, setCreateBusy] = useState(false);

  // Append dialog state
  const [appendOpen, setAppendOpen] = useState(false);
  const [appendMode, setAppendMode] = useState<"file" | "glob">("file");
  const [appendValue, setAppendValue] = useState("");
  const [appendError, setAppendError] = useState<string | null>(null);
  const [appendBusy, setAppendBusy] = useState(false);

  // Action error/busy state
  const [actionError, setActionError] = useState<string | null>(null);
  const [actionBusy, setActionBusy] = useState(false);

  async function handleCreate() {
    if (!newName.trim()) return;
    setCreateBusy(true);
    setCreateError(null);
    try {
      const files = newFiles.split("\n").map((s) => s.trim()).filter(Boolean);
      const globs = newGlobs.split("\n").map((s) => s.trim()).filter(Boolean);
      await onCreate(newName.trim(), files, globs);
      setCreateOpen(false);
      setNewName("");
      setNewFiles("");
      setNewGlobs("");
      setSelected(newName.trim());
    } catch (e) {
      setCreateError(String(e));
    }
    setCreateBusy(false);
  }

  async function handleAppend() {
    if (!selectedProfile || !appendValue.trim()) return;
    setAppendBusy(true);
    setAppendError(null);
    try {
      const files = appendMode === "file" ? appendValue.split("\n").map((s) => s.trim()).filter(Boolean) : [];
      const globs = appendMode === "glob" ? appendValue.split("\n").map((s) => s.trim()).filter(Boolean) : [];
      await onAppend(selectedProfile.name, files, globs);
      setAppendOpen(false);
      setAppendValue("");
    } catch (e) {
      setAppendError(String(e));
    }
    setAppendBusy(false);
  }

  return (
    <div className="p-6 space-y-4">
      <div className="mb-6">
        <h1 className="text-2xl font-semibold tracking-tight">Profiles</h1>
        <p className="text-sm text-muted-foreground">Manage your file protection profiles</p>
      </div>

      <div className="grid grid-cols-[220px_1fr] gap-4 items-start">
        {/* Profile list */}
        <div className="space-y-2">
          <ScrollArea className="h-64">
            <div className="space-y-1 pr-2">
              {profiles.map((profile) => (
                <button
                  key={profile.name}
                  onClick={() => setSelected(profile.name)}
                  className={`w-full text-left px-3 py-2.5 rounded-lg text-sm transition-colors ${
                    selected === profile.name
                      ? "bg-primary text-primary-foreground"
                      : "hover:bg-muted"
                  }`}
                >
                  <div className="flex items-center justify-between">
                    <span className="font-medium">{profile.name}</span>
                    {profile.active && (
                      <span className="w-1.5 h-1.5 rounded-full bg-emerald-400 shrink-0" />
                    )}
                  </div>
                  <div
                    className={`text-xs mt-0.5 ${
                      selected === profile.name ? "text-primary-foreground/70" : "text-muted-foreground"
                    }`}
                  >
                    {profile.files.length} files
                  </div>
                </button>
              ))}
            </div>
          </ScrollArea>
          <Button
            variant="outline"
            size="sm"
            className="w-full"
            onClick={() => setCreateOpen(true)}
          >
            <Plus className="w-3.5 h-3.5 mr-1" />
            Create Profile
          </Button>
        </div>

        {/* Profile detail */}
        {selectedProfile ? (
          <Card>
            <CardHeader className="pb-3">
              <CardTitle className="flex items-center gap-2 text-base">
                <Shield className="w-4 h-4" />
                {selectedProfile.name}
                {selectedProfile.active && (
                  <Badge className="bg-emerald-600 hover:bg-emerald-600 text-white text-xs ml-1">
                    Active
                  </Badge>
                )}
                {selectedProfile.active && (
                  <Badge
                    variant={selectedProfile.encrypted ? "default" : "secondary"}
                    className={`text-xs ml-1 ${selectedProfile.encrypted ? "bg-amber-600 hover:bg-amber-600 text-white" : ""}`}
                  >
                    {selectedProfile.encrypted ? "Encrypted" : "Decrypted"}
                  </Badge>
                )}
              </CardTitle>
              <p className="text-sm text-muted-foreground">{selectedProfile.root}</p>
            </CardHeader>
            <CardContent className="space-y-4">
              {/* Files */}
              <div>
                <p className="text-xs font-medium text-muted-foreground uppercase tracking-wide mb-2">
                  Protected Files
                </p>
                <ScrollArea className="h-36">
                  <div className="space-y-1.5">
                    {selectedProfile.files.map((file) => (
                      <div key={file} className="flex items-center gap-2 text-sm">
                        <FileText className="w-3.5 h-3.5 text-muted-foreground shrink-0" />
                        <span className="font-mono text-xs">{file}</span>
                      </div>
                    ))}
                  </div>
                </ScrollArea>
                <div className="flex gap-2 mt-3">
                  <Button
                    variant="outline"
                    size="sm"
                    disabled={selectedProfile.active}
                    onClick={() => { setAppendMode("file"); setAppendValue(""); setAppendError(null); setAppendOpen(true); }}
                  >
                    + Add File
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    disabled={selectedProfile.active}
                    onClick={() => { setAppendMode("glob"); setAppendValue(""); setAppendError(null); setAppendOpen(true); }}
                  >
                    + Add Glob
                  </Button>
                </div>
                {selectedProfile.active && (
                  <p className="text-xs text-muted-foreground mt-1">Deactivate profile to add files</p>
                )}
              </div>

              <Separator />

              {/* Actions */}
              <div>
                <p className="text-xs font-medium text-muted-foreground uppercase tracking-wide mb-2">
                  Actions
                </p>
                <div className="flex gap-2 flex-wrap">
                  {selectedProfile.active ? (
                    <Button
                      variant="outline"
                      size="sm"
                      disabled={actionBusy}
                      onClick={async () => {
                        setActionBusy(true);
                        setActionError(null);
                        try { await onDeactivate(selectedProfile.name); } catch (e) { setActionError(String(e)); }
                        setActionBusy(false);
                      }}
                      className="flex items-center gap-1.5"
                    >
                      <Circle className="w-3.5 h-3.5" />
                      {actionBusy ? "Working..." : "Deactivate"}
                    </Button>
                  ) : (
                    <Button
                      size="sm"
                      disabled={actionBusy}
                      onClick={async () => {
                        setActionBusy(true);
                        setActionError(null);
                        try {
                          await onActivate(selectedProfile.name);
                          onNavigate("dashboard");
                        } catch (e) { setActionError(String(e)); }
                        setActionBusy(false);
                      }}
                      className="flex items-center gap-1.5"
                    >
                      <CheckCircle2 className="w-3.5 h-3.5" />
                      {actionBusy ? "Working..." : "Activate"}
                    </Button>
                  )}
                  <Button
                    variant="destructive"
                    size="sm"
                    disabled={selectedProfile.active || actionBusy}
                    onClick={async () => {
                      setActionBusy(true);
                      setActionError(null);
                      try {
                        await onDelete(selectedProfile.name);
                        setSelected(profiles.find((p) => p.name !== selectedProfile.name)?.name ?? "");
                      } catch (e) { setActionError(String(e)); }
                      setActionBusy(false);
                    }}
                    className="flex items-center gap-1.5"
                  >
                    <Trash2 className="w-3.5 h-3.5" />
                    Delete
                  </Button>
                </div>
                {selectedProfile.active && !actionError && (
                  <p className="text-xs text-muted-foreground mt-1">Deactivate profile before deleting</p>
                )}
                {actionError && (
                  <p className="text-xs text-destructive mt-2">{actionError}</p>
                )}
              </div>
            </CardContent>
          </Card>
        ) : (
          <Card>
            <CardContent className="flex items-center justify-center h-48">
              <p className="text-sm text-muted-foreground">No profiles yet</p>
            </CardContent>
          </Card>
        )}
      </div>

      {/* Create Profile Dialog */}
      <Dialog open={createOpen} onOpenChange={setCreateOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Create Profile</DialogTitle>
          </DialogHeader>
          <div className="space-y-4 py-2">
            <div className="space-y-2">
              <Label htmlFor="profile-name">Profile Name</Label>
              <Input
                id="profile-name"
                placeholder="e.g. secrets"
                value={newName}
                onChange={(e) => setNewName(e.target.value)}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="profile-files">Files (one per line, absolute or project‑relative)</Label>
              <textarea
                id="profile-files"
                className="flex w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring min-h-[80px] font-mono"
                placeholder={"secrets.txt\n.env"}
                value={newFiles}
                onChange={(e) => setNewFiles(e.target.value)}
              />
              <p className="text-xs text-muted-foreground">
                Paths are resolved from your project root (the folder with <code>.git</code>) and must point to existing files.
              </p>
            </div>
            <div className="space-y-2">
              <Label htmlFor="profile-globs">Globs (one per line, optional, project‑relative)</Label>
              <textarea
                id="profile-globs"
                className="flex w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring min-h-[60px] font-mono"
                placeholder={"*.env\ntemp/*.txt"}
                value={newGlobs}
                onChange={(e) => setNewGlobs(e.target.value)}
              />
            </div>
            {createError && (
              <p className="text-sm text-destructive">{createError}</p>
            )}
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setCreateOpen(false)}>Cancel</Button>
            <Button onClick={handleCreate} disabled={!newName.trim() || createBusy}>
              {createBusy ? "Creating..." : "Create"}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Append File/Glob Dialog */}
      <Dialog open={appendOpen} onOpenChange={setAppendOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>
              Add {appendMode === "file" ? "Files" : "Globs"} to {selectedProfile?.name}
            </DialogTitle>
          </DialogHeader>
          <div className="space-y-4 py-2">
            <div className="space-y-2">
              <Label htmlFor="append-value">
                {appendMode === "file" ? "File paths (one per line)" : "Glob patterns (one per line)"}
              </Label>
              <textarea
                id="append-value"
                className="flex w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring min-h-[80px] font-mono"
                placeholder={appendMode === "file" ? "/path/to/file.txt" : "./.*"}
                value={appendValue}
                onChange={(e) => setAppendValue(e.target.value)}
              />
            </div>
            {appendError && (
              <p className="text-sm text-destructive">{appendError}</p>
            )}
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setAppendOpen(false)}>Cancel</Button>
            <Button onClick={handleAppend} disabled={!appendValue.trim() || appendBusy}>
              {appendBusy ? "Adding..." : "Add"}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
