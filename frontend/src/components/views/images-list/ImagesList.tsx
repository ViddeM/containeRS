import { TextField } from "@/components/elements/textfield/TextField";
import styles from "./ImagesList.module.scss";

type Image = {
  name: string;
  author: string;
  lastModified: string;
};

const IMAGES: Image[] = [
  {
    name: "PelleSvans",
    author: "Vidde",
    lastModified: "2024-02-23T14:22:53",
  },
  {
    name: "Dallepoo",
    author: "Vidde",
    lastModified: "2023-01-21T09:19:12",
  },
  {
    name: "Leffeeeepo",
    author: "Vidde",
    lastModified: "2024-02-25T20:22:53",
  },
];

export const ImagesList = () => {
  return (
    <div className={`card ${styles.imageListCard}`}>
      <div className={styles.row}>
        <h3>Images</h3>
        <TextField postfixText="Search images" />
      </div>
      <div>
        {IMAGES.map((image) => (
          <ImageRow image={image} key={image.name} />
        ))}
      </div>
    </div>
  );
};

const ImageRow = ({ image }: { image: Image }) => {
  const diffString = getDiffString(image.lastModified);

  return (
    <div className={styles.imageRow}>
      <div className={styles.row}>
        <span>
          <b>{image.name}</b>
        </span>
        <span>{diffString}</span>
      </div>
      <div className={styles.row}>
        <p />
        {image.author}
      </div>
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
    if (diffSeconds < 60) {
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
