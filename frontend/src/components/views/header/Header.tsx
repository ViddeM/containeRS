import { Button } from "@/components/elements/button/Button";
import styles from "./Header.module.scss";

export const Header = () => {
  return (
    <header className={styles.header}>
      <h2>Containers</h2>

      <Button variant="primary">Login</Button>
    </header>
  );
};
