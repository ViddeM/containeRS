import styles from "./page.module.scss";
import { ImagesList } from "@/components/views/images-list/ImagesList";

export default function Home() {
  return (
    <main className={styles.main}>
      <ImagesList />
    </main>
  );
}
