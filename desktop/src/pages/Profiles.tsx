import { useState } from "react";
import { Shield, FileText, Trash2, CheckCircle2, Circle } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import type { Profile } from "@/types";

interface Props {
  profiles: Profile[];
  onActivate: (name: string) => void;
  onDeactivate: (name: string) => void;
  onDelete: (name: string) => void;
}

export default function Profiles({ profiles, onActivate, onDeactivate, onDelete }: Props) {
  const [selected, setSelected] = useState<string>(profiles[0]?.name ?? "");
  const selectedProfile = profiles.find((p) => p.name === selected) ?? null;

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
          <Button variant="outline" size="sm" className="w-full">
            + Create Profile
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
                  <Button variant="outline" size="sm">+ Add File</Button>
                  <Button variant="outline" size="sm">+ Add Glob</Button>
                </div>
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
                      onClick={() => onDeactivate(selectedProfile.name)}
                      className="flex items-center gap-1.5"
                    >
                      <Circle className="w-3.5 h-3.5" />
                      Deactivate
                    </Button>
                  ) : (
                    <Button
                      size="sm"
                      onClick={() => onActivate(selectedProfile.name)}
                      className="flex items-center gap-1.5"
                    >
                      <CheckCircle2 className="w-3.5 h-3.5" />
                      Activate
                    </Button>
                  )}
                  <Button
                    variant="destructive"
                    size="sm"
                    onClick={() => {
                      onDelete(selectedProfile.name);
                      setSelected(profiles.find((p) => p.name !== selectedProfile.name)?.name ?? "");
                    }}
                    className="flex items-center gap-1.5"
                  >
                    <Trash2 className="w-3.5 h-3.5" />
                    Delete
                  </Button>
                </div>
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
    </div>
  );
}
