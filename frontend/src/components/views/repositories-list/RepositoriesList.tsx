"use client";

import { TextField } from "@/components/elements/textfield/TextField";
import styles from "./RepositoriesList.module.scss";
import { useState } from "react";
import { IconButton } from "@/components/elements/button/Button";
import { faAngleRight } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import Link from "next/link";
import { ListRepository } from "@/api/Repository";
import { dateStringToDiffString, getDiffString } from "@/util/DateUtil";

export interface RepositoriesListProps {
  repositories: ListRepository[];
}

export const RepositoriesList = ({ repositories }: RepositoriesListProps) => {
  const [filterText, setFilterText] = useState<string>("");

  const filteredRepos = repositories.filter(
    (i) => i.name.includes(filterText) || i.author.includes(filterText)
  );

  return (
    <div className={`${styles.repositoriesListCard}`}>
      <h3>Repositories</h3>
      <TextField
        maxLength={100}
        placeholder="Search repositories"
        className={`${styles.searchField} margin-top`}
        inputClassName={styles.searchField}
        value={filterText}
        onChange={(e) => setFilterText(e.target.value)}
      />

      <div>
        {filteredRepos.map((repo) => (
          <RepositoryRow repo={repo} key={repo.name} />
        ))}
      </div>
    </div>
  );
};

const RepositoryRow = ({ repo }: { repo: ListRepository }) => {
  const diffString = dateStringToDiffString(repo.lastModified);

  return (
    <div className={`${styles.repositoryRow} card`}>
      <div className={styles.col}>
        <div className={styles.row}>
          <p>
            <b>{repo.name}</b>
          </p>
          <p>{diffString}</p>
        </div>
        <div className={styles.row}>
          <p />
          <p>{repo.author}</p>
        </div>
      </div>
      <Link href={`/repositories/${repo.name}`}>
        <IconButton className={"margin-left margin-right"}>
          <FontAwesomeIcon icon={faAngleRight} />
        </IconButton>
      </Link>
    </div>
  );
};
