"use client";

import {
  FC,
  InputHTMLAttributes,
  MutableRefObject,
  useEffect,
  useRef,
} from "react";
import styles from "./TextField.module.scss";

export type TextFieldProps = InputHTMLAttributes<HTMLInputElement> & {
  postfixText?: string;
  inputClassName?: string;
  responsive?: boolean;
  focus?: boolean;
  externalRef?: MutableRefObject<HTMLInputElement | null>;
};

export const TextField: FC<TextFieldProps> = ({
  postfixText,
  className,
  inputClassName,
  required,
  responsive,
  focus,
  externalRef,
  ...props
}) => {
  const responsiveClass = responsive ? styles.responsive : "";
  const inputRef = useRef<HTMLInputElement | null>(null);

  useEffect(() => {
    if (focus) {
      inputRef.current?.focus();
    }
  }, [focus]);

  return (
    <div
      className={`${className} ${styles.textFieldContainer} ${responsiveClass}`}
    >
      <input
        className={`${styles.textFieldBase} ${inputClassName} `}
        {...props}
        required={required}
        ref={(node) => {
          inputRef.current = node;
          if (externalRef) {
            externalRef.current = node;
          }
        }}
      />
      {postfixText && <span>{postfixText}</span>}
      {required && <span className={styles.requiredTextField}>*</span>}
    </div>
  );
};
