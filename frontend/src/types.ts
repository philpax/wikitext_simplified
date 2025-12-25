// Types matching the Rust WikitextSimplifiedNode enum

export interface Span {
  start: number;
  end: number;
}

export interface Spanned<T> {
  value: T;
  span: Span;
}

export interface TemplateParameter {
  name: string;
  value: string;
}

export interface WikitextSimplifiedListItem {
  content: Spanned<WikitextSimplifiedNode>[];
}

export interface WikitextSimplifiedDefinitionListItem {
  type_: 'Term' | 'Details';
  content: Spanned<WikitextSimplifiedNode>[];
}

export interface WikitextSimplifiedTableCaption {
  attributes: Spanned<WikitextSimplifiedNode>[] | null;
  content: Spanned<WikitextSimplifiedNode>[];
}

export interface WikitextSimplifiedTableCell {
  is_header: boolean;
  attributes: Spanned<WikitextSimplifiedNode>[] | null;
  content: Spanned<WikitextSimplifiedNode>[];
}

export interface WikitextSimplifiedTableRow {
  attributes: Spanned<WikitextSimplifiedNode>[];
  cells: WikitextSimplifiedTableCell[];
}

export type WikitextSimplifiedNode =
  | { type: 'fragment'; children: Spanned<WikitextSimplifiedNode>[] }
  | { type: 'template'; name: string; parameters: TemplateParameter[] }
  | { type: 'template-parameter-use'; name: string; default: Spanned<WikitextSimplifiedNode>[] | null }
  | { type: 'heading'; level: number; children: Spanned<WikitextSimplifiedNode>[] }
  | { type: 'link'; text: string; title: string }
  | { type: 'ext-link'; link: string; text: string | null }
  | { type: 'bold'; children: Spanned<WikitextSimplifiedNode>[] }
  | { type: 'italic'; children: Spanned<WikitextSimplifiedNode>[] }
  | { type: 'blockquote'; children: Spanned<WikitextSimplifiedNode>[] }
  | { type: 'superscript'; children: Spanned<WikitextSimplifiedNode>[] }
  | { type: 'subscript'; children: Spanned<WikitextSimplifiedNode>[] }
  | { type: 'small'; children: Spanned<WikitextSimplifiedNode>[] }
  | { type: 'preformatted'; children: Spanned<WikitextSimplifiedNode>[] }
  | { type: 'tag'; name: string; attributes: string | null; children: Spanned<WikitextSimplifiedNode>[] }
  | { type: 'text'; text: string }
  | { type: 'table'; attributes: Spanned<WikitextSimplifiedNode>[]; captions: WikitextSimplifiedTableCaption[]; rows: WikitextSimplifiedTableRow[] }
  | { type: 'ordered-list'; items: WikitextSimplifiedListItem[] }
  | { type: 'unordered-list'; items: WikitextSimplifiedListItem[] }
  | { type: 'definition-list'; items: WikitextSimplifiedDefinitionListItem[] }
  | { type: 'redirect'; target: string }
  | { type: 'horizontal-divider' }
  | { type: 'paragraph-break' }
  | { type: 'newline' };

export interface ParseResult {
  nodes: Spanned<WikitextSimplifiedNode>[];
  success: boolean;
  error?: string;
}
