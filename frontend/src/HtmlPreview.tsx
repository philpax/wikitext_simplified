import type { ReactNode } from 'react';
import type { Spanned, WikitextSimplifiedNode, WikitextSimplifiedListItem, WikitextSimplifiedDefinitionListItem, WikitextSimplifiedTableRow, WikitextSimplifiedTableCaption } from './types';

interface HtmlPreviewProps {
  nodes: Spanned<WikitextSimplifiedNode>[];
}

function renderNode(node: Spanned<WikitextSimplifiedNode>, key: string | number): ReactNode {
  const v = node.value;

  switch (v.type) {
    case 'text':
      return <span key={key}>{v.text}</span>;

    case 'bold':
      return (
        <strong key={key}>
          {v.children.map((child, i) => renderNode(child, `${key}-${i}`))}
        </strong>
      );

    case 'italic':
      return (
        <em key={key}>
          {v.children.map((child, i) => renderNode(child, `${key}-${i}`))}
        </em>
      );

    case 'heading': {
      const sizeClasses: Record<number, string> = {
        1: 'text-3xl',
        2: 'text-2xl',
        3: 'text-xl',
        4: 'text-lg',
        5: 'text-base',
        6: 'text-sm',
      };
      const className = `${sizeClasses[v.level] || 'text-base'} font-bold my-2 text-blue-300`;
      const children = v.children.map((child, i) => renderNode(child, `${key}-${i}`));
      switch (v.level) {
        case 1: return <h1 key={key} className={className}>{children}</h1>;
        case 2: return <h2 key={key} className={className}>{children}</h2>;
        case 3: return <h3 key={key} className={className}>{children}</h3>;
        case 4: return <h4 key={key} className={className}>{children}</h4>;
        case 5: return <h5 key={key} className={className}>{children}</h5>;
        case 6: return <h6 key={key} className={className}>{children}</h6>;
        default: return <h2 key={key} className={className}>{children}</h2>;
      }
    }

    case 'link':
      return (
        <a key={key} href={`https://en.wikipedia.org/wiki/${encodeURIComponent(v.title)}`} className="text-cyan-400 hover:text-cyan-300 underline" target="_blank" rel="noopener noreferrer">
          {v.text}
        </a>
      );

    case 'ext-link':
      return (
        <a key={key} href={v.link} className="text-cyan-400 hover:text-cyan-300 underline" target="_blank" rel="noopener noreferrer">
          {v.text || v.link}
        </a>
      );

    case 'template':
      return (
        <span key={key} className="bg-emerald-900/50 text-emerald-300 px-1 rounded border border-emerald-700">
          {'{{'}
          {v.name}
          {v.parameters.length > 0 && (
            <>
              {'|'}
              {v.parameters.map((p, i) => (
                <span key={i}>
                  {i > 0 && '|'}
                  {p.name}={p.value}
                </span>
              ))}
            </>
          )}
          {'}}'}
        </span>
      );

    case 'template-parameter-use':
      return (
        <span key={key} className="bg-emerald-800/50 text-emerald-200 px-1 rounded border border-emerald-600">
          {'{{{'}
          {v.name}
          {v.default && (
            <>
              {'|'}
              {v.default.map((child, i) => renderNode(child, `${key}-default-${i}`))}
            </>
          )}
          {'}}}'}
        </span>
      );

    case 'blockquote':
      return (
        <blockquote key={key} className="border-l-4 border-teal-500 pl-4 my-2 italic text-slate-300">
          {v.children.map((child, i) => renderNode(child, `${key}-${i}`))}
        </blockquote>
      );

    case 'superscript':
      return (
        <sup key={key}>
          {v.children.map((child, i) => renderNode(child, `${key}-${i}`))}
        </sup>
      );

    case 'subscript':
      return (
        <sub key={key}>
          {v.children.map((child, i) => renderNode(child, `${key}-${i}`))}
        </sub>
      );

    case 'small':
      return (
        <small key={key}>
          {v.children.map((child, i) => renderNode(child, `${key}-${i}`))}
        </small>
      );

    case 'preformatted':
      return (
        <pre key={key} className="bg-slate-800 p-2 rounded font-mono text-sm overflow-x-auto">
          {v.children.map((child, i) => renderNode(child, `${key}-${i}`))}
        </pre>
      );

    case 'tag':
      return (
        <span key={key} className="text-slate-300">
          {v.children.map((child, i) => renderNode(child, `${key}-${i}`))}
        </span>
      );

    case 'fragment':
      return (
        <span key={key}>
          {v.children.map((child, i) => renderNode(child, `${key}-${i}`))}
        </span>
      );

    case 'ordered-list':
      return (
        <ol key={key} className="list-decimal list-inside my-2 ml-4">
          {(v.items as WikitextSimplifiedListItem[]).map((item, i) => (
            <li key={i}>
              {item.content.map((child, j) => renderNode(child, `${key}-${i}-${j}`))}
            </li>
          ))}
        </ol>
      );

    case 'unordered-list':
      return (
        <ul key={key} className="list-disc list-inside my-2 ml-4">
          {(v.items as WikitextSimplifiedListItem[]).map((item, i) => (
            <li key={i}>
              {item.content.map((child, j) => renderNode(child, `${key}-${i}-${j}`))}
            </li>
          ))}
        </ul>
      );

    case 'definition-list':
      return (
        <dl key={key} className="my-2">
          {(v.items as WikitextSimplifiedDefinitionListItem[]).map((item, i) => {
            if (item.type_ === 'Term') {
              return (
                <dt key={i} className="font-bold text-blue-300">
                  {item.content.map((child, j) => renderNode(child, `${key}-${i}-${j}`))}
                </dt>
              );
            } else {
              return (
                <dd key={i} className="ml-4 text-slate-300">
                  {item.content.map((child, j) => renderNode(child, `${key}-${i}-${j}`))}
                </dd>
              );
            }
          })}
        </dl>
      );

    case 'table':
      return (
        <table key={key} className="border-collapse border border-slate-600 my-2">
          {(v.captions as WikitextSimplifiedTableCaption[]).length > 0 && (
            <caption className="text-slate-300 mb-2">
              {(v.captions as WikitextSimplifiedTableCaption[]).map((caption, i) => (
                <span key={i}>
                  {caption.content.map((child, j) => renderNode(child, `${key}-caption-${i}-${j}`))}
                </span>
              ))}
            </caption>
          )}
          <tbody>
            {(v.rows as WikitextSimplifiedTableRow[]).map((row, i) => (
              <tr key={i}>
                {row.cells.map((cell, j) => {
                  const CellTag = cell.is_header ? 'th' : 'td';
                  return (
                    <CellTag key={j} className="border border-slate-600 px-2 py-1">
                      {cell.content.map((child, k) => renderNode(child, `${key}-row-${i}-cell-${j}-${k}`))}
                    </CellTag>
                  );
                })}
              </tr>
            ))}
          </tbody>
        </table>
      );

    case 'redirect':
      return (
        <div key={key} className="text-amber-400 italic">
          #REDIRECT [[{v.target}]]
        </div>
      );

    case 'horizontal-divider':
      return <hr key={key} className="border-slate-600 my-4" />;

    case 'paragraph-break':
      return <br key={key} />;

    case 'newline':
      return <br key={key} />;

    default:
      return <span key={key} className="text-red-400">[Unknown node type]</span>;
  }
}

export default function HtmlPreview({ nodes }: HtmlPreviewProps) {
  return (
    <div className="h-full overflow-auto bg-slate-900 p-4 rounded-lg">
      <h2 className="text-lg font-semibold text-green-400 mb-4">HTML Preview</h2>
      <div className="prose prose-invert max-w-none text-slate-200">
        {nodes.length === 0 ? (
          <div className="text-slate-500 italic">No content to preview</div>
        ) : (
          nodes.map((node, i) => renderNode(node, i))
        )}
      </div>
    </div>
  );
}
