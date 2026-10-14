import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { useExternalAnchors } from "@/shared/rendered-html/use-external-anchors";

const openExternal = vi.hoisted(() => vi.fn());
vi.mock("@/core/external-link", () => ({ openExternal }));

/** Stands in for a note or the licence: backend-rendered HTML in a clickable box. */
function RenderedNote({ html, onBoxClick }: { html: string; onBoxClick(): void }) {
  const handleAnchorClick = useExternalAnchors();
  return (
    <div
      data-testid="note"
      onClick={(e) => {
        handleAnchorClick(e);
        if (e.defaultPrevented) return;
        onBoxClick();
      }}
      dangerouslySetInnerHTML={{ __html: html }}
    />
  );
}

describe("useExternalAnchors", () => {
  it("sends an external link to the browser instead of navigating the webview", async () => {
    openExternal.mockClear();
    const onBoxClick = vi.fn();
    render(
      <RenderedNote
        html='<p>see <a href="https://ademe.fr/facteurs">the source</a></p>'
        onBoxClick={onBoxClick}
      />,
    );

    await userEvent.click(screen.getByText("the source"));

    expect(openExternal).toHaveBeenCalledWith("https://ademe.fr/facteurs");
    // Clicking a link is not a request to edit the note.
    expect(onBoxClick).not.toHaveBeenCalled();
  });

  /** comrak's autolink turns a pasted URL into a real anchor — including in a
   *  note that arrived from someone else's bundle. */
  it("covers autolinked URLs, not just hand-written anchors", async () => {
    openExternal.mockClear();
    render(
      <RenderedNote
        html='<p><a href="http://basic-presence-support.example/reconnexion">http://basic-presence-support.example/reconnexion</a></p>'
        onBoxClick={vi.fn()}
      />,
    );

    await userEvent.click(
      screen.getByText("http://basic-presence-support.example/reconnexion"),
    );

    expect(openExternal).toHaveBeenCalledWith(
      "http://basic-presence-support.example/reconnexion",
    );
  });

  it("leaves in-document anchors to the webview", async () => {
    openExternal.mockClear();
    const onBoxClick = vi.fn();
    render(
      <RenderedNote
        html='<p><a href="#section-2">jump</a></p>'
        onBoxClick={onBoxClick}
      />,
    );

    await userEvent.click(screen.getByText("jump"));

    expect(openExternal).not.toHaveBeenCalled();
    expect(onBoxClick).toHaveBeenCalled();
  });

  it("clicking plain text still reaches the container handler", async () => {
    openExternal.mockClear();
    const onBoxClick = vi.fn();
    render(
      <RenderedNote html="<p>just a note</p>" onBoxClick={onBoxClick} />,
    );

    await userEvent.click(screen.getByText("just a note"));

    expect(openExternal).not.toHaveBeenCalled();
    expect(onBoxClick).toHaveBeenCalled();
  });

  it("does not hand unexpected schemes to the OS", async () => {
    openExternal.mockClear();
    render(
      <RenderedNote
        html='<p><a href="file:///etc/passwd">local</a></p>'
        onBoxClick={vi.fn()}
      />,
    );

    await userEvent.click(screen.getByText("local"));

    expect(openExternal).not.toHaveBeenCalled();
  });
});
