// Tests for the safe statement parser (ide-workspace T06). The parser never produces HTML, so each test
// names the markup or layout it protects.
import { describe, expect, it } from 'vitest';
import { parseStatement } from './statement';

describe('parseStatement', () => {
  // Breaks if two paragraphs collapse into one, which would glue unrelated sentences together.
  it('splits paragraphs at blank lines', () => {
    expect(parseStatement('First line.\nStill first.\n\nSecond.')).toEqual([
      {
        kind: 'paragraph',
        segments: [{ code: false, text: 'First line.\nStill first.' }]
      },
      { kind: 'paragraph', segments: [{ code: false, text: 'Second.' }] }
    ]);
  });

  // Breaks if backtick names lose their code styling, so `nums` would read as ordinary prose.
  it('marks backtick spans as code segments', () => {
    expect(parseStatement('Use `nums` and `target`.')).toEqual([
      {
        kind: 'paragraph',
        segments: [
          { code: false, text: 'Use ' },
          { code: true, text: 'nums' },
          { code: false, text: ' and ' },
          { code: true, text: 'target' },
          { code: false, text: '.' }
        ]
      }
    ]);
  });

  // Breaks if a fence is parsed as markup, so a sample containing <script> could become live HTML.
  it('keeps fenced code verbatim, including angle brackets', () => {
    expect(parseStatement('```\n<script>x</script>\n```')).toEqual([
      { kind: 'code', text: '<script>x</script>' }
    ]);
  });

  // Breaks if an unclosed fence loops forever or drops its text.
  it('keeps the text of a fence that is never closed', () => {
    expect(parseStatement('```\ncode')).toEqual([
      { kind: 'code', text: 'code' }
    ]);
  });

  // Breaks if consecutive bullets become separate lists, or the paragraph after them is lost.
  it('groups consecutive bullet lines into one list', () => {
    expect(parseStatement('- a\n- b\n\nafter')).toEqual([
      {
        kind: 'list',
        items: [[{ code: false, text: 'a' }], [{ code: false, text: 'b' }]]
      },
      { kind: 'paragraph', segments: [{ code: false, text: 'after' }] }
    ]);
  });

  // Breaks if a raw HTML tag in the statement is interpreted instead of shown as text.
  it('treats raw HTML in prose as plain text', () => {
    expect(parseStatement('<img src=x onerror=alert(1)>')).toEqual([
      {
        kind: 'paragraph',
        segments: [{ code: false, text: '<img src=x onerror=alert(1)>' }]
      }
    ]);
  });

  // Breaks if a single # heading maps to the wrong visual level, which would break the outline order.
  it('maps one, two and three or more hashes to heading levels 2, 3 and 4', () => {
    expect(parseStatement('# One\n\n## Two\n\n#### Four')).toEqual([
      { kind: 'heading', level: 2, segments: [{ code: false, text: 'One' }] },
      { kind: 'heading', level: 3, segments: [{ code: false, text: 'Two' }] },
      { kind: 'heading', level: 4, segments: [{ code: false, text: 'Four' }] }
    ]);
  });
});
