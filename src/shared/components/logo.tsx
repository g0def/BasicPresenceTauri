import { cn } from "@/lib/utils";

interface LogoProps {
  className?: string;
}

/**
 * App logo: the brand icon followed by the "Basic Presence" wordmark set in
 * JetBrains Mono. The icon swaps between light/dark variants via the `.dark`
 * class on <html>, and is sized in `em` so it scales with the surrounding
 * font-size. The brand name is intentionally not translated.
 */
export function Logo({ className }: LogoProps) {
  return (
    <span
      className={cn(
        "inline-flex items-center gap-2 font-jetbrains font-semibold tracking-tight text-foreground",
        className,
      )}
    >
      <img
        src="/basic-presence-icon/logo.svg"
        alt=""
        aria-hidden="true"
        className="block h-[1.4em] w-[1.4em] dark:hidden"
      />
      <img
        src="/basic-presence-icon/logo-dark.svg"
        alt=""
        aria-hidden="true"
        className="hidden h-[1.4em] w-[1.4em] dark:block"
      />
      Basic Presence
    </span>
  );
}
