import { Button } from "@/components/elements/button/Button";
import styles from "./Header.module.scss";

export const Header = () => {
  return (
    <header className={styles.header}>
      <h3>Containers</h3>

      <Button variant="primary">Login</Button>
    </header>
  );
};
