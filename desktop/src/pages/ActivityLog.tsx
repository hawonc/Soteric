import { Download } from "lucide-react";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import type { ActivityEntry } from "@/types";

interface Props {
  activity: ActivityEntry[];
}

function eventColor(event: string) {
  if (event.includes("detected") || event.includes("encrypted")) return "text-destructive";
  if (event.includes("activated") || event.includes("completed")) return "text-emerald-600 dark:text-emerald-400";
  return "text-foreground";
}

export default function ActivityLog({ activity }: Props) {
  return (
    <div className="p-6 space-y-4 max-w-2xl">
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-2xl font-semibold tracking-tight">Activity Log</h1>
          <p className="text-sm text-muted-foreground">{activity.length} events recorded</p>
        </div>
        <Button variant="outline" size="sm" className="flex items-center gap-2">
          <Download className="w-4 h-4" />
          Export Log
        </Button>
      </div>

      <div className="border rounded-lg">
        <div className="grid grid-cols-[80px_1fr] text-xs font-medium text-muted-foreground uppercase tracking-wide px-4 py-2.5 border-b bg-muted/40">
          <span>Time</span>
          <span>Event</span>
        </div>
        <ScrollArea className="h-96">
          <div>
            {activity.map((entry, i) => (
              <div key={i}>
                <div className="grid grid-cols-[80px_1fr] px-4 py-3 text-sm hover:bg-muted/30 transition-colors">
                  <span className="font-mono text-muted-foreground">{entry.time}</span>
                  <span className={eventColor(entry.event)}>{entry.event}</span>
                </div>
                {i < activity.length - 1 && <Separator />}
              </div>
            ))}
          </div>
        </ScrollArea>
      </div>
    </div>
  );
}
