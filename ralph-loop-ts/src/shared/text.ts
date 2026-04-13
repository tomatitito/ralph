export function trimTrailingWhitespace(text: string): string {
  return text.replace(/[\t ]+$/gm, "");
}
