import { useEffect, useRef } from "react";

import { Crepe } from "@milkdown/crepe";

import "@milkdown/crepe/theme/common/style.css";
import "@milkdown/crepe/theme/frame.css";

interface DayNoteEditorProps {
  /** Initial Markdown; read once at mount (the editor is uncontrolled). */
  defaultValue: string;
  placeholder?: string;
  onChange: (markdown: string) => void;
}

/**
 * Milkdown (Crepe) WYSIWYG editor for a day's note: renders Markdown inline as
 * the user types. The editor is uncontrolled — `defaultValue` seeds it at mount
 * and edits are pushed out via `onChange` (serialized back to Markdown). The
 * raw Markdown remains the source of truth; the backend re-renders it to safe
 * HTML for read-only display.
 */
export function DayNoteEditor({
  defaultValue,
  placeholder,
  onChange,
}: DayNoteEditorProps) {
  const rootRef = useRef<HTMLDivElement>(null);
  // Latest handler via a ref so the editor is created exactly once.
  const onChangeRef = useRef(onChange);
  onChangeRef.current = onChange;
  const initialRef = useRef(defaultValue);
  const placeholderRef = useRef(placeholder);

  useEffect(() => {
    const root = rootRef.current;
    if (!root) return;

    const crepe = new Crepe({
      root,
      defaultValue: initialRef.current,
      featureConfigs: {
        [Crepe.Feature.Placeholder]: {
          text: placeholderRef.current ?? "",
          mode: "doc",
        },
      },
    });
    crepe.on((listener) => {
      listener.markdownUpdated((_, md) => onChangeRef.current(md));
    });

    let destroyed = false;
    void crepe.create().then(() => {
      if (destroyed) return;
      root.querySelector<HTMLElement>(".ProseMirror")?.focus();
    });
    return () => {
      destroyed = true;
      void crepe.destroy();
    };
    // Mount once: uncontrolled editor reading the refs above for latest handlers.
  }, []);

  return <div ref={rootRef} className="milkdown-host" />;
}
