export interface Profile {
  name: string;
  root: string;
  files: string[];
  active: boolean;
}

export interface DetectedProcess {
  name: string;
  pid: number;
}

export interface ActivityEntry {
  time: string;
  event: string;
}
