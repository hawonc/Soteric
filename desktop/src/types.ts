export interface Profile {
  name: string;
  root: string;
  files: string[];
  active: boolean;
  encrypted: boolean;
}

export interface DetectedProcess {
  name: string;
  pid: number;
  command: string;
}

export interface ActivityEntry {
  time: string;
  event: string;
}

export interface Mapping {
  process: string;
  profile: string;
}
