export function dateStringToDiffString(dateTime: string): string {
  const lastModified = new Date(dateTime);
  return getDiffString(lastModified);
}

export function getDiffString(lastModified: Date): string {
  const now = new Date();

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
