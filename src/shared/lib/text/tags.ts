export function parseTagList(value: string): string[] {
  const tags: string[] = [];

  for (const tag of value.split(/[,，\n]/).map((item) => item.trim())) {
    if (tag && !tags.some((known) => known.toLowerCase() === tag.toLowerCase())) {
      tags.push(tag);
    }
  }

  return tags;
}
