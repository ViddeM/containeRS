export interface RepositoryViewProps {
  repository: string;
}

export const RepositoryView = ({ repository }: RepositoryViewProps) => {
  return (
    <div>
      <h3>
        Repository {repository} ({REPOSITORY.name})
      </h3>
      <p>{REPOSITORY.author}</p>
      {REPOSITORY.tags.map((tag) => (
        <div key={tag}>{tag}</div>
      ))}
    </div>
  );
};

const REPOSITORY = {
  name: "Pelle",
  author: "Lars",
  tags: ["v0.1.2", "v0.3.9", "v0.5.1-alpine"],
};
