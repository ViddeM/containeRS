import { ButtonHTMLAttributes } from "react";
import styles from "./Button.module.scss";

export type ButtonVariant = {
  variant?: "primary" | "secondary";
};

export type ButtonProps = ButtonHTMLAttributes<HTMLButtonElement> &
  ButtonVariant;

export const Button = ({ className, variant, ...props }: ButtonProps) => {
  const style = `${className} ${styles.baseButtonStyle} ${
    styles[`button-${variant}`]
  }`;

  return <button className={style} {...props}></button>;
};
