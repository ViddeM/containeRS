export interface Repositories {
  repositories: Repository[];
}

export interface Repository {
  name: string;
  author: string;
  lastModified: string;
}
