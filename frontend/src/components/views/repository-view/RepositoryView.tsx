import { Repository } from "@/api/Repository";
import { getDiffString } from "@/util/DateUtil";
import styles from "./RepositoryView.module.scss";

export interface RepositoryViewProps {
  repository: Repository;
}

export const RepositoryView = ({ repository }: RepositoryViewProps) => {
  const tags = repository.tags
    .map((tag) => {
      return {
        ...tag,
        createdAt: new Date(tag.createdAt),
      };
    })
    .sort(
      (a, b) =>
        a.createdAt.getUTCMilliseconds() - b.createdAt.getUTCMilliseconds()
    )
    .map((tag) => {
      return {
        ...tag,
        createdAt: getDiffString(tag.createdAt),
      };
    });

  return (
    <div>
      <h3>Repository {repository.name}</h3>
      <p>{repository.author}</p>
      <div className={styles.tagsList}>
        {tags.map((tag) => (
          <div key={tag.name} className={`card ${styles.tagRow}`}>
            <p>{tag.name}</p>
            <p>{tag.createdAt}</p>
          </div>
        ))}
      </div>
    </div>
  );
};
