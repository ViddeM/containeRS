export interface Repositories {
  repositories: ListRepository[];
}

export interface ListRepository {
  name: string;
  author: string;
  lastModified: string;
}

export interface Repository {
  name: string;
  author: string;
  tags: Tag[];
}

export interface Tag {
  name: string;
  createdAt: string;
}
