/**
 * 脚注插件 - 解析 [^1] 引用 和 [^1]: text 定义并渲染
 * 不修改 ProseMirror schema，纯渲染层处理
 */
import { Plugin, PluginKey } from "prosemirror-state";
import { Decoration, DecorationSet } from "prosemirror-view";

export const footnotePluginKey = new PluginKey("footnote");

interface Footnote {
  id: string;
  content: string;
  refPos: number; // 引用位置
  defPos: number;  // 定义位置
}

let footnoteCounter = 0;

export function createFootnotePlugin(): Plugin {
  return new Plugin({
    key: footnotePluginKey,
    state: {
      init() {
        return DecorationSet.empty;
      },
      apply(tr, old) {
        if (!tr.docChanged) return old;
        return buildFootnoteDecorations(tr.doc);
      },
    },
    props: {
      decorations(state) {
        return this.getState(state);
      },
    },
  });
}

function buildFootnoteDecorations(doc: any): DecorationSet {
  const decorations: Decoration[] = [];
  let pos = 0;
  const footnoteDefs: Map<string, { pos: number; content: string }> = new Map();

  // Find definitions: [^1]: text
  doc.descendants((node: any, nodePos: number) => {
    if (!node.isText) return true;
    const text = node.text || "";
    const regex = /^\[(\^[^\]]+)\]:\s*(.+)$/gm;
    let match;
    while ((match = regex.exec(text)) !== null) {
      const id = match[1];
      const content = match[2];
      footnoteDefs.set(id, { pos: nodePos + match.index, content });
    }
    return true;
  });

  if (footnoteDefs.size === 0) return DecorationSet.empty;

  // Find references: [^1]  
  doc.descendants((node: any, nodePos: number) => {
    if (!node.isText) return true;
    const text = node.text || "";
    const regex = /\[(\^[^\]]+)\](?!\s*:)/g;
    let match;
    while ((match = regex.exec(text)) !== null) {
      const id = match[1];
      if (footnoteDefs.has(id)) {
        const from = nodePos + match.index;
        const to = from + match[0].length;
        // Replace reference with superscript number
        footnoteCounter++;
        const numEl = document.createElement("sup");
        numEl.className = "milkup-footnote-ref";
        numEl.textContent = String(footnoteCounter);
        numEl.title = footnoteDefs.get(id)!.content;
        decorations.push(Decoration.inline(from, to, {}, { inclusiveStart: false, inclusiveEnd: false }));
        decorations.push(
          Decoration.widget(to, numEl, { side: 1 })
        );
      }
    }
    return true;
  });

  return DecorationSet.create(doc, decorations);
}
