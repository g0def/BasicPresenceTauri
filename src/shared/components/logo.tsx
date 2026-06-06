import { cn } from "@/lib/utils";

interface LogoProps {
  className?: string;
}

/**
 * App wordmark: "Basic Presence 🌿" set in JetBrains Mono. The brand name is
 * intentionally not translated.
 */
export function Logo({ className }: LogoProps) {
  return (
    <span
      className={cn(
        "font-jetbrains font-semibold tracking-tight text-foreground",
        className,
      )}
    >
      Basic Presence <span aria-hidden="true">🌿</span>
    </span>
  );
}
