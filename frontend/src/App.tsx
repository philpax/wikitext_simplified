import { useState, useEffect, useCallback, useRef } from 'react';
import init, { parse_wikitext } from './wasm/wikitext_wasm';
import TreeView from './TreeView';
import HtmlPreview from './HtmlPreview';
import { examples } from './examples';
import type { Spanned, WikitextSimplifiedNode } from './types';
import './index.css';

type ViewMode = 'tree' | 'preview' | 'both';

function App() {
  const [wasmReady, setWasmReady] = useState(false);
  const [wikitext, setWikitext] = useState(examples[0].wikitext);
  const [nodes, setNodes] = useState<Spanned<WikitextSimplifiedNode>[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [viewMode, setViewMode] = useState<ViewMode>('both');
  const [highlightSpan, setHighlightSpan] = useState<{ start: number; end: number } | null>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  // Initialize WASM
  useEffect(() => {
    init().then(() => {
      setWasmReady(true);
    }).catch((e) => {
      setError(`Failed to initialize WASM: ${e}`);
    });
  }, []);

  // Parse wikitext with debounce
  useEffect(() => {
    if (!wasmReady) return;

    const timer = setTimeout(() => {
      try {
        const result = parse_wikitext(wikitext);
        if (Array.isArray(result)) {
          setNodes(result as Spanned<WikitextSimplifiedNode>[]);
          setError(null);
        } else {
          setError('Unexpected result format');
          setNodes([]);
        }
      } catch (e) {
        setError(String(e));
        setNodes([]);
      }
    }, 300);

    return () => clearTimeout(timer);
  }, [wikitext, wasmReady]);

  const handleNodeHover = useCallback((span: { start: number; end: number } | null) => {
    setHighlightSpan(span);
  }, []);

  const handleNodeClick = useCallback((span: { start: number; end: number }) => {
    if (textareaRef.current) {
      textareaRef.current.focus();
      textareaRef.current.setSelectionRange(span.start, span.end);
    }
  }, []);

  const handleExampleSelect = useCallback((e: React.ChangeEvent<HTMLSelectElement>) => {
    const example = examples.find(ex => ex.name === e.target.value);
    if (example) {
      setWikitext(example.wikitext);
    }
  }, []);

  // Render textarea with highlighting
  const renderEditor = () => {
    return (
      <div className="relative h-full">
        <textarea
          ref={textareaRef}
          value={wikitext}
          onChange={(e) => setWikitext(e.target.value)}
          className="w-full h-full bg-slate-900 text-green-300 font-mono text-sm p-4 rounded-lg border border-slate-700 focus:border-blue-500 focus:outline-none resize-none"
          placeholder="Enter wikitext here..."
          spellCheck={false}
        />
        {highlightSpan && (
          <div className="absolute top-0 left-0 w-full h-full pointer-events-none overflow-hidden p-4 font-mono text-sm">
            <div className="whitespace-pre-wrap break-words text-transparent">
              {wikitext.slice(0, highlightSpan.start)}
              <span className="bg-blue-500/30 text-transparent rounded">
                {wikitext.slice(highlightSpan.start, highlightSpan.end)}
              </span>
              {wikitext.slice(highlightSpan.end)}
            </div>
          </div>
        )}
      </div>
    );
  };

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-950 via-slate-900 to-slate-950 text-white">
      {/* Header */}
      <header className="border-b border-slate-800 bg-slate-900/50 backdrop-blur-sm">
        <div className="container mx-auto px-4 py-4">
          <div className="flex flex-col md:flex-row md:items-center md:justify-between gap-4">
            <div>
              <h1 className="text-2xl font-bold bg-gradient-to-r from-blue-400 to-green-400 bg-clip-text text-transparent">
                Wikitext Simplified
              </h1>
              <p className="text-slate-400 text-sm">
                Parse and visualize MediaWiki wikitext
              </p>
            </div>
            <div className="flex flex-wrap items-center gap-4">
              {/* Example selector */}
              <select
                onChange={handleExampleSelect}
                className="bg-slate-800 text-slate-200 px-3 py-2 rounded-lg border border-slate-700 focus:border-blue-500 focus:outline-none text-sm"
              >
                <option value="">Load example...</option>
                {examples.map((ex) => (
                  <option key={ex.name} value={ex.name}>
                    {ex.name}
                  </option>
                ))}
              </select>

              {/* View mode toggle */}
              <div className="flex bg-slate-800 rounded-lg p-1">
                <button
                  onClick={() => setViewMode('tree')}
                  className={`px-3 py-1.5 rounded-md text-sm transition-colors ${
                    viewMode === 'tree'
                      ? 'bg-blue-600 text-white'
                      : 'text-slate-400 hover:text-white'
                  }`}
                >
                  Tree
                </button>
                <button
                  onClick={() => setViewMode('preview')}
                  className={`px-3 py-1.5 rounded-md text-sm transition-colors ${
                    viewMode === 'preview'
                      ? 'bg-green-600 text-white'
                      : 'text-slate-400 hover:text-white'
                  }`}
                >
                  Preview
                </button>
                <button
                  onClick={() => setViewMode('both')}
                  className={`px-3 py-1.5 rounded-md text-sm transition-colors ${
                    viewMode === 'both'
                      ? 'bg-gradient-to-r from-blue-600 to-green-600 text-white'
                      : 'text-slate-400 hover:text-white'
                  }`}
                >
                  Both
                </button>
              </div>
            </div>
          </div>
        </div>
      </header>

      {/* Main content */}
      <main className="container mx-auto px-4 py-6">
        {!wasmReady ? (
          <div className="flex items-center justify-center h-64">
            <div className="text-center">
              <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500 mx-auto mb-4"></div>
              <p className="text-slate-400">Loading WASM module...</p>
            </div>
          </div>
        ) : (
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 h-[calc(100vh-12rem)]">
            {/* Editor panel */}
            <div className="flex flex-col min-h-0">
              <div className="flex items-center justify-between mb-2">
                <h2 className="text-lg font-semibold text-slate-300">Wikitext Input</h2>
                <span className="text-xs text-slate-500">
                  {wikitext.length} characters
                </span>
              </div>
              <div className="flex-1 min-h-0">
                {renderEditor()}
              </div>
            </div>

            {/* Output panel(s) */}
            <div className={`flex flex-col min-h-0 ${viewMode === 'both' ? 'gap-4' : ''}`}>
              {error ? (
                <div className="bg-red-900/50 border border-red-700 rounded-lg p-4">
                  <h3 className="text-red-400 font-semibold mb-2">Parse Error</h3>
                  <pre className="text-red-300 text-sm whitespace-pre-wrap">{error}</pre>
                </div>
              ) : (
                <>
                  {(viewMode === 'tree' || viewMode === 'both') && (
                    <div className={viewMode === 'both' ? 'flex-1 min-h-0 overflow-hidden' : 'h-full'}>
                      <TreeView
                        nodes={nodes}
                        onNodeHover={handleNodeHover}
                        onNodeClick={handleNodeClick}
                      />
                    </div>
                  )}
                  {(viewMode === 'preview' || viewMode === 'both') && (
                    <div className={viewMode === 'both' ? 'flex-1 min-h-0 overflow-hidden' : 'h-full'}>
                      <HtmlPreview nodes={nodes} />
                    </div>
                  )}
                </>
              )}
            </div>
          </div>
        )}
      </main>

      {/* Footer */}
      <footer className="border-t border-slate-800 mt-auto">
        <div className="container mx-auto px-4 py-4">
          <p className="text-center text-slate-500 text-sm">
            Powered by{' '}
            <a
              href="https://github.com/philpax/wikitext_simplified"
              className="text-blue-400 hover:text-blue-300"
              target="_blank"
              rel="noopener noreferrer"
            >
              wikitext_simplified
            </a>
            {' '}compiled to WebAssembly
          </p>
        </div>
      </footer>
    </div>
  );
}

export default App;
