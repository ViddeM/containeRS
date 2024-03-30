import { getDiffString } from "@/util/DateUtil";
import styles from "./RepositoryView.module.scss";
import { Api } from "@/api/Api";
import { Error } from "@/components/views/error/Error";

export interface RepositoryViewProps {
  repositoryName: string;
}

export const RepositoryView = async ({
  repositoryName,
}: RepositoryViewProps) => {
  const data = await Api.repositories.getOne(repositoryName);

  if (!data.isSuccess) {
    let message = data.error || "unknown error";
    console.error("Failed to retrieve repository from server, response", data);
    return <Error message={message} />;
  }

  if (!data.data) {
    return <Error message={"Got no data from server"} />;
  }

  const repository = data.data!!;

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
