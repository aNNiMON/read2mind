import type { ItemQuery } from "../api/Item.ts";

export class SearchParser {
  public parseQuery(query: string): ItemQuery {
    const result: ItemQuery = {};

    let input = query.trim();
    input = this.parseTags(input, result);
    input = this.parseDate(input, result);
    input = input.replace(/\s+/g, " ").trim();

    if (input) result.keyword = input;

    return result;
  }

  // #value tag:value -tag:excluded tag:"tag with spaces"
  private parseTags(input: string, result: ItemQuery): string {
    const tagPattern = /(?:^|\s)(-?)(?:#|tag:)(?:"([^"]*)"|([^\s]+))/gi;
    const tags: string[] = [];
    const excludeTags: string[] = [];
    let end = 0;
    let match = null;
    let newInput = "";
    while ((match = tagPattern.exec(input)) !== null) {
      newInput += input.slice(end, match.index);
      const tag = (match[2] ?? match[3] ?? "").trim();
      if (tag) (match[1] === "-" ? excludeTags : tags).push(tag);
      end = tagPattern.lastIndex;
    }
    newInput += input.slice(end);
    if (tags.length > 0) result.tags = tags;
    if (excludeTags.length > 0) result.exclude = excludeTags;
    return newInput.replace(/\s+/g, " ").trim();
  }

  // after:2026-06-06 before:2026-09-01
  private parseDate(input: string, result: ItemQuery): string {
    const datePattern = /(?:^|\s)(after|before):(\d{4}-\d{2}-\d{2})/gi;
    let after: string = "";
    let before: string = "";
    let end = 0;
    let match = null;
    let newInput = "";
    while ((match = datePattern.exec(input)) !== null) {
      newInput += input.slice(end, match.index);
      if (match[1] === "after") after = match[2];
      else if (match[1] === "before") before = match[2];
      end = datePattern.lastIndex;
    }
    newInput += input.slice(end);
    if (after) result.after = after;
    if (before) result.before = before;
    return newInput.replace(/\s+/g, " ").trim();
  }
}
