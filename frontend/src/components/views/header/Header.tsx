import { Button } from "@/components/elements/button/Button";
import styles from "./Header.module.scss";
import Link from "next/link";

export const Header = () => {
  return (
    <header className={styles.header}>
      <Link href="/">
        <h3>Containers</h3>
      </Link>

      <Button variant="primary">Login</Button>
    </header>
  );
};
