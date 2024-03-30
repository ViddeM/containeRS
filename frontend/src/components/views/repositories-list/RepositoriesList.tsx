"use client";

import { TextField } from "@/components/elements/textfield/TextField";
import styles from "./RepositoriesList.module.scss";
import { useState } from "react";
import { IconButton } from "@/components/elements/button/Button";
import { faAngleRight } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import Link from "next/link";
import { Repository } from "@/api/Repository";

export interface RepositoriesListProps {
  repositories: Repository[];
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

const RepositoryRow = ({ repo }: { repo: Repository }) => {
  const diffString = getDiffString(repo.lastModified);

  return (
    <div className={styles.repositoryRow}>
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

function getDiffString(dateTime: string) {
  const now = new Date();
  const lastModified = new Date(dateTime);

  const diff = now.getTime() - lastModified.getTime();
  const diffSeconds = Math.round(diff / 1000);

  const getScale = () => {
    if (diffSeconds < 60) {
      return { number: diffSeconds, unit: "second" };
    }

    const diffMinutes = (diffSeconds - (diffSeconds % 60)) / 60;
    if (diffMinutes < 60) {
      return { number: diffMinutes, unit: "minute" };
    }

    const diffHours = (diffMinutes - (diffMinutes % 60)) / 60;
    if (diffHours < 24) {
      return { number: diffHours, unit: "hour" };
    }

    const diffDays = (diffHours - (diffHours % 24)) / 24;
    if (diffDays < 30) {
      return { number: diffDays, unit: "day" };
    }

    if (diffDays < 365) {
      const diffMonths = (diffDays - (diffDays % 30)) / 30;
      return { number: diffMonths, unit: "month" };
    }

    const diffYears = (diffDays - (diffDays % 365)) / 365;
    return { number: diffYears, unit: "year" };
  };

  const diffObj = getScale();
  return `${diffObj.number} ${diffObj.unit}${
    diffObj.number > 1 ? "s" : ""
  } ago`;
}
