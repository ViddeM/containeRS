import { Button } from "@/components/elements/button/Button";
import styles from "./page.module.css";
import { ImagesList } from "@/components/views/images-list/ImagesList";

export default function Home() {
  return (
    <main className={styles.main}>
      <ImagesList />
    </main>
  );
}
