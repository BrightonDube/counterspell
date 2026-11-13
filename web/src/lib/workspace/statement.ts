// A small, safe parser for problem statements (ide-workspace T06). It turns a limited Markdown subset
// into plain data. It never produces HTML: components render every segment as text.

/** A run of text. `code` is true for text that was wrapped in backticks. */
export interface InlineSegment {
  code: boolean;
  text: string;
}

/** One block of a statement, in reading order. */
export type StatementBlock =
  | { kind: 'heading'; level: 2 | 3 | 4; segments: InlineSegment[] }
  | { kind: 'paragraph'; segments: InlineSegment[] }
  | { kind: 'list'; items: InlineSegment[][] }
  | { kind: 'code'; text: string };

const BULLET = /^[-*]\s+/;
const HEADING = /^(#{1,6})\s+(.*)$/;
const FENCE = '```';

/** Splits prose on backticks. Text between a pair of backticks is code. An unpaired backtick runs to the end. */
function inline(text: string): InlineSegment[] {
  return text
    .split('`')
    .map((part, index) => ({ code: index % 2 === 1, text: part }))
    .filter((segment) => segment.text.length > 0);
}

/** Maps one, two, or three-or-more hashes to heading levels 2, 3 and 4. */
function headingLevel(hashes: number): 2 | 3 | 4 {
  if (hashes <= 1) return 2;
  if (hashes === 2) return 3;
  return 4;
}

/** True when a line starts a block of its own, so a paragraph must stop before it. */
function startsBlock(line: string): boolean {
  return line.startsWith(FENCE) || HEADING.test(line) || BULLET.test(line);
}

/** Parses a statement into blocks. Blank lines separate blocks; an unclosed fence runs to the end. */
export function parseStatement(markdown: string): StatementBlock[] {
  const lines = markdown.replace(/\r\n?/g, '\n').split('\n');
  const blocks: StatementBlock[] = [];
  let index = 0;

  while (index < lines.length) {
    const line = lines[index];

    if (line.trim() === '') {
      index += 1;
    } else if (line.startsWith(FENCE)) {
      const body: string[] = [];
      index += 1;
      while (index < lines.length && !lines[index].startsWith(FENCE)) {
        body.push(lines[index]);
        index += 1;
      }
      index += 1;
      blocks.push({ kind: 'code', text: body.join('\n') });
    } else if (HEADING.test(line)) {
      const match = HEADING.exec(line);
      if (match) {
        blocks.push({
          kind: 'heading',
          level: headingLevel(match[1].length),
          segments: inline(match[2])
        });
      }
      index += 1;
    } else if (BULLET.test(line)) {
      const items: InlineSegment[][] = [];
      while (index < lines.length && BULLET.test(lines[index])) {
        items.push(inline(lines[index].replace(BULLET, '')));
        index += 1;
      }
      blocks.push({ kind: 'list', items });
    } else {
      const paragraph: string[] = [];
      while (
        index < lines.length &&
        lines[index].trim() !== '' &&
        !startsBlock(lines[index])
      ) {
        paragraph.push(lines[index]);
        index += 1;
      }
      blocks.push({
        kind: 'paragraph',
        segments: inline(paragraph.join('\n'))
      });
    }
  }

  return blocks;
}
