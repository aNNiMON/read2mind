import type { MarkedExtension, Token, Tokens } from "marked";
import DOMPurify from "dompurify";
import IconBadgeCheck from "~icons/lucide/badge-check?raw";
import IconBug from "~icons/lucide/bug?raw";
import IconCircleHelp from "~icons/lucide/circle-help?raw";
import IconCircleX from "~icons/lucide/circle-x?raw";
import IconFlame from "~icons/lucide/flame?raw";
import IconInfo from "~icons/lucide/info?raw";
import IconLightbulb from "~icons/lucide/lightbulb?raw";
import IconList from "~icons/lucide/list?raw";
import IconQuote from "~icons/lucide/quote?raw";
import IconStickyNote from "~icons/lucide/sticky-note?raw";
import IconTriangleAlert from "~icons/lucide/triangle-alert?raw";

export type CalloutMeta = {
  variant: string;
  icon: string;
  title: string;
};

const rawSvg = (icon: unknown) => icon as string;

const variantIcons: Record<string, string> = {
  note: rawSvg(IconStickyNote),
  abstract: rawSvg(IconList),
  info: rawSvg(IconInfo),
  todo: rawSvg(IconBadgeCheck),
  tip: rawSvg(IconLightbulb),
  success: rawSvg(IconBadgeCheck),
  question: rawSvg(IconCircleHelp),
  warning: rawSvg(IconTriangleAlert),
  failure: rawSvg(IconCircleX),
  danger: rawSvg(IconFlame),
  bug: rawSvg(IconBug),
  example: rawSvg(IconList),
  quote: rawSvg(IconQuote),
};

const aliases: Record<string, string> = {
  summary: "abstract",
  tldr: "abstract",
  hint: "tip",
  important: "tip",
  check: "success",
  done: "success",
  help: "question",
  faq: "question",
  caution: "warning",
  attention: "warning",
  fail: "failure",
  error: "danger",
  cite: "quote",
};

const calloutPattern = /^\[!([A-Za-z]+)\](?:[ \t]+([^\n]*))?(?:\n|$)/;

const titleCase = (value: string) => value.charAt(0).toUpperCase() + value.slice(1);

const stripCalloutHeader = (paragraph: Tokens.Paragraph, pattern: RegExp) => {
  paragraph.raw = paragraph.raw.replace(pattern, "");
  paragraph.text = paragraph.text.replace(pattern, "");

  const lineBreak = paragraph.tokens.findIndex((token) => token.type === "br");
  paragraph.tokens.splice(0, lineBreak === -1 ? paragraph.tokens.length : lineBreak + 1);
};

/**
 * A marked extension for callouts. Example:
 *
 * > [!NOTE] A custom title
 * > Callout content.
 */
export default function markedCallout(): MarkedExtension {
  return {
    walkTokens(token: Token) {
      if (token.type !== "blockquote" || !token.tokens?.length) return;

      const firstToken = token.tokens[0] as Tokens.Paragraph;
      if (firstToken.type !== "paragraph") return;

      const match = calloutPattern.exec(firstToken.raw);
      if (!match) return;

      const [marker, sourceType, customTitle] = match;
      const requestedType = sourceType.toLowerCase();
      const variant = aliases[requestedType] ?? requestedType;
      const icon = variantIcons[variant];
      if (!icon) return;

      Object.assign(token, {
        type: "callout",
        meta: {
          variant,
          icon,
          title: customTitle?.trim() || titleCase(requestedType),
        } as CalloutMeta,
      });

      stripCalloutHeader(firstToken, new RegExp(marker.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")));
      if (!firstToken.tokens.length) token.tokens.shift();
    },
    extensions: [
      {
        name: "callout",
        level: "block",
        renderer({ meta, tokens = [] }) {
          const cm = meta as CalloutMeta;
          const escapedTitle = DOMPurify.sanitize(cm.title);
          return [
            `<div class="markdown-callout markdown-callout-${cm.variant}">`,
            `<p class="markdown-callout-title">${cm.icon}${escapedTitle}</p>`,
            this.parser.parse(tokens),
            "</div>",
          ].join("\n");
        },
      },
    ],
  };
}
