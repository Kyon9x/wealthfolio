/**
 * MessageInput - Text input for sending chat messages.
 */

import { useState, useRef, useEffect, KeyboardEvent } from "react";

import { Button, Icons, Textarea } from "@/ui";

interface MessageInputProps {
  onSend: (content: string) => void;
  onCancel?: () => void;
  isStreaming?: boolean;
}

export function MessageInput({ onSend, onCancel, isStreaming }: MessageInputProps) {
  const [value, setValue] = useState("");
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  // Auto-resize textarea
  useEffect(() => {
    const textarea = textareaRef.current;
    if (!textarea) return;

    textarea.style.height = "auto";
    textarea.style.height = `${Math.min(textarea.scrollHeight, 200)}px`;
  }, [value]);

  const handleSend = () => {
    const trimmed = value.trim();
    if (!trimmed || isStreaming) return;

    onSend(trimmed);
    setValue("");
  };

  const handleKeyDown = (e: KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  return (
    <div className="relative">
      <Textarea
        ref={textareaRef}
        value={value}
        onChange={(e) => setValue(e.target.value)}
        onKeyDown={handleKeyDown}
        placeholder="Send a message..."
        className="min-h-[60px] max-h-[200px] pr-24 resize-none"
        disabled={isStreaming}
      />
      <div className="absolute right-2 bottom-2 flex items-center gap-1">
        {isStreaming ? (
          <Button
            type="button"
            variant="destructive"
            size="sm"
            className="h-8 px-3"
            onClick={onCancel}
          >
            <div className="size-3 bg-white rounded-full mr-1" />
            Stop
          </Button>
        ) : (
          <Button
            type="button"
            size="sm"
            className="h-8 px-3"
            onClick={handleSend}
            disabled={!value.trim()}
          >
            <Icons.ArrowUp className="size-4" />
          </Button>
        )}
      </div>
    </div>
  );
}
