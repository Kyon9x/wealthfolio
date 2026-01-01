import * as React from "react";
import { cn } from "../../lib/utils";
import { Input } from "../ui/input";

export interface InputProps extends React.InputHTMLAttributes<HTMLInputElement> {
  currency?: string;
  maxDecimalPlaces?: number;
}

const MoneyInput = React.forwardRef<HTMLInputElement, InputProps>(
  ({ className, maxDecimalPlaces = 6, value, onChange, ...props }, ref) => {
    const { placeholder = "0.00" } = props;

    // Ensure value is always a string
    const controlledValue = value === undefined || value === null ? "" : value.toString();

    // Track cursor position to preserve it during edits
    const cursorPositionRef = React.useRef<number>(0);
    const inputRef = React.useRef<HTMLInputElement | null>(null);

    // Combine refs to ensure we have access to the input element
    React.useImperativeHandle(ref, () => inputRef.current!);

    // Restore cursor position after value changes
    React.useEffect(() => {
      if (inputRef.current && document.activeElement === inputRef.current) {
        inputRef.current.setSelectionRange(cursorPositionRef.current, cursorPositionRef.current);
      }
    }, [controlledValue]);

    const formatCurrency = (value: string): string => {
      const numericValue = parseFloat(value);
      return isNaN(numericValue)
        ? ""
        : numericValue.toLocaleString(undefined, {
            minimumFractionDigits: 2,
            maximumFractionDigits: maxDecimalPlaces,
          });
    };

    const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
      // Store cursor position before processing
      const cursorPos = e.target.selectionStart ?? 0;
      let rawValue = e.target.value.replace(/[^\d.]/g, "");

      // Ensure only one decimal point
      const decimalIndex = rawValue.indexOf(".");
      if (decimalIndex !== -1) {
        rawValue = rawValue.slice(0, decimalIndex + 1) + rawValue.slice(decimalIndex + 1).replace(/\./g, "");
      }

      const formattedValue = formatCurrency(rawValue);

      // Calculate cursor position adjustment for comma formatting
      // Count formatting characters (commas) added before the cursor position
      let adjustedCursorPos = cursorPos;
      if (formattedValue && cursorPos > 0) {
        // Get the portion of formatted value before the cursor position
        const beforeCursor = formattedValue.substring(0, Math.min(cursorPos, formattedValue.length));
        // Count formatting characters (commas) in this portion
        const formattingCharCount = (beforeCursor.match(/,/g) || []).length;
        adjustedCursorPos = cursorPos + formattingCharCount;
        // Ensure the adjusted position doesn't exceed the formatted value length
        adjustedCursorPos = Math.min(adjustedCursorPos, formattedValue.length);
      }

      // Update the input value with the formatted amount
      e.target.value = formattedValue;

      // Store adjusted cursor position for restoration after re-render
      cursorPositionRef.current = adjustedCursorPos;

      // Call the original onChange with the numeric value
      if (onChange) {
        const numericValue = parseFloat(rawValue);
        const syntheticEvent = {
          ...e,
          target: { ...e.target, value: isNaN(numericValue) ? "" : rawValue },
        };
        onChange(syntheticEvent as React.ChangeEvent<HTMLInputElement>);
      }
    };

    return (
      <Input
        className={cn("text-right", className)}
        ref={inputRef}
        {...props}
        value={controlledValue}
        onChange={handleChange}
        placeholder={placeholder}
      />
    );
  },
);
MoneyInput.displayName = "MoneyInput";

export { MoneyInput };
