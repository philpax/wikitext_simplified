import { useState, useCallback, type ReactNode } from 'react';
import type { Spanned, WikitextSimplifiedNode, TemplateParameter, WikitextSimplifiedListItem, WikitextSimplifiedDefinitionListItem, WikitextSimplifiedTableRow, WikitextSimplifiedTableCaption } from './wasm/wikitext_wasm';

// Color scheme for different node types (blue and green theme)
const nodeColors: Record<string, string> = {
  'fragment': 'text-slate-400',
  'template': 'text-emerald-400',
  'template-parameter-use': 'text-emerald-300',
  'heading': 'text-blue-400',
  'link': 'text-cyan-400',
  'ext-link': 'text-cyan-300',
  'bold': 'text-blue-300',
  'italic': 'text-blue-200',
  'blockquote': 'text-teal-400',
  'superscript': 'text-green-300',
  'subscript': 'text-green-300',
  'small': 'text-green-200',
  'preformatted': 'text-teal-300',
  'tag': 'text-emerald-500',
  'text': 'text-green-400',
  'table': 'text-blue-500',
  'ordered-list': 'text-cyan-500',
  'unordered-list': 'text-cyan-500',
  'definition-list': 'text-cyan-400',
  'redirect': 'text-amber-400',
  'horizontal-divider': 'text-slate-500',
  'paragraph-break': 'text-slate-500',
  'newline': 'text-slate-500',
};

interface TreeNodeProps {
  node: Spanned<WikitextSimplifiedNode>;
  depth: number;
  onNodeHover: (span: { start: number; end: number } | null) => void;
  onNodeClick: (span: { start: number; end: number }) => void;
}

function TreeNode({ node, depth, onNodeHover, onNodeClick }: TreeNodeProps) {
  const [isExpanded, setIsExpanded] = useState(depth < 2);
  const nodeType = node.value.type;
  const colorClass = nodeColors[nodeType] || 'text-gray-400';

  const hasChildren = (): boolean => {
    const v = node.value;
    if ('children' in v && Array.isArray(v.children)) return v.children.length > 0;
    if ('items' in v && Array.isArray(v.items)) return v.items.length > 0;
    if ('rows' in v && Array.isArray(v.rows)) return v.rows.length > 0;
    if ('default' in v && v.default) return v.default.length > 0;
    if ('parameters' in v && Array.isArray(v.parameters)) return v.parameters.length > 0;
    return false;
  };

  const renderNodeLabel = (): string => {
    const v = node.value;
    switch (v.type) {
      case 'text':
        return `text: "${v.text.length > 30 ? v.text.slice(0, 30) + '...' : v.text}"`;
      case 'link':
        return `link: [[${v.title}${v.text !== v.title ? '|' + v.text : ''}]]`;
      case 'ext-link':
        return `ext-link: [${v.link}${v.text ? ' ' + v.text : ''}]`;
      case 'template':
        return `template: {{${v.name}}}`;
      case 'template-parameter-use':
        return `param: {{{${v.name}}}}`;
      case 'heading':
        return `heading (h${v.level})`;
      case 'tag':
        return `tag: <${v.name}${v.attributes ? ' ' + v.attributes : ''}>`;
      case 'redirect':
        return `redirect: [[${v.target}]]`;
      default:
        return v.type;
    }
  };

  const renderChildren = () => {
    const v = node.value;
    const children: ReactNode[] = [];

    if ('children' in v && Array.isArray(v.children)) {
      v.children.forEach((child, i) => {
        children.push(
          <TreeNode
            key={`child-${i}`}
            node={child}
            depth={depth + 1}
            onNodeHover={onNodeHover}
            onNodeClick={onNodeClick}
          />
        );
      });
    }

    if ('default' in v && v.default) {
      children.push(
        <div key="default" className="ml-4">
          <span className="text-slate-500 text-xs">default:</span>
          {v.default.map((child, i) => (
            <TreeNode
              key={`default-${i}`}
              node={child}
              depth={depth + 1}
              onNodeHover={onNodeHover}
              onNodeClick={onNodeClick}
            />
          ))}
        </div>
      );
    }

    if ('parameters' in v && Array.isArray(v.parameters) && v.parameters.length > 0) {
      children.push(
        <div key="params" className="ml-4">
          <span className="text-slate-500 text-xs">parameters:</span>
          {(v.parameters as TemplateParameter[]).map((param, i) => (
            <div key={`param-${i}`} className="ml-4 text-emerald-200 text-sm">
              {param.name}={param.value.length > 20 ? param.value.slice(0, 20) + '...' : param.value}
            </div>
          ))}
        </div>
      );
    }

    if ('items' in v && Array.isArray(v.items)) {
      (v.items as (WikitextSimplifiedListItem | WikitextSimplifiedDefinitionListItem)[]).forEach((item, i) => {
        children.push(
          <div key={`item-${i}`} className="ml-4">
            <span className="text-slate-500 text-xs">
              {'type_' in item ? (item.type_ === 'Term' ? ';' : ':') : 'item'}:
            </span>
            {item.content.map((child, j) => (
              <TreeNode
                key={`item-${i}-${j}`}
                node={child}
                depth={depth + 1}
                onNodeHover={onNodeHover}
                onNodeClick={onNodeClick}
              />
            ))}
          </div>
        );
      });
    }

    if ('rows' in v && Array.isArray(v.rows)) {
      if ('captions' in v && (v.captions as WikitextSimplifiedTableCaption[]).length > 0) {
        children.push(
          <div key="captions" className="ml-4">
            <span className="text-slate-500 text-xs">captions:</span>
            {(v.captions as WikitextSimplifiedTableCaption[]).map((caption, i) => (
              <div key={`caption-${i}`} className="ml-4">
                {caption.content.map((child, j) => (
                  <TreeNode
                    key={`caption-${i}-${j}`}
                    node={child}
                    depth={depth + 1}
                    onNodeHover={onNodeHover}
                    onNodeClick={onNodeClick}
                  />
                ))}
              </div>
            ))}
          </div>
        );
      }
      children.push(
        <div key="rows" className="ml-4">
          <span className="text-slate-500 text-xs">rows:</span>
          {(v.rows as WikitextSimplifiedTableRow[]).map((row, i) => (
            <div key={`row-${i}`} className="ml-4">
              <span className="text-slate-400 text-xs">row {i + 1}:</span>
              {row.cells.map((cell, j) => (
                <div key={`cell-${i}-${j}`} className="ml-4">
                  <span className="text-slate-500 text-xs">{cell.is_header ? 'th' : 'td'}:</span>
                  {cell.content.map((child, k) => (
                    <TreeNode
                      key={`cell-${i}-${j}-${k}`}
                      node={child}
                      depth={depth + 1}
                      onNodeHover={onNodeHover}
                      onNodeClick={onNodeClick}
                    />
                  ))}
                </div>
              ))}
            </div>
          ))}
        </div>
      );
    }

    return children;
  };

  const expandable = hasChildren();

  return (
    <div className="font-mono text-sm">
      <div
        className={`flex items-center gap-1 py-0.5 px-1 rounded cursor-pointer hover:bg-slate-700/50 transition-colors ${colorClass}`}
        onMouseEnter={() => onNodeHover(node.span)}
        onMouseLeave={() => onNodeHover(null)}
        onClick={() => {
          if (expandable) setIsExpanded(!isExpanded);
          onNodeClick(node.span);
        }}
      >
        {expandable ? (
          <span className="w-4 text-center text-slate-500">
            {isExpanded ? '▼' : '▶'}
          </span>
        ) : (
          <span className="w-4" />
        )}
        <span>{renderNodeLabel()}</span>
        <span className="text-slate-600 text-xs ml-2">
          [{node.span.start}:{node.span.end}]
        </span>
      </div>
      {isExpanded && expandable && (
        <div className="ml-4 border-l border-slate-700 pl-2">
          {renderChildren()}
        </div>
      )}
    </div>
  );
}

interface TreeViewProps {
  nodes: Spanned<WikitextSimplifiedNode>[];
  onNodeHover: (span: { start: number; end: number } | null) => void;
  onNodeClick: (span: { start: number; end: number }) => void;
}

export default function TreeView({ nodes, onNodeHover, onNodeClick }: TreeViewProps) {
  const handleExpandAll = useCallback(() => {
    // This is a simplified approach - in a real app you'd use state management
    // For now, the tree expands on click
  }, []);

  return (
    <div className="h-full overflow-auto bg-slate-900 p-4 rounded-lg">
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-lg font-semibold text-blue-400">AST Tree</h2>
        <button
          onClick={handleExpandAll}
          className="text-xs text-slate-400 hover:text-slate-200 px-2 py-1 bg-slate-800 rounded"
        >
          Click nodes to expand
        </button>
      </div>
      {nodes.length === 0 ? (
        <div className="text-slate-500 italic">No nodes to display</div>
      ) : (
        <div className="space-y-1">
          {nodes.map((node, i) => (
            <TreeNode
              key={i}
              node={node}
              depth={0}
              onNodeHover={onNodeHover}
              onNodeClick={onNodeClick}
            />
          ))}
        </div>
      )}
    </div>
  );
}
